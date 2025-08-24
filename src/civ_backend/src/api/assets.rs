use crate::api::common::{assert_mutable, user_id};
use crate::audit::push_audit;
use crate::models::{Asset, AssetInput, AssetTokenMetaInput, AuditEventKind, CivError, User};
use crate::storage::USERS;
use crate::time::now_secs;

// Asset CRUD operations.
// Note: Timer now starts on first distribution, not asset add.
pub fn add_asset(new_asset: AssetInput) -> Result<(), CivError> {
    let user = user_id();
    USERS.with(|users| {
        let mut users = users.borrow_mut();
        let now = now_secs();
        let user = users.entry(user.clone()).or_insert_with(|| {
            let mut u = User::new(&user, 0);
            push_audit(&mut u, AuditEventKind::UserCreated);
            u
        });
        assert_mutable(user)?;
        let next_id = user.assets.iter().map(|a| a.id).max().unwrap_or(0) + 1;
        user.assets.push(Asset {
            id: next_id,
            name: new_asset.name,
            asset_type: new_asset.asset_type,
            value: new_asset.value,
            decimals: new_asset.decimals,
            description: new_asset.description,
            created_at: now,
            updated_at: now,
            token_canister: None,
            token_id: None,
            holding_mode: None,
            nft_standard: None,
            chain_wrapped: None,
        });
        push_audit(user, AuditEventKind::AssetAdded { asset_id: next_id });
        Ok(())
    })
}

pub fn list_assets() -> Vec<Asset> {
    let user = user_id();
    USERS.with(|users| {
        let users = users.borrow();
        users
            .get(&user)
            .map(|u| u.assets.clone())
            .unwrap_or_default()
    })
}

pub fn remove_asset(asset_id: u64) -> Result<(), CivError> {
    let user = user_id();
    USERS.with(|users| {
        let mut users = users.borrow_mut();
        if let Some(user) = users.get_mut(&user) {
            assert_mutable(user)?;
            let existed = user.assets.iter().any(|a| a.id == asset_id);
            if !existed {
                return Err(CivError::AssetNotFound);
            }
            user.assets.retain(|a| a.id != asset_id);
            user.distributions.retain(|d| d.asset_id != asset_id);
            user.distributions_v2.retain(|d| d.asset_id != asset_id);
            push_audit(user, AuditEventKind::AssetRemoved { asset_id });
            Ok(())
        } else {
            Err(CivError::UserNotFound)
        }
    })
}

pub fn update_asset(asset_id: u64, new_asset: AssetInput) -> Result<(), CivError> {
    let user = user_id();
    USERS.with(|users| {
        let mut users = users.borrow_mut();
        if let Some(user) = users.get_mut(&user) {
            assert_mutable(user)?;
            if let Some(existing) = user.assets.iter_mut().find(|a| a.id == asset_id) {
                existing.name = new_asset.name;
                existing.asset_type = new_asset.asset_type;
                existing.value = new_asset.value;
                existing.decimals = new_asset.decimals;
                existing.description = new_asset.description;
                existing.updated_at = now_secs();
                push_audit(user, AuditEventKind::AssetUpdated { asset_id });
                Ok(())
            } else {
                Err(CivError::AssetNotFound)
            }
        } else {
            Err(CivError::UserNotFound)
        }
    })
}

pub fn update_asset_token_meta(asset_id: u64, meta: AssetTokenMetaInput) -> Result<(), CivError> {
    let caller = user_id();
    // Phase 1: apply updates & capture fetch target
    let fetch_canister_opt = USERS.with(|users| {
        let mut users = users.borrow_mut();
        if let Some(u) = users.get_mut(&caller) {
            if let Err(e) = assert_mutable(u) {
                return Err(e);
            }
            if let Some(a) = u.assets.iter_mut().find(|a| a.id == asset_id) {
                a.token_canister = meta.token_canister.clone();
                a.token_id = meta.token_id;
                a.holding_mode = meta.holding_mode;
                if meta.decimals.is_some() {
                    a.decimals = meta.decimals;
                }
                if meta.nft_standard.is_some() {
                    a.nft_standard = meta.nft_standard;
                }
                if meta.chain_wrapped.is_some() {
                    a.chain_wrapped = meta.chain_wrapped;
                }
                let needs_fetch = a.decimals.is_none() && a.token_canister.is_some();
                let can_txt = if needs_fetch {
                    a.token_canister.clone()
                } else {
                    None
                };
                a.updated_at = now_secs();
                push_audit(u, AuditEventKind::AssetMetadataUpdated { asset_id });
                Ok(can_txt)
            } else {
                Err(CivError::AssetNotFound)
            }
        } else {
            Err(CivError::UserNotFound)
        }
    })?;
    // Phase 2: async metadata fetch without holding mutable borrow
    if let Some(Some(can_txt)) = fetch_canister_opt.map(|c| Some(c)) {
        // flatten Option<Option<String>>
        ic_cdk::futures::spawn_017_compat(async move {
            if let Ok(p) = candid::Principal::from_text(&can_txt) {
                use candid::types::value::IDLValue;
                use ic_cdk::call::Call;
                // First attempt: metadata list
                let mut have_decimals = false;
                if let Ok(call) = std::panic::catch_unwind(|| {
                    Call::unbounded_wait(p, "icrc1_metadata").with_arg(())
                }) {
                    if let Ok(res) = call
                        .await
                        .map_err(|e| format!("call failed: {:?}", e))
                        .and_then(|r| {
                            r.candid_tuple::<(Vec<(String, IDLValue)>,)>()
                                .map_err(|e| format!("decode err:{:?}", e))
                        })
                    {
                        let (list,) = res;
                        for (k, v) in list.into_iter() {
                            if k == "icrc1:decimals" {
                                if let IDLValue::Nat(n) = v {
                                    if let Ok(parsed) = n.0.to_string().parse::<u64>() {
                                        if parsed <= 38 {
                                            let dec = parsed as u8;
                                            USERS.with(|users| {
                                                let mut users = users.borrow_mut();
                                                if let Some(u2) = users.get_mut(&caller) {
                                                    if let Some(a2) = u2
                                                        .assets
                                                        .iter_mut()
                                                        .find(|asst| asst.id == asset_id)
                                                    {
                                                        if a2.decimals.is_none() {
                                                            a2.decimals = Some(dec);
                                                            a2.updated_at = now_secs();
                                                        }
                                                    }
                                                }
                                            });
                                            have_decimals = true;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                // Fallback: dedicated icrc1_decimals call if still missing
                if !have_decimals {
                    if let Ok(call) = std::panic::catch_unwind(|| {
                        Call::unbounded_wait(p, "icrc1_decimals").with_arg(())
                    }) {
                        if let Ok(reply) = call.await {
                            if let Ok(tuple) = reply.candid_tuple::<(u8,)>() {
                                let dec = tuple.0;
                                USERS.with(|users| {
                                    let mut users = users.borrow_mut();
                                    if let Some(u2) = users.get_mut(&caller) {
                                        if let Some(a2) =
                                            u2.assets.iter_mut().find(|asst| asst.id == asset_id)
                                        {
                                            if a2.decimals.is_none() {
                                                a2.decimals = Some(dec);
                                                a2.updated_at = now_secs();
                                            }
                                        }
                                    }
                                });
                            }
                        }
                    }
                }
            }
        });
    }
    Ok(())
}
