use super::adaptive;
use super::bridge_retry::retry_bridge_submit;
use super::item::RetryItem;
use super::kind::RetryKind;
use super::policy::{compute_backoff, RETRY_PRUNE_AGE_SECS, RETRY_PRUNE_MAX};
use crate::audit::push_audit;
use crate::models::*;
use crate::storage::USERS;
use crate::time::now_secs;

thread_local! { static RETRY_SEQ: std::cell::RefCell<u64> = std::cell::RefCell::new(1); }
pub fn next_retry_id() -> u64 {
    RETRY_SEQ.with(|c| {
        let mut v = c.borrow_mut();
        let id = *v;
        *v += 1;
        id
    })
}

fn prune(q: &mut Vec<RetryItem>, now: u64) {
    // Hard max guard: drop oldest non-terminal first
    if q.len() > RETRY_PRUNE_MAX {
        q.sort_by_key(|r| r.next_attempt_after);
        while q.len() > RETRY_PRUNE_MAX {
            if let Some(pos) = q.iter().position(|r| r.terminal) {
                q.remove(pos);
            } else {
                q.remove(0);
            }
        }
    }
    // Age-based prune for terminal items
    q.retain(|r| !(r.terminal && now.saturating_sub(r.next_attempt_after) > RETRY_PRUNE_AGE_SECS));
    // Prune succeeded (terminal with no last_error) older than 10 minutes to avoid clutter
    let success_age = 600u64;
    q.retain(|r| {
        !(r.terminal
            && r.last_error.is_none()
            && now.saturating_sub(r.next_attempt_after) > success_age)
    });
    // Cap per kind terminal records (keep last 5)
    use std::collections::BTreeMap;
    let mut per: BTreeMap<String, Vec<usize>> = BTreeMap::new();
    for (i, r) in q.iter().enumerate() {
        if r.terminal {
            per.entry(r.kind.name().into()).or_default().push(i);
        }
    }
    let mut to_remove: Vec<usize> = Vec::new();
    for (_k, idxs) in per.into_iter() {
        if idxs.len() > 5 {
            for i in idxs[..idxs.len() - 5].iter() {
                to_remove.push(*i);
            }
        }
    }
    to_remove.sort();
    to_remove.drain(..).rev().for_each(|i| {
        if i < q.len() {
            q.remove(i);
        }
    });
}

