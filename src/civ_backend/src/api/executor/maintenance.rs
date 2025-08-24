//! Maintenance timer, custody release attempts, retry processing & auto execution.

use super::work;
use crate::audit::push_audit;
use crate::models::*;
use crate::storage::USERS;
use crate::time::now_secs;
use candid::Principal;
use ic_cdk_timers::TimerId;

thread_local! { static MAINT_TIMER: std::cell::RefCell<Option<TimerId>> = std::cell::RefCell::new(None); }

fn maintenance_tick() {
    crate::api::executor::run_phase_maintenance_once();
    // Reconciliation pass (lightweight selection, async spawn per user)
    let now = now_secs();
    let recon_users = crate::api::reconciliation::users_needing_recon(now);
    for u in recon_users.into_iter() {
        ic_cdk::futures::spawn_017_compat(async move {
            let _ = crate::api::reconciliation::reconcile_custody_for(&u).await;
        });
    }
    let principals: Vec<String> = USERS.with(|users| {
        let users = users.borrow();
        users
            .iter()
            .filter_map(|(k, u)| {
                // Determine if sessions need purge (expired)
                let sessions_expired = u.sessions.iter().any(|s| now > s.expires_at);
                let nft_pending = u
                    .nft_custody
                    .iter()
                    .any(|c| c.released_at.is_none() && !c.releasing);
                let fungible_pending = u
                    .fungible_custody
                    .iter()
                    .any(|c| c.released_at.is_none() && !c.releasing);
                let retries_pending = u
                    .retry_queue
                    .as_ref()
                    .map(|q| q.iter().any(|i| !i.terminal))
                    .unwrap_or(false);
                let notifications_pending = u.notifications.iter().any(|n| n.sent_at.is_none());
                if nft_pending
                    || fungible_pending
                    || retries_pending
                    || notifications_pending
                    || sessions_expired
                {
                    Some(k.clone())
                } else {
                    None
                }
            })
            .collect()
    });
    for p in principals {
        ic_cdk::futures::spawn_017_compat(async move {
            crate::api::executor::attempt_nft_custody_releases(&p).await;
            crate::api::executor::attempt_fungible_custody_releases_for(&p).await;
            crate::api::retry::process_retries_for(&p, 8).await;
            // Process notifications (synchronous small loop)
            crate::api::notify::process_notifications_for(&p, 10);
            // Purge expired sessions & emit audits
            crate::api::executor::purge_expired_sessions_for(&p, 64);
            // Capture metrics frame into ring buffer
            crate::api::executor::capture_metrics_frame_for(&p, 168); // keep last 168 frames (~weekly if hourly)
                                                                      // Evaluate escrow auto management (top-up / reclaim)
            crate::api::executor::scan_escrow_auto_actions(&p, 16);
        });
    }
}

pub fn schedule_maintenance() {
    MAINT_TIMER.with(|cell| {
        if cell.borrow().is_some() {
            return;
        }
        let id = ic_cdk_timers::set_timer_interval(std::time::Duration::from_secs(3600), || {
            maintenance_tick()
        });
        *cell.borrow_mut() = Some(id);
    });
}

