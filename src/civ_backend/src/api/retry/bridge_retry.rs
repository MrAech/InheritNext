use crate::models::*;
use crate::storage::USERS;

pub async fn retry_bridge_submit(
    user_principal: &str,
    asset_id: u64,
    heir_id: u64,
) -> Result<(), String> {
    let (token_canister_opt, has_requested, completed, tx_exists, amount, chain_kind_opt) = USERS
        .with(|users| {
            let users = users.borrow();
            if let Some(u) = users.get(user_principal) {
                let asset = u.assets.iter().find(|a| a.id == asset_id);
                let asset_can = asset.and_then(|a| a.token_canister.clone());
                let chain_kind = asset.and_then(|a| a.chain_wrapped.clone());
                let rec = u
                    .ck_withdraws
                    .iter()
                    .find(|r| r.asset_id == asset_id && r.heir_id == heir_id);
                if let Some(r) = rec {
                    (
                        asset_can,
                        r.requested_at.is_some(),
                        r.completed_at.is_some(),
                        r.bridge_tx_id.is_some(),
                        r.amount,
                        chain_kind,
                    )
                } else {
                    (asset_can, false, false, false, 0u128, chain_kind)
                }
            } else {
                (None, false, false, false, 0u128, None)
            }
        });
    if completed || !has_requested || tx_exists {
        return Ok(());
    }
    let can_txt = token_canister_opt.ok_or_else(|| "missing_token_canister".to_string())?;
    let can_principal =
        candid::Principal::from_text(&can_txt).map_err(|_| "invalid_token_canister".to_string())?;
    // Call new async submit_bridge_withdraw path
    let chain_kind = chain_kind_opt.ok_or_else(|| "missing_chain_kind".to_string())?;
    let res =
        crate::api::bridge::submit_bridge_withdraw(can_principal, amount, "retry-l1", chain_kind)
            .await;
    match res {
        Ok(tx_opt) => {
            USERS.with(|users| {
                let mut users = users.borrow_mut();
                if let Some(u) = users.get_mut(user_principal) {
                    if let Some(rec) = u
                        .ck_withdraws
                        .iter_mut()
                        .find(|r| r.asset_id == asset_id && r.heir_id == heir_id)
                    {
                        rec.bridge_tx_id = tx_opt;
                        rec.bridge_status = Some(BridgeStatus::Submitted);
                    }
                }
            });
            Ok(())
        }
        Err(e) => Err(format!("{:?}", e)),
    }
}
