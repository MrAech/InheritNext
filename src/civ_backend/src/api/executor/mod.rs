//! Executor facade: estate lifecycle, manual execution trigger, queries.
//! Internal logic split across submodules (work, maintenance, stable) to keep
//! this file concise while preserving the original public API surface.

pub mod maintenance;
pub mod stable;
pub mod work;

use crate::api::common::{maybe_advance_phase, user_id};
use crate::audit::push_audit;
use crate::models::*;
use crate::storage::USERS;
use crate::time::now_secs;

pub use maintenance::{
    attempt_escrow_release, attempt_fungible_custody_releases_for, attempt_nft_custody_releases,
    perform_maintenance, schedule_maintenance,
};
pub use stable::{post_upgrade, pre_upgrade};
// Re-export selected work helpers needed by other API modules (custody uses icrc1_transfer directly)
pub use work::{icrc1_transfer, icrc1_transfer_from_sub};

// -------- Estate status & lifecycle (public) --------
pub fn estate_status() -> EstateStatus {
    let caller = user_id();
    USERS.with(|users| {
        let users = users.borrow();
        if let Some(u) = users.get(&caller) {
            let now = now_secs();
            let remaining = if u.timer_expiry == 0 {
                -1
            } else {
                u.timer_expiry.saturating_sub(now) as i64
            };
            EstateStatus {
                phase: u.phase.clone(),
                seconds_to_expiry: remaining,
                warning_started_at: u.warning_started_at,
                locked_at: u.locked_at,
                executed_at: u.executed_at,
            }
        } else {
            EstateStatus {
                phase: EstatePhase::Draft,
                seconds_to_expiry: -1,
                warning_started_at: None,
                locked_at: None,
                executed_at: None,
            }
        }
    })
}

pub fn start_warning() -> Result<(), CivError> {
    let caller = user_id();
    USERS.with(|users| {
        let mut users = users.borrow_mut();
        if let Some(u) = users.get_mut(&caller) {
            if matches!(u.phase, EstatePhase::Draft) {
                let from = u.phase.clone();
                u.phase = EstatePhase::Warning;
                u.warning_started_at = Some(now_secs());
                push_audit(
                    u,
                    AuditEventKind::PhaseChanged {
                        from,
                        to: u.phase.clone(),
                    },
                );
                Ok(())
            } else {
                Err(CivError::Other("cannot_start_warning".into()))
            }
        } else {
            Err(CivError::UserNotFound)
        }
    })
}

pub fn lock_estate() -> Result<(), CivError> {
    let caller = user_id();
    USERS.with(|users| {
        let mut users = users.borrow_mut();
        if let Some(u) = users.get_mut(&caller) {
            if u.phase == EstatePhase::Locked || u.phase == EstatePhase::Executed {
                return Err(CivError::EstateLocked);
            }
            let readiness = crate::api::distributions::estate_readiness();
            if !readiness.ready {
                return Err(CivError::EstateNotReady);
            }
            let from = u.phase.clone();
            u.phase = EstatePhase::Locked;
            u.locked_at = Some(now_secs());
            push_audit(
                u,
                AuditEventKind::PhaseChanged {
                    from,
                    to: u.phase.clone(),
                },
            );
            Ok(())
        } else {
            Err(CivError::UserNotFound)
        }
    })
}