pub fn perform_maintenance() -> Result<(), CivError> {
    let now = now_secs();
    #[derive(Clone)]
    enum TransitionKind {
        ToWarning,
        ToLocked,
    }
    struct Transition {
        principal: String,
        kind: TransitionKind,
    }
    let transitions: Vec<Transition> = USERS.with(|users| {
        let users_ref = users.borrow();
        users_ref
            .iter()
            .filter_map(|(k, u)| match u.phase {
                EstatePhase::Draft => {
                    if u.timer_expiry > 0
                        && u.timer_expiry.saturating_sub(now) <= WARNING_WINDOW_SECS
                    {
                        Some(Transition {
                            principal: k.clone(),
                            kind: TransitionKind::ToWarning,
                        })
                    } else {
                        None
                    }
                }
                EstatePhase::Warning => {
                    if u.timer_expiry > 0 && now >= u.timer_expiry {
                        Some(Transition {
                            principal: k.clone(),
                            kind: TransitionKind::ToLocked,
                        })
                    } else {
                        None
                    }
                }
                _ => None,
            })
            .collect()
    });
    let mut to_execute: Vec<String> = Vec::new();
    USERS.with(|users| {
        let mut users_mut = users.borrow_mut();
        for t in transitions.into_iter() {
            if let Some(u) = users_mut.get_mut(&t.principal) {
                match t.kind {
                    TransitionKind::ToWarning => {
                        if matches!(u.phase, EstatePhase::Draft) {
                            let from = u.phase.clone();
                            u.phase = EstatePhase::Warning;
                            u.warning_started_at = Some(now);
                            push_audit(
                                u,
                                AuditEventKind::PhaseChanged {
                                    from,
                                    to: u.phase.clone(),
                                },
                            );
                        }
                    }
                    TransitionKind::ToLocked => {
                        if matches!(u.phase, EstatePhase::Warning) {
                            let from = u.phase.clone();
                            u.phase = EstatePhase::Locked;
                            u.locked_at = Some(now);
                            push_audit(
                                u,
                                AuditEventKind::PhaseChanged {
                                    from,
                                    to: u.phase.clone(),
                                },
                            );
                            if !u.distributions_v2.is_empty() && u.execution_nonce.is_none() {
                                to_execute.push(u.user.clone());
                            }
                        }
                    }
                }
            }
        }
    });
    for principal in to_execute.into_iter() {
        ic_cdk::futures::spawn_017_compat(async move {
            let _ = execute_trigger_for(&principal).await;
        });
    }
    Ok(())
}

async fn execute_trigger_for(principal: &str) -> Result<(), CivError> {
    let snapshot = work::snapshot_workitems(principal)?;
    if snapshot.is_empty() {
        return Ok(());
    }
    let already = USERS.with(|users| {
        let mut users = users.borrow_mut();
        if let Some(u) = users.get_mut(principal) {
            if u.phase == EstatePhase::Executed {
                return Some(CivError::AlreadyExecuted);
            }
            if u.execution_nonce.is_some() {
                return Some(CivError::Other("execution_in_progress".into()));
            }
            u.execution_nonce = Some(now_secs());
            None
        } else {
            Some(CivError::UserNotFound)
        }
    });
    if let Some(err) = already {
        return Err(err);
    }
    let mut records: Vec<TransferRecord> = Vec::new();
    let ts = now_secs();
    let mut next_id = work::next_transfer_id(principal);
    let mut ck_staged_count: u64 = 0;
    for w in snapshot.iter() {
        let (record, ck_stage) = work::process_workitem(principal, w, ts, next_id).await;
        if ck_stage.is_some() {
            ck_staged_count += 1;
        }
        records.push(record);
        if let Some(stage) = ck_stage {
            work::stage_ck_withdraw(principal, stage);
        }
        next_id += 1;
    }
    let total_items = records.len() as u64;
    let failure_count = records.iter().filter(|r| r.error.is_some()).count() as u64;
    let skipped_count = records
        .iter()
        .filter(|r| r.note.as_deref() == Some("zero_amount_skip"))
        .count() as u64;
    let success_count = total_items.saturating_sub(failure_count + skipped_count);
    let summary = ExecutionSummary {
        started_at: ts,
        finished_at: now_secs(),
        total_items,
        success_count,
        failure_count,
        skipped_count,
        ck_staged_count,
        auto: true,
    };
    work::finalize_execution(principal, records, ts, summary);
    Ok(())
}