pub async fn process(principal: &str, max_attempts: u32) -> u32 {
    // RNG seeding now handled globally via rng::init_rng in canister init/post-upgrade.
    // No action required here; jitter logic in policy is tolerant if RNG not yet initialized.
    // Phase 1: collect due items & record attempt audit events (deferred) without nested mutable borrows
    let (due, attempt_events): (Vec<RetryItem>, Vec<AuditEventKind>) = USERS.with(|users| {
        let mut users = users.borrow_mut();
        if let Some(u) = users.get_mut(principal) {
            let now = now_secs();
            if let Some(q) = u.retry_queue.as_mut() {
                let mut out: Vec<RetryItem> = Vec::new();
                let mut events: Vec<AuditEventKind> = Vec::new();
                for it in q.iter_mut() {
                    if !it.terminal && now >= it.next_attempt_after {
                        it.attempts += 1; // increment attempt counter
                        out.push(it.clone());
                        events.push(AuditEventKind::RetryAttempt {
                            retry_id: it.id,
                            attempt: it.attempts,
                            kind: it.kind.name().into(),
                        });
                    }
                }
                (out, events)
            } else {
                (Vec::new(), Vec::new())
            }
        } else {
            (Vec::new(), Vec::new())
        }
    });
    if !attempt_events.is_empty() {
        USERS.with(|users| {
            let mut users = users.borrow_mut();
            if let Some(u) = users.get_mut(principal) {
                for ev in attempt_events {
                    push_audit(u, ev);
                }
            }
        });
    }
    if due.is_empty() {
        return 0;
    }
    let mut bridge_errors: Vec<(u64, String)> = Vec::new();
    let mut bridge_success: Vec<u64> = Vec::new();
    let mut escrow_success: Vec<u64> = Vec::new();
    let mut escrow_errors: Vec<(u64, String)> = Vec::new();
    for item in due.iter() {
        match &item.kind {
            RetryKind::FungibleCustodyRelease { .. } => {
                crate::api::executor::attempt_fungible_custody_releases_for(principal).await;
            }
            RetryKind::NftCustodyRelease { .. } => {
                crate::api::executor::attempt_nft_custody_releases(principal).await;
            }
            RetryKind::BridgeSubmit { asset_id, heir_id } => {
                match retry_bridge_submit(principal, *asset_id, *heir_id).await {
                    Ok(()) => bridge_success.push(item.id),
                    Err(e) => bridge_errors.push((item.id, e)),
                }
            }
            RetryKind::BridgePoll { asset_id, heir_id } => {
                // attempt poll (session id unknown in retry context => use 0 sentinel; underlying function will no-op if session invalid)
                if let Err(e) = crate::api::ckbridge::poll_ck_withdraw(0, *asset_id, *heir_id).await
                {
                    bridge_errors.push((item.id, format!("poll_err:{:?}", e)));
                } else {
                    // inspect status to decide terminal
                    let terminal = USERS.with(|users| {
                        let users = users.borrow();
                        if let Some(u) = users.get(principal) {
                            u.ck_withdraws
                                .iter()
                                .find(|r| r.asset_id == *asset_id && r.heir_id == *heir_id)
                                .map(|r| {
                                    matches!(
                                        r.bridge_status,
                                        Some(BridgeStatus::Completed)
                                            | Some(BridgeStatus::Failed(_))
                                            | Some(BridgeStatus::Reimbursed)
                                    )
                                })
                                .unwrap_or(false)
                        } else {
                            false
                        }
                    });
                    if terminal {
                        bridge_success.push(item.id);
                    }
                }
            }
            RetryKind::EscrowRelease { asset_id, heir_id } => {
                // Emit attempt event with synthetic attempt number from item.attempts
                USERS.with(|users| {
                    let mut users = users.borrow_mut();
                    if let Some(u) = users.get_mut(principal) {
                        push_audit(
                            u,
                            AuditEventKind::EscrowReleaseAttempt {
                                asset_id: *asset_id,
                                heir_id: *heir_id,
                                amount: 0,
                                attempt: item.attempts,
                            },
                        );
                    }
                });
                match crate::api::executor::attempt_escrow_release(principal, *asset_id, *heir_id)
                    .await
                {
                    Ok(()) => escrow_success.push(item.id),
                    Err(e) => {
                        escrow_errors.push((item.id, e.clone()));
                        USERS.with(|users| {
                            let mut users = users.borrow_mut();
                            if let Some(u) = users.get_mut(principal) {
                                push_audit(
                                    u,
                                    AuditEventKind::EscrowReleaseFailed {
                                        asset_id: *asset_id,
                                        heir_id: *heir_id,
                                        amount: 0,
                                        attempt: item.attempts,
                                        error: e,
                                    },
                                );
                            }
                        });
                    }
                }
            }
        }
    }
    // Phase 2: update queue items & collect outcome audit events
    let outcome_events: Vec<AuditEventKind> = USERS.with(|users| {
        let mut users = users.borrow_mut();
        if let Some(u) = users.get_mut(principal) {
            if let Some(q) = u.retry_queue.as_mut() {
                let now = now_secs();
                let mut events: Vec<AuditEventKind> = Vec::new();
                for rec in q.iter_mut() {
                    if due.iter().any(|d| d.id == rec.id) {
                        if let Some((_, err)) = bridge_errors.iter().find(|(id, _)| *id == rec.id) {
                            rec.last_error = Some(err.clone());
                        }
                        if let Some((_, err)) = escrow_errors.iter().find(|(id, _)| *id == rec.id) {
                            rec.last_error = Some(err.clone());
                        }
                        let kind_name = rec.kind.name().to_string();
                        if bridge_success.iter().any(|id| *id == rec.id)
                            || escrow_success.iter().any(|id| *id == rec.id)
                        {
                            rec.terminal = true;
                            events.push(AuditEventKind::RetrySucceeded {
                                retry_id: rec.id,
                                attempts: rec.attempts,
                                kind: kind_name.clone(),
                            });
                            adaptive::record_outcome(&kind_name, true, principal);
                            continue;
                        }
                        if rec.attempts >= max_attempts {
                            rec.terminal = true;
                            events.push(AuditEventKind::RetryTerminal {
                                retry_id: rec.id,
                                attempts: rec.attempts,
                                kind: kind_name.clone(),
                            });
                            adaptive::record_outcome(&kind_name, false, principal);
                            continue;
                        }
                        let delay = compute_backoff(&rec.kind, rec.attempts);
                        let adjusted = adaptive::adjust_delay(&kind_name, delay, principal);
                        rec.next_attempt_after = now.saturating_add(adjusted);
                        // Only record failure outcome when we set a new delay and there is a last_error this round
                        if rec.last_error.is_some() {
                            adaptive::record_outcome(&kind_name, false, principal);
                        }
                    }
                }
                prune(q, now);
                events
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        }
    });
    if !outcome_events.is_empty() {
        USERS.with(|users| {
            let mut users = users.borrow_mut();
            if let Some(u) = users.get_mut(principal) {
                for ev in outcome_events {
                    push_audit(u, ev);
                }
            }
        });
    }
    due.len() as u32
}