// -------- Execution trigger (public) --------
pub async fn execute_trigger() -> Result<(), CivError> {
    let caller = user_id();
    let already = USERS.with(|users| {
        let mut users = users.borrow_mut();
        if let Some(u) = users.get_mut(&caller) {
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

    let snapshot = work::snapshot_workitems(&caller)?;
    if snapshot.is_empty() {
        return Err(CivError::EstateLocked);
    }

    let mut records: Vec<TransferRecord> = Vec::new();
    let ts = now_secs();
    let mut next_id = work::next_transfer_id(&caller);
    let mut ck_staged_count: u64 = 0;
    for w in snapshot.iter() {
        let (record, ck_stage) = work::process_workitem(&caller, w, ts, next_id).await;
        if ck_stage.is_some() {
            ck_staged_count += 1;
        }
        records.push(record);
        if let Some(stage) = ck_stage {
            work::stage_ck_withdraw(&caller, stage);
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
        auto: false,
    };
    work::finalize_execution(&caller, records, ts, summary);
    Ok(())
}

// -------- Queries --------
pub fn list_transfers() -> Vec<TransferRecord> {
    let caller = user_id();
    USERS.with(|users| {
        let users = users.borrow();
        users
            .get(&caller)
            .map(|u| u.transfers.clone())
            .unwrap_or_default()
    })
}

// Compute a Merkle-like root over ordered transfers (simple binary hash fold) and store attestation.
pub fn compute_ledger_attestation() -> Result<Vec<u8>, CivError> {
    let caller = user_id();
    use sha2::{Digest, Sha256};
    USERS.with(|users| {
        let mut users = users.borrow_mut();
        if let Some(u) = users.get_mut(&caller) {
            let mut leaves: Vec<[u8; 32]> = Vec::new();
            for t in &u.transfers {
                // deterministic leaf serialization
                let mut h = Sha256::new();
                h.update(&t.id.to_le_bytes());
                h.update(&t.timestamp.to_le_bytes());
                h.update(&t.asset_id.unwrap_or(0).to_le_bytes());
                h.update(&t.heir_id.unwrap_or(0).to_le_bytes());
                h.update(&[match t.kind {
                    TransferKind::Fungible => 0,
                    TransferKind::Nft => 1,
                    TransferKind::Document => 2,
                }]);
                if let Some(a) = t.amount {
                    h.update(&a.to_le_bytes());
                }
                if let Some(pref) = &t.payout_preference {
                    h.update(&[match pref {
                        PayoutPreference::ToPrincipal => 1,
                        PayoutPreference::ToCustody => 2,
                        PayoutPreference::CkWithdraw => 3,
                    }]);
                }
                if let Some(tx) = t.tx_index {
                    h.update(&tx.to_le_bytes());
                }
                if let Some(errk) = &t.error_kind {
                    h.update(&[match errk {
                        TransferErrorKind::MissingApproval => 1,
                        TransferErrorKind::AllowanceNotFoundOnChain => 2,
                        TransferErrorKind::InvalidOwnerPrincipal => 3,
                        TransferErrorKind::MissingDestinationPrincipal => 4,
                        TransferErrorKind::NftDip721 => 5,
                        TransferErrorKind::NftExt => 6,
                        TransferErrorKind::NftUnsupported => 7,
                        TransferErrorKind::TransferCallFailed => 8,
                        TransferErrorKind::Other => 9,
                    }]);
                }
                let leaf = h.finalize();
                let mut arr = [0u8; 32];
                arr.copy_from_slice(&leaf);
                leaves.push(arr);
            }
            // Build tree
            fn fold_level(mut nodes: Vec<[u8; 32]>) -> [u8; 32] {
                use sha2::{Digest, Sha256};
                if nodes.is_empty() {
                    return [0u8; 32];
                }
                while nodes.len() > 1 {
                    let mut next: Vec<[u8; 32]> = Vec::with_capacity((nodes.len() + 1) / 2);
                    for chunk in nodes.chunks(2) {
                        if chunk.len() == 1 {
                            next.push(chunk[0]);
                        } else {
                            let mut h = Sha256::new();
                            h.update(&chunk[0]);
                            h.update(&chunk[1]);
                            let d = h.finalize();
                            let mut a = [0u8; 32];
                            a.copy_from_slice(&d);
                            next.push(a);
                        }
                    }
                    nodes = next;
                }
                nodes[0]
            }
            let root = fold_level(leaves);
            let root_vec = root.to_vec();
            u.ledger_attestation = Some(crate::models::payout::LedgerAttestation {
                merkle_root: root_vec.clone(),
                computed_at: crate::time::now_secs(),
                transfer_count: u.transfers.len() as u64,
            });
            push_audit(
                u,
                AuditEventKind::LedgerAttested {
                    merkle_root: root_vec.clone(),
                },
            );
            Ok(root_vec)
        } else {
            Err(CivError::UserNotFound)
        }
    })
}

pub fn reset_timer() -> Result<(), CivError> {
    let user = user_id();
    USERS.with(|users| {
        let mut users = users.borrow_mut();
        let now = now_secs();
        if let Some(user) = users.get_mut(&user) {
            user.timer_expiry = now + INACTIVITY_PERIOD_SECS;
            user.distributed = false;
            user.last_timer_reset = now;
            Ok(())
        } else {
            Err(CivError::UserNotFound)
        }
    })
}

pub fn last_execution_summary() -> Option<ExecutionSummary> {
    let caller = user_id();
    USERS.with(|users| {
        let users = users.borrow();
        users
            .get(&caller)
            .and_then(|u| u.last_execution_summary.clone())
    })
}

// Internal convenience used by maintenance module
pub(crate) fn run_phase_maintenance_once() {
    USERS.with(|users| {
        let mut users = users.borrow_mut();
        for (_k, u) in users.iter_mut() {
            maybe_advance_phase(u);
        }
    });
}

// -------- Session TTL purge & metrics history helpers (invoked by maintenance tick) --------
pub fn purge_expired_sessions_for(principal: &str, max: usize) {
    USERS.with(|users| {
        let mut users = users.borrow_mut();
        if let Some(u) = users.get_mut(principal) {
            let now = now_secs();
            let mut removed: Vec<(u64, u64)> = Vec::new();
            let mut kept = Vec::with_capacity(u.sessions.len());
            for s in u.sessions.iter() {
                if now > s.expires_at {
                    removed.push((s.heir_id, s.id));
                    if removed.len() >= max {
                        continue;
                    }
                } else {
                    kept.push(s.clone());
                }
            }
            u.sessions = kept;
            for (heir_id, session_id) in removed.into_iter() {
                push_audit(
                    u,
                    AuditEventKind::HeirSessionExpired {
                        heir_id,
                        session_id,
                    },
                );
            }
        }
    });
}

pub fn capture_metrics_frame_for(principal: &str, capacity: usize) {
    // Build frame from current metrics snapshot (per-user variant)
    let caller_metrics =
        if let Some(frame) = crate::api::distributions::metrics_snapshot_internal(principal) {
            Some(frame)
        } else {
            None
        };
    if let Some(frame) = caller_metrics {
        USERS.with(|users| {
            let mut users = users.borrow_mut();
            if let Some(u) = users.get_mut(principal) {
                if u.metrics_history.len() >= capacity {
                    let drop = u.metrics_history.len() + 1 - capacity;
                    if drop > 0 {
                        u.metrics_history
                            .drain(0..drop.min(u.metrics_history.len()));
                    }
                }
                u.metrics_history.push(frame);
            }
        });
    }
}

// -------- Escrow auto-management (scan & schedule) --------
pub fn scan_escrow_auto_actions(principal: &str, max: usize) {
    use crate::models::custody::ESCROW_AUTO_ACTION_COOLDOWN_SECS;
    use crate::models::custody::{ESCROW_RECLAIM_MIN_SURPLUS, ESCROW_TOP_UP_MIN_SHORTFALL};
    let mut actions: Vec<(u64, bool /*top_up*/, i128 /*delta*/)> = Vec::new();
    let now = now_secs();
    // Gather candidate actions based on reconciliation deltas
    USERS.with(|users| {
        let users = users.borrow();
        if let Some(u) = users.get(principal) {
            if let Some(recon) = &u.escrow_recon {
                for e in recon.iter() {
                    if actions.len() >= max {
                        break;
                    }
                    if let Some(delta) = e.delta {
                        if delta < 0 {
                            // shortfall needs top-up
                            let short = (-delta) as u128;
                            if short >= ESCROW_TOP_UP_MIN_SHORTFALL as u128 {
                                actions.push((e.asset_id, true, delta));
                            }
                        } else if delta > 0 {
                            // surplus reclaim
                            let surplus = delta as u128;
                            if surplus >= ESCROW_RECLAIM_MIN_SURPLUS {
                                actions.push((e.asset_id, false, delta));
                            }
                        }
                    }
                }
            }
        }
    });
    if actions.is_empty() {
        return;
    }
    // Apply cooldown and enqueue retries + audits
    USERS.with(|users| {
        let mut users = users.borrow_mut();
        if let Some(u) = users.get_mut(principal) {
            // Pre-compute heirs per asset (distribution targets) so we can enqueue EscrowRelease retries with heir context
            let mut heirs_by_asset: std::collections::HashMap<u64, Vec<u64>> =
                std::collections::HashMap::new();
            for d in u.distributions_v2.iter() {
                heirs_by_asset
                    .entry(d.asset_id)
                    .or_default()
                    .push(d.heir_id);
            }
            for (asset_id, top_up, delta) in actions.into_iter() {
                let recent = u.audit_log.iter().rev().take(100).any(|a| match &a.kind {
                    AuditEventKind::EscrowAutoTopUp { asset_id: aid, .. } if *aid == asset_id => {
                        now.saturating_sub(a.timestamp) < ESCROW_AUTO_ACTION_COOLDOWN_SECS
                    }
                    AuditEventKind::EscrowAutoReclaim { asset_id: aid, .. } if *aid == asset_id => {
                        now.saturating_sub(a.timestamp) < ESCROW_AUTO_ACTION_COOLDOWN_SECS
                    }
                    _ => false,
                });
                if recent {
                    continue;
                }
                if top_up {
                    push_audit(
                        u,
                        AuditEventKind::EscrowAutoTopUp {
                            asset_id,
                            amount: (-delta) as u128,
                        },
                    );
                } else {
                    push_audit(
                        u,
                        AuditEventKind::EscrowAutoReclaim {
                            asset_id,
                            amount: delta as u128,
                        },
                    );
                }
                // Enqueue release retries per heir (best-effort) so actual per-heir escrow releases can proceed
                if let Some(heirs) = heirs_by_asset.get(&asset_id) {
                    for heir_id in heirs.iter().take(10) {
                        // cap fan-out for safety
                        crate::api::retry::enqueue_retry(
                            crate::api::retry::RetryKind::EscrowRelease {
                                asset_id,
                                heir_id: *heir_id,
                            },
                            15,
                        );
                    }
                }
            }
        }
    });
}