pub async fn attempt_nft_custody_releases(user_principal: &str) {
    let items: Vec<(u64, u64, u64)> = USERS.with(|users| {
        let mut users = users.borrow_mut();
        if let Some(u) = users.get_mut(user_principal) {
            u.nft_custody
                .iter_mut()
                .filter(|c| c.released_at.is_none() && !c.releasing)
                .filter_map(|c| {
                    let now = now_secs();
                    if let Some(next_at) = c.next_attempt_after {
                        if now < next_at {
                            return None;
                        }
                    }
                    c.releasing = true;
                    Some((c.asset_id, c.heir_id, c.token_id))
                })
                .collect()
        } else {
            Vec::new()
        }
    });
    struct ReleaseOutcome {
        asset_id: u64,
        heir_id: u64,
        token_id: u64,
        success: bool,
        err: Option<String>,
    }
    let mut outcomes: Vec<ReleaseOutcome> = Vec::with_capacity(items.len());
    for (asset_id, heir_id, token_id) in items.into_iter() {
        let (to_principal_opt, token_canister_opt) = USERS.with(|users| {
            let users = users.borrow();
            if let Some(u) = users.get(user_principal) {
                let heir_principal = u
                    .heirs_v2
                    .iter()
                    .find(|h| h.id == heir_id)
                    .and_then(|h| h.principal.clone());
                let asset_can = u
                    .assets
                    .iter()
                    .find(|a| a.id == asset_id)
                    .and_then(|a| a.token_canister.clone());
                (heir_principal, asset_can)
            } else {
                (None, None)
            }
        });
        let mut success = false;
        let mut err_msg: Option<String> = None;
        if let (Some(to_p_txt), Some(can_txt)) = (to_principal_opt, token_canister_opt) {
            if let (Ok(tp), Ok(can_principal)) = (
                Principal::from_text(&to_p_txt),
                Principal::from_text(&can_txt),
            ) {
                // Determine NFT standard for adapter selection
                let standard_opt = USERS.with(|users| {
                    let users = users.borrow();
                    users.get(user_principal).and_then(|u| {
                        u.assets
                            .iter()
                            .find(|a| a.id == asset_id)
                            .and_then(|a| a.nft_standard.clone())
                    })
                });
                let adapter = crate::api::nft_adapter::adapter_for(standard_opt.clone());
                let outcome = adapter.transfer(can_principal, tp, token_id).await;
                match outcome {
                    crate::api::nft_adapter::NftTransferOutcome::Success { .. } => success = true,
                    crate::api::nft_adapter::NftTransferOutcome::Failure { code, .. } => {
                        err_msg = Some(code)
                    }
                }
            } else {
                err_msg = Some("invalid_principal".into());
            }
        } else {
            err_msg = Some("missing_destination_or_canister".into());
        }
        outcomes.push(ReleaseOutcome {
            asset_id,
            heir_id,
            token_id,
            success,
            err: err_msg,
        });
    }
    USERS.with(|users| {
        let mut users = users.borrow_mut();
        if let Some(u) = users.get_mut(user_principal) {
            let now = now_secs();
            for o in outcomes.into_iter() {
                let mut emit: Vec<AuditEventKind> = Vec::new();
                if let Some(rec) = u.nft_custody.iter_mut().find(|c| {
                    c.asset_id == o.asset_id && c.heir_id == o.heir_id && c.token_id == o.token_id
                }) {
                    rec.releasing = false;
                    rec.attempts += 1;
                    if o.success {
                        rec.released_at = Some(now);
                        rec.last_error = None;
                        rec.next_attempt_after = None;
                        emit.push(AuditEventKind::NftCustodyReleased {
                            asset_id: o.asset_id,
                            heir_id: o.heir_id,
                            token_id: o.token_id,
                        });
                    } else {
                        rec.last_error = o.err.clone();
                        let base: u64 = 60;
                        let exp = if rec.attempts == 0 {
                            0
                        } else {
                            rec.attempts - 1
                        };
                        let shift = std::cmp::min(exp as u32, 10);
                        let factor = 1u64 << shift;
                        let backoff = base.saturating_mul(factor);
                        let capped = backoff.min(24 * 3600);
                        rec.next_attempt_after = Some(now.saturating_add(capped));
                        let attempt_count = rec.attempts;
                        emit.push(AuditEventKind::NftCustodyReleaseAttempt {
                            asset_id: o.asset_id,
                            heir_id: o.heir_id,
                            token_id: o.token_id,
                            attempt: attempt_count,
                        });
                        if let Some(error_msg) = o.err {
                            emit.push(AuditEventKind::NftCustodyReleaseFailed {
                                asset_id: o.asset_id,
                                heir_id: o.heir_id,
                                token_id: o.token_id,
                                attempt: attempt_count,
                                error: error_msg,
                            });
                        }
                    }
                }
                for ev in emit.into_iter() {
                    push_audit(u, ev);
                }
            }
        }
    });
}

