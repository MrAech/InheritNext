//! Stable memory (upgrade) hooks separated for clarity.

use crate::models::*;
use crate::storage::USERS;
use ic_cdk::storage as stable;

#[derive(candid::CandidType, serde::Deserialize)]
struct StableState(Vec<(String, User)>);

pub fn pre_upgrade() {
    USERS.with(|users| {
        let map = users.borrow();
        let vec: Vec<(String, User)> = map.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
        stable::stable_save((StableState(vec),)).expect("stable save failed");
    });
}

pub fn post_upgrade() {
    if let Ok((StableState(vec),)) = stable::stable_restore::<(StableState,)>() {
        USERS.with(|users| {
            let mut map = users.borrow_mut();
            map.clear();
            for (k, v) in vec {
                let mut user = v;
                // Migration: ensure schema_version present / bumped
                if user.schema_version == 0 {
                    user.schema_version = crate::models::CURRENT_SCHEMA_VERSION;
                }
                if user.schema_version < 2 {
                    if user.custody_recon.is_none() {
                        user.custody_recon = None;
                    }
                    user.schema_version = 2;
                }
                if user.schema_version < 3 {
                    // v3: add chain_wrapped field to Asset, and bridge_error/poll fields to CkWithdrawRecord
                    for a in user.assets.iter_mut() {
                        if a.chain_wrapped.is_none() {
                            // infer from asset_type prefix
                            let lt = a.asset_type.to_ascii_lowercase();
                            a.chain_wrapped = if lt.starts_with("ckbtc") {
                                Some(crate::models::asset::ChainWrappedKind::CkBtc)
                            } else if lt.starts_with("cketh") {
                                Some(crate::models::asset::ChainWrappedKind::CkEth)
                            } else {
                                None
                            };
                        }
                    }
                    for r in user.ck_withdraws.iter_mut() {
                        if r.bridge_error.is_none() {
                            r.bridge_error = None;
                        }
                    }
                    user.schema_version = 3;
                }
                if user.schema_version < 4 {
                    // v4: add tx_hash & effective_fee fields to CkWithdrawRecord
                    for r in user.ck_withdraws.iter_mut() {
                        if r.tx_hash.is_none() {
                            r.tx_hash = None;
                        }
                        if r.effective_fee.is_none() {
                            r.effective_fee = None;
                        }
                    }
                    user.schema_version = 4;
                }
                if user.schema_version < 5 {
                    // v5: structured transfer errors; map legacy string codes to enum
                    for t in user.transfers.iter_mut() {
                        if t.error_kind.is_none() {
                            if let Some(ref code) = t.error {
                                let (kind, _detail) =
                                    crate::models::payout::TransferErrorKind::from_legacy(code);
                                t.error_kind = Some(kind);
                            }
                        }
                    }
                    user.schema_version = 5;
                    // NOTE: After deploying this upgrade, regenerate candid (civ_backend.did) to expose error_kind if needed externally.
                }
                if user.schema_version < 6 {
                    // v6: remove legacy poll_attempts/next_poll_after; nothing to migrate (fields dropped)
                    user.schema_version = 6;
                }
                if user.schema_version < 7 {
                    // v7: add notifications vec (already defaults if absent)
                    if user.notifications.is_empty() { /* nothing else required */ }
                    user.schema_version = 7;
                }
                map.insert(k, user);
            }
        });
    }
    crate::api::executor::schedule_maintenance();
}
