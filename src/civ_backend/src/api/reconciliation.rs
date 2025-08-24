use crate::api::common::user_id;
use crate::api::executor::work::icrc1_balance_of; // reuse internal helper
use crate::audit::push_audit;
use crate::models::custody::{CustodyReconEntry, EscrowReconEntry, ReconStatus};
use crate::storage::USERS;
use crate::time::now_secs;
use candid::Principal; // use existing helper for caller

// Reconciliation interval heuristic: don't refresh more often than every 6 hours per user unless missing.
const RECON_INTERVAL_SECS: u64 = 6 * 3600;

// Internal helper performing reconciliation for a specific user principal.
#[allow(dead_code)] // Admin dashboard (pending) – detailed custody reconciliation per user
pub async fn reconcile_custody_for(principal: &str) -> Vec<CustodyReconEntry> {
    // snapshot data first to minimize lock time
    let snapshot = USERS.with(|users| {
        let users = users.borrow();
        if let Some(u) = users.get(principal) {
            // if user exists
            let mut rows: Vec<(u64, u64, Option<String>, Vec<u8>, u128)> = Vec::new();
            for f in &u.fungible_custody {
                if f.released_at.is_none() {
                    // only unreleased amounts

                    // find subaccount for heir
                    if let Some(cust) = u.custody.iter().find(|c| c.heir_id == f.heir_id) {
                        let asset = u.assets.iter().find(|a| a.id == f.asset_id);
                        let can_txt = asset.and_then(|a| a.token_canister.clone());
                        rows.push((
                            f.asset_id,
                            f.heir_id,
                            can_txt,
                            cust.subaccount.clone(),
                            f.amount,
                        ));
                    }
                }
            }
            rows
        } else {
            Vec::new()
        }
    });

    let mut recon: Vec<CustodyReconEntry> = Vec::new();
    if snapshot.is_empty() {
        // clear any existing reconciliation if no custody staged
        USERS.with(|users| {
            let mut users = users.borrow_mut();
            if let Some(u) = users.get_mut(principal) {
                u.custody_recon = None;
            }
        });
        return recon;
    }
    // aggregate staged sums per (asset, heir, canister, subaccount)
    use std::collections::HashMap;
    let mut sums: HashMap<(u64, u64, Option<String>, Vec<u8>), u128> = HashMap::new();
    for (asset_id, heir_id, can_txt, sub, amt) in snapshot.into_iter() {
        *sums
            .entry((asset_id, heir_id, can_txt.clone(), sub.clone()))
            .or_insert(0) += amt;
    }
    for ((asset_id, heir_id, can_txt, sub), staged_sum) in sums.into_iter() {
        let now = now_secs();
        if let Some(can_str) = can_txt.clone() {
            if let Ok(token_can) = Principal::from_text(&can_str) {
                let balance_res =
                    icrc1_balance_of(token_can, ic_cdk::api::canister_self(), Some(sub.clone()))
                        .await;
                match balance_res {
                    Ok(on_chain) => {
                        let delta = on_chain as i128 - staged_sum as i128;
                        let status = if delta == 0 {
                            ReconStatus::Exact
                        } else if delta < 0 {
                            ReconStatus::Shortfall
                        } else {
                            ReconStatus::Surplus
                        };
                        recon.push(CustodyReconEntry {
                            asset_id,
                            heir_id,
                            on_chain: Some(on_chain),
                            staged_sum,
                            delta: Some(delta),
                            status,
                            last_checked: now,
                        });
                        if !matches!(status, ReconStatus::Exact) {
                            // emit audit discrepancy for visibility
                            USERS.with(|users| { let mut users = users.borrow_mut(); if let Some(u)=users.get_mut(principal) { let note = if delta < 0 { format!("shortfall:{}", delta) } else { format!("surplus:+{}", delta) }; push_audit(u, crate::models::audit::AuditEventKind::CustodyReconciliationDiscrepancy { heir_id, expected_total: staged_sum, note }); } });
                        }
                    }
                    Err(err) => {
                        recon.push(CustodyReconEntry {
                            asset_id,
                            heir_id,
                            on_chain: None,
                            staged_sum,
                            delta: None,
                            status: ReconStatus::QueryError,
                            last_checked: now,
                        });
                        ic_cdk::println!(
                            "recon query error asset={} heir={} err={}",
                            asset_id,
                            heir_id,
                            err
                        );
                    }
                }
            } else {
                recon.push(CustodyReconEntry {
                    asset_id,
                    heir_id,
                    on_chain: None,
                    staged_sum,
                    delta: None,
                    status: ReconStatus::QueryError,
                    last_checked: now,
                });
            }
        } else {
            recon.push(CustodyReconEntry {
                asset_id,
                heir_id,
                on_chain: None,
                staged_sum,
                delta: None,
                status: ReconStatus::QueryError,
                last_checked: now,
            });
        }
    }
    // persist
    USERS.with(|users| {
        let mut users = users.borrow_mut();
        if let Some(u) = users.get_mut(principal) {
            u.custody_recon = Some(recon.clone());
        }
    });
    recon
}