pub async fn attempt_fungible_custody_releases_for(user_principal: &str) {
    struct Item {
        idx: usize,
        asset_id: u64,
        heir_id: u64,
        amount: u128,
        token_canister: Principal,
        heir_principal: Principal,
        use_approval: bool,
        attempt_next: u32,
    }
    let items: Vec<Item> = USERS.with(|users| {
        let mut users = users.borrow_mut();
        if let Some(u) = users.get_mut(user_principal) {
            u.fungible_custody
                .iter_mut()
                .enumerate()
                .filter_map(|(idx, rec)| {
                    let now = now_secs();
                    if rec.released_at.is_none() && !rec.releasing {
                        if let Some(next_at) = rec.next_attempt_after {
                            if now < next_at {
                                return None;
                            }
                        }
                        let asset = u.assets.iter().find(|a| a.id == rec.asset_id)?;
                        let token_canister_principal = asset
                            .token_canister
                            .as_ref()
                            .and_then(|t| Principal::from_text(t).ok())?;
                        let heir = u.heirs_v2.iter().find(|h| h.id == rec.heir_id)?;
                        let heir_principal = heir
                            .principal
                            .as_ref()
                            .and_then(|p| Principal::from_text(p).ok())?;
                        let mut use_approval = false;
                        if let Some(mode) = &asset.holding_mode {
                            if matches!(mode, HoldingMode::Approval) {
                                use_approval = true;
                            }
                        }
                        if rec.amount == 0 {
                            return None;
                        }
                        rec.releasing = true;
                        let attempt_next = rec.attempts + 1;
                        Some(Item {
                            idx,
                            asset_id: rec.asset_id,
                            heir_id: rec.heir_id,
                            amount: rec.amount,
                            token_canister: token_canister_principal,
                            heir_principal,
                            use_approval,
                            attempt_next,
                        })
                    } else {
                        None
                    }
                })
                .collect()
        } else {
            Vec::new()
        }
    });
    if items.is_empty() {
        return;
    }
    USERS.with(|users| {
        let mut users = users.borrow_mut();
        if let Some(u) = users.get_mut(user_principal) {
            for it in items.iter() {
                push_audit(
                    u,
                    AuditEventKind::FungibleCustodyReleaseAttempt {
                        asset_id: it.asset_id,
                        heir_id: it.heir_id,
                        amount: it.amount,
                        attempt: it.attempt_next,
                    },
                );
            }
        }
    });
    struct Outcome {
        idx: usize,
        success: bool,
        err: Option<String>,
    }
    let mut outcomes: Vec<Outcome> = Vec::with_capacity(items.len());
    let owner_principal = Principal::from_text(user_principal).ok();
    for it in items.iter() {
        let (tx_ok, err, used_approval) = if it.use_approval {
            if let Some(owner) = owner_principal {
                let (tx, e) = work::icrc2_transfer_from(
                    it.token_canister,
                    owner,
                    None,
                    it.heir_principal,
                    None,
                    it.amount,
                )
                .await;
                (tx, e, true)
            } else {
                (None, Some("invalid_owner_principal".into()), true)
            }
        } else {
            let (tx, e) =
                work::icrc1_transfer(it.token_canister, it.heir_principal, None, it.amount).await;
            (tx, e, false)
        };
        if used_approval && err.is_none() && tx_ok.is_some() {
            USERS.with(|users| {
                let mut users = users.borrow_mut();
                if let Some(u) = users.get_mut(user_principal) {
                    if let Some(appr) = u.approvals.iter_mut().find(|a| a.asset_id == it.asset_id) {
                        if let Some(alw) = &mut appr.allowance {
                            *alw = alw.saturating_sub(it.amount);
                        }
                    }
                }
            });
        }
        if err.is_none() && tx_ok.is_some() {
            outcomes.push(Outcome {
                idx: it.idx,
                success: true,
                err: None,
            });
        } else {
            outcomes.push(Outcome {
                idx: it.idx,
                success: false,
                err,
            });
        }
    }
    USERS.with(|users| {
        let mut users = users.borrow_mut();
        if let Some(u) = users.get_mut(user_principal) {
            let now = now_secs();
            for o in outcomes.into_iter() {
                let mut emit: Vec<AuditEventKind> = Vec::new();
                if let Some(rec) = u.fungible_custody.get_mut(o.idx) {
                    rec.releasing = false;
                    rec.attempts += 1;
                    if o.success {
                        rec.released_at = Some(now);
                        rec.last_error = None;
                        rec.next_attempt_after = None;
                        emit.push(AuditEventKind::FungibleCustodyReleased {
                            asset_id: rec.asset_id,
                            heir_id: rec.heir_id,
                            amount: rec.amount,
                        });
                    } else {
                        rec.last_error = o.err.clone();
                        let attempt = rec.attempts;
                        let err_text = rec.last_error.clone().unwrap_or_else(|| "unknown".into());
                        let delay = (60u64).saturating_mul(
                            2u64.saturating_pow((attempt.saturating_sub(1)) as u32),
                        );
                        let capped = delay.min(24 * 3600);
                        rec.next_attempt_after = Some(now.saturating_add(capped));
                        emit.push(AuditEventKind::FungibleCustodyReleaseFailed {
                            asset_id: rec.asset_id,
                            heir_id: rec.heir_id,
                            amount: rec.amount,
                            attempt,
                            error: err_text,
                        });
                    }
                }
                for ev in emit.into_iter() {
                    push_audit(u, ev);
                }
            }
        }
    });
}

