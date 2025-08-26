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
            // Backend authoritative: do NOT trust frontend-provided `value`.
            // Store 0 as sentinel (unknown/current value will be populated
            // by trusted metadata updates / fetches).
            value: 0u64,
            // Backend authoritative: do NOT trust frontend-provided `decimals`.
            // Always store 0 (sentinel == unknown/not-yet-fetched). Decimals will be
            // populated asynchronously when token metadata is supplied via
            // `update_asset_token_meta` (which fetches icrc metadata / icrc1_decimals).
            decimals: 0u8,
            description: new_asset.description,
            created_at: now,
            updated_at: now,
            token_canister: new_asset.token_canister,
            token_id: new_asset.token_id,
            holding_mode: new_asset.holding_mode,
            nft_standard: new_asset.nft_standard,
            chain_wrapped: new_asset.chain_wrapped,
            file_path: new_asset.file_path,
        });
        // If we have a token canister, schedule an async metadata fetch to populate decimals/value
        let maybe_canister = new_asset.token_canister.clone();
        let created_asset_id = next_id;
        let caller = user.clone();
        if maybe_canister.is_some() {
            if let Some(can_txt) = maybe_canister {
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
                                                                .find(|asst| asst.id == created_asset_id)
                                                            {
                                                                if a2.decimals == 0 {
                                                                    a2.decimals = dec;
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
                                                if let Some(a2) = u2.assets.iter_mut().find(|asst| asst.id == created_asset_id) {
                                                    if a2.decimals == 0 {
                                                        a2.decimals = dec;
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
        }
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
                // Update only user-editable fields. Decimals and value are server-managed
                // and must not be supplied by the frontend via AssetInput.
                existing.name = new_asset.name;
                existing.asset_type = new_asset.asset_type;
                existing.description = new_asset.description;
                // Apply optional token/linking fields if the frontend provided them.
                if new_asset.token_canister.is_some() {
                    existing.token_canister = new_asset.token_canister.clone();
                }
                if new_asset.token_id.is_some() {
                    existing.token_id = new_asset.token_id;
                }
                if new_asset.holding_mode.is_some() {
                    existing.holding_mode = new_asset.holding_mode;
                }
                if new_asset.nft_standard.is_some() {
                    existing.nft_standard = new_asset.nft_standard.clone();
                }
                if new_asset.chain_wrapped.is_some() {
                    existing.chain_wrapped = new_asset.chain_wrapped;
                }
                if new_asset.file_path.is_some() {
                    // store the file path for document assets (placeholder)
                    // Note: Asset struct does not currently have file_path; if needed,
                    // store elsewhere or extend Asset model. We'll preserve in metadata
                    // via token_canister/file_path pattern or adjust model later.
                }
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
                // meta.decimals may be provided by a trusted metadata update path.
                if let Some(d) = meta.decimals {
                    // apply explicit decimals when provided (trusted path)
                    a.decimals = d;
                }
                if meta.nft_standard.is_some() {
                    a.nft_standard = meta.nft_standard;
                }
                if meta.chain_wrapped.is_some() {
                    a.chain_wrapped = meta.chain_wrapped;
                }
                // if decimals remain unknown (0) and we have a token canister, schedule fetch
                let needs_fetch = a.decimals == 0 && a.token_canister.is_some();
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
                                                        if a2.decimals == 0 {
                                                                        a2.decimals = dec;
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
                                            if a2.decimals == 0 {
                                                a2.decimals = dec;
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