// Public API (caller) variant retained for backward compatibility.
#[allow(dead_code)] // Admin dashboard (pending) – caller convenience wrapper
pub async fn reconcile_custody() -> Vec<CustodyReconEntry> {
    let caller = user_id();
    reconcile_custody_for(&caller).await
}

pub fn get_custody_reconciliation() -> Option<Vec<CustodyReconEntry>> {
    let caller = user_id();
    USERS.with(|users| {
        let users = users.borrow();
        users.get(&caller).and_then(|u| u.custody_recon.clone())
    })
}

// Determine which users require reconciliation (used by maintenance timer).
#[allow(dead_code)] // Admin dashboard (pending) – scheduler helper to list stale users
pub fn users_needing_recon(now: u64) -> Vec<String> {
    USERS.with(|users| {
        let users = users.borrow();
        users
            .iter()
            .filter_map(|(k, u)| {
                if u.fungible_custody.iter().any(|c| c.released_at.is_none()) {
                    let stale = u
                        .custody_recon
                        .as_ref()
                        .map(|vec| vec.iter().map(|e| e.last_checked).min().unwrap_or(0))
                        .unwrap_or(0);
                    if stale == 0 || now.saturating_sub(stale) >= RECON_INTERVAL_SECS {
                        return Some(k.clone());
                    }
                }
                None
            })
            .collect()
    })
}

// -------- Escrow Reconciliation (skeleton) --------
// Compares logical remaining escrow amount per asset to on-chain escrow subaccount balance.
// Emits discrepancy audits using existing CustodyReconciliationDiscrepancy kind (re-using structure) with note prefix 'escrow:'.
#[allow(dead_code)] // Admin dashboard (pending) – escrow reconciliation prototype
pub async fn reconcile_escrow_for(principal: &str) {
    // Snapshot escrow assets with token canisters and logical remaining amounts
    let snapshot: Vec<(u64, String, u128, Vec<u8>)> = USERS.with(|users| {
        let users = users.borrow();
        if let Some(u) = users.get(principal) {
            u.escrow
                .iter()
                .filter_map(|e| {
                    let logical = e.amount.unwrap_or(0);
                    if logical == 0 {
                        return None;
                    }
                    let asset = u.assets.iter().find(|a| a.id == e.asset_id)?;
                    let can = asset.token_canister.clone()?;
                    let sub = crate::crypto::derive_escrow_subaccount(principal, e.asset_id);
                    Some((e.asset_id, can, logical, sub.to_vec()))
                })
                .collect()
        } else {
            Vec::new()
        }
    });
    if snapshot.is_empty() {
        return;
    }
    let mut recon: Vec<EscrowReconEntry> = Vec::new();
    for (asset_id, can_txt, logical_remaining, sub) in snapshot.into_iter() {
        if let Ok(can_principal) = Principal::from_text(&can_txt) {
            let bal_res = icrc1_balance_of(
                can_principal,
                ic_cdk::api::canister_self(),
                Some(sub.clone()),
            )
            .await;
            let now = now_secs();
            match bal_res {
                Ok(on_chain) => {
                    let delta = on_chain as i128 - logical_remaining as i128;
                    let status = if delta == 0 {
                        ReconStatus::Exact
                    } else if delta < 0 {
                        ReconStatus::Shortfall
                    } else {
                        ReconStatus::Surplus
                    };
                    if delta != 0 {
                        USERS.with(|users| { let mut users=users.borrow_mut(); if let Some(u)=users.get_mut(principal) { let note = if delta<0 { format!("shortfall:{}", delta) } else { format!("surplus:+{}", delta) }; push_audit(u, crate::models::audit::AuditEventKind::EscrowReconciliationDiscrepancy { asset_id, expected_total: logical_remaining, note }); }});
                    }
                    recon.push(EscrowReconEntry {
                        asset_id,
                        on_chain: Some(on_chain),
                        logical_remaining,
                        delta: Some(delta),
                        status,
                        last_checked: now,
                    });
                }
                Err(e) => {
                    ic_cdk::println!("escrow_recon query error asset={} err={}", asset_id, e);
                    recon.push(EscrowReconEntry {
                        asset_id,
                        on_chain: None,
                        logical_remaining,
                        delta: None,
                        status: ReconStatus::QueryError,
                        last_checked: now,
                    });
                }
            }
        }
    }
    // persist on user
    USERS.with(|users| {
        let mut users = users.borrow_mut();
        if let Some(u) = users.get_mut(principal) {
            u.escrow_recon = Some(recon);
        }
    });
}