// Escrow release attempt: executes one transfer per (asset_id, heir_id) pair derived from a scheduled retry.
// We recompute heir share against current escrow record each attempt (idempotent if already reduced externally once implemented).
pub async fn attempt_escrow_release(
    user_principal: &str,
    asset_id: u64,
    heir_id: u64,
) -> Result<(), String> {
    // Snapshot needed data
    struct Snap {
        token: Principal,
        heir: Principal,
        amount: u128,
    }
    let snap_opt: Option<Snap> = USERS.with(|users| {
        let users = users.borrow();
        let u = users.get(user_principal)?;
        let asset = u.assets.iter().find(|a| a.id == asset_id)?;
        let token_p = asset
            .token_canister
            .as_ref()
            .and_then(|t| Principal::from_text(t).ok())?;
        let heir = u.heirs_v2.iter().find(|h| h.id == heir_id)?;
        let heir_p = heir
            .principal
            .as_ref()
            .and_then(|p| Principal::from_text(p).ok())?;
        let esc = u.escrow.iter().find(|e| e.asset_id == asset_id)?;
        let pct = u
            .distributions_v2
            .iter()
            .find(|d| d.asset_id == asset_id && d.heir_id == heir_id)
            .map(|d| d.percentage as u128)
            .unwrap_or(0);
        let total = esc.amount.unwrap_or(0);
        if total == 0 || pct == 0 {
            return None;
        }
        let share = total.saturating_mul(pct).checked_div(100).unwrap_or(0);
        Some(Snap {
            token: token_p,
            heir: heir_p,
            amount: share,
        })
    });
    let snap = match snap_opt {
        Some(s) => s,
        None => return Err("escrow_share_zero".into()),
    };
    let sub = crate::crypto::derive_escrow_subaccount(user_principal, asset_id);
    let (tx_opt, err_opt) = crate::api::executor::work::icrc1_transfer_from_sub(
        snap.token,
        sub.to_vec(),
        snap.heir,
        None,
        snap.amount,
    )
    .await;
    if let Some(e) = err_opt {
        return Err(format!("escrow_transfer_err:{}", e));
    }
    if tx_opt.is_none() {
        return Err("escrow_transfer_noindex".into());
    }
    // On success, emit audit event and mark corresponding transfer record note
    USERS.with(|users| {
        let mut users = users.borrow_mut();
        if let Some(u) = users.get_mut(user_principal) {
            // Decrement escrow logical record by released share (idempotent: share computed from current amount; retry terminates after success)
            if let Some(rec) = u.escrow.iter_mut().find(|e| e.asset_id == asset_id) {
                if let Some(total) = rec.amount {
                    rec.amount = Some(total.saturating_sub(snap.amount));
                }
            }
            // Remove escrow record if fully depleted (amount == 0) to keep model clean
            u.escrow
                .retain(|e| !(e.asset_id == asset_id && e.amount == Some(0)));
            push_audit(
                u,
                AuditEventKind::EscrowReleased {
                    asset_id,
                    heir_id,
                    amount: snap.amount,
                },
            );
            // update most recent matching transfer note if still enqueued
            if let Some(rec) = u.transfers.iter_mut().rev().find(|r| {
                r.asset_id == Some(asset_id)
                    && r.heir_id == Some(heir_id)
                    && r.note.as_deref() == Some("escrow_release_enqueued")
            }) {
                rec.note = Some("escrow_released".into());
            }
        }
    });
    Ok(())
}
