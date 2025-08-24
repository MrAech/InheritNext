// Custody related APIs extracted from monolithic api.rs
use crate::api::common::user_id;
use crate::audit::push_audit;
use crate::models::*;
use crate::storage::USERS;
use crate::time::now_secs;
use candid::Principal; // Needed for async transfer spawning logic

// Derive (or return existing) custody subaccount for an heir.
pub fn custody_subaccount_for_heir(heir_id: u64) -> Result<Vec<u8>, CivError> {
    let caller = user_id();
    USERS.with(|users| {
        let mut users = users.borrow_mut();
        if let Some(u) = users.get_mut(&caller) {
            if let Some(existing) = u.custody.iter().find(|c| c.heir_id == heir_id) {
                return Ok(existing.subaccount.clone());
            }
            let sub = crate::crypto::derive_custody_subaccount(&u.user, heir_id);
            u.custody.push(CustodyRecord {
                heir_id,
                subaccount: sub.to_vec(),
            });
            Ok(sub.to_vec())
        } else {
            Err(CivError::UserNotFound)
        }
    })
}

// Withdraw (simulate release) of fungible amount for an heir from custody.
pub fn withdraw_custody(asset_id: u64, heir_id: u64) -> Result<TransferRecord, CivError> {
    // For now: synchronous wrapper that spawns async transfer path would be ideal; simplified synchronous attempt using blocking assumptions.
    let caller = user_id();
    // First phase: gather data & mutate user minimally, returning snapshot needed for async transfer.
    let (rec, heir_principal_txt, token_canister_txt, amount, next_id, from_sub_opt) =
        USERS.with(|users| {
            let mut users = users.borrow_mut();
            if let Some(u) = users.get_mut(&caller) {
                if !matches!(u.phase, EstatePhase::Locked | EstatePhase::Executed) {
                    return Err(CivError::EstateNotReady);
                }
                // Heir & principal
                let (heir_principal_txt, heir_verified) = {
                    let h = u
                        .heirs_v2
                        .iter()
                        .find(|h| h.id == heir_id)
                        .ok_or(CivError::HeirNotFound)?;
                    (
                        h.principal
                            .clone()
                            .ok_or(CivError::Other("heir_principal_missing".into()))?,
                        h.identity_secret.status == HeirSecretStatus::Verified,
                    )
                };
                if !heir_verified {
                    return Err(CivError::SecretInvalid);
                }
                // Asset snapshot (value + token canister)
                let (asset_value, token_canister_txt) = {
                    let a = u
                        .assets
                        .iter()
                        .find(|a| a.id == asset_id)
                        .ok_or(CivError::AssetNotFound)?;
                    (a.value, a.token_canister.clone())
                };
                // Distribution share (percentage + preference)
                let (percentage, payout_pref) = {
                    let d = u
                        .distributions_v2
                        .iter()
                        .find(|d| d.asset_id == asset_id && d.heir_id == heir_id)
                        .ok_or(CivError::DistributionHeirNotFound)?;
                    (d.percentage, d.payout_preference.clone())
                };
                // Ensure custody subaccount exists (now used as source for transfer)
                let from_subaccount: Vec<u8> =
                    if let Some(c) = u.custody.iter().find(|c| c.heir_id == heir_id) {
                        c.subaccount.clone()
                    } else {
                        let sub = crate::crypto::derive_custody_subaccount(&u.user, heir_id);
                        let v = sub.to_vec();
                        u.custody.push(CustodyRecord {
                            heir_id,
                            subaccount: v.clone(),
                        });
                        v
                    };
                let amount: u128 = (asset_value as u128)
                    .saturating_mul(percentage as u128)
                    .checked_div(100)
                    .unwrap_or(0);
                let next_id = u.transfers.iter().map(|t| t.id).max().unwrap_or(0) + 1;
                let rec = TransferRecord {
                    id: next_id,
                    timestamp: now_secs(),
                    asset_id: Some(asset_id),
                    heir_id: Some(heir_id),
                    kind: TransferKind::Fungible,
                    amount: Some(amount),
                    payout_preference: Some(payout_pref),
                    note: Some("custody_withdraw_pending_transfer".into()),
                    tx_index: None,
                    error: None,
                    error_kind: None,
                };
                u.transfers.push(rec.clone());
                push_audit(
                    u,
                    AuditEventKind::CustodyWithdrawExecuted { asset_id, heir_id },
                );
                Ok((
                    rec,
                    heir_principal_txt,
                    token_canister_txt,
                    amount,
                    next_id,
                    Some(from_subaccount),
                ))
            } else {
                Err(CivError::UserNotFound)
            }
        })?;
    // Second phase: spawn async transfer using snapshot (no outstanding borrows)
    let heir_principal_txt_clone = heir_principal_txt.clone();
    ic_cdk::futures::spawn_017_compat(async move {
        if let (Some(can_txt), Ok(heir_p)) = (
            token_canister_txt,
            Principal::from_text(heir_principal_txt_clone),
        ) {
            if let Ok(can_p) = Principal::from_text(can_txt) {
                let (tx, err) = if let Some(from_sub) = from_sub_opt.clone() {
                    crate::api::executor::icrc1_transfer_from_sub(
                        can_p, from_sub, heir_p, None, amount,
                    )
                    .await
                } else {
                    crate::api::executor::icrc1_transfer(can_p, heir_p, None, amount).await
                };
                USERS.with(|users| {
                    let mut users = users.borrow_mut();
                    if let Some(u2) = users.get_mut(&caller) {
                        if let Some(r) = u2.transfers.iter_mut().find(|t| t.id == next_id) {
                            r.tx_index = tx;
                            if let Some(e) = err {
                                r.error = Some(e.clone());
                                let (k, _d) =
                                    crate::models::payout::TransferErrorKind::from_legacy(&e);
                                r.error_kind = Some(k);
                                r.note = Some("custody_withdraw_transfer_failed".into());
                            } else {
                                r.note = Some("custody_withdraw_transferred".into());
                                // Mark fungible custody record released if present (real release)
                                if let Some(fc) = u2.fungible_custody.iter_mut().find(|c| {
                                    c.asset_id == asset_id
                                        && c.heir_id == heir_id
                                        && c.released_at.is_none()
                                }) {
                                    fc.released_at = Some(now_secs());
                                }
                            }
                        }
                    }
                });
            }
        }
    });
    Ok(rec)
}

// (Legacy placeholder reconciliation removed; real reconciliation lives in reconciliation.rs)
