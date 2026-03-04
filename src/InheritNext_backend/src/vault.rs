use candid::Principal;
use ic_cdk::call::Call;
use icrc_ledger_types::{
    icrc1::account::Account,
    icrc2::allowance::{Allowance, AllowanceArgs},
};

use crate::{
    helpers::{
        is_vault_released, log_event, now, DEFAULT_GRACE_PERIOD, DEFAULT_HEARTBEAT_INTERVAL,
        NANOS_PER_DAY,
    },
    storage::{self, insert_vault, update_vault, vault_exists},
    types::{DeadManSwitch, EventType, Vault, VaultStatus},
};

pub fn create_new_vault(caller: &Principal) -> Result<(), String> {
    if vault_exists(caller) {
        return Err("Vault Already Exists".to_string());
    }

    let cur_time = now();
    insert_vault(
        caller,
        Vault {
            owner: *caller,
            created_at: cur_time,
            status: VaultStatus::Active,
            dms: DeadManSwitch {
                last_heartbeat: cur_time,
                heartbeat_interval: DEFAULT_HEARTBEAT_INTERVAL,
                grace_period: DEFAULT_GRACE_PERIOD,
                pending_since: None,
            },
            recovery_config: None,
            next_asset_id: 0,
        },
    );

    Ok(())
}

pub fn configure_switch(
    caller: &Principal,
    heartbeat_intervals_d: u32,
    grace_period_d: u32,
) -> Result<(), String> {
    if heartbeat_intervals_d == 0 || grace_period_d == 0 {
        return Err("intervals must be greater thna zero".to_string());
    }

    storage::update_vault(caller, |vault| {
        if vault.status == VaultStatus::Released {
            return Err("Cannot modify released vault".to_string());
        }

        vault.dms.heartbeat_interval = (heartbeat_intervals_d as u64) * NANOS_PER_DAY;
        vault.dms.grace_period = (grace_period_d as u64) * NANOS_PER_DAY;

        Ok(())
    })
}

pub fn send_heartbeat(caller: &Principal) -> Result<(), String> {
    // println!("heartbeat SEND ===========================");
    update_vault(caller, |vault| {
        if is_vault_released(vault) {
            return Err("Vault Already Released".to_string());
        }
        let cur_time = now();
        vault.dms.last_heartbeat = cur_time;
        vault.dms.pending_since = None;
        vault.status = VaultStatus::Active;

        Ok(())
    })
}

pub async fn verify_icrc2_allowance(
    caller: &Principal,
    ledger_canister: &Principal,
    amount: u64,
) -> Result<(), String> {
    let backend_canister = ic_cdk::api::canister_self();
    let allowance_args = AllowanceArgs {
        account: Account {
            owner: *caller,
            subaccount: None,
        },
        spender: Account {
            owner: backend_canister,
            subaccount: None,
        },
    };

    let response = Call::unbounded_wait(*ledger_canister, "icrc2_allowance")
        .with_arg(allowance_args)
        .await;

    let check_result: Result<(Allowance,), _> = match response {
        Ok(resp) => resp.candid_tuple().map_err(|e| {
            (
                ic_cdk::call::RejectCode::CanisterError,
                format!("Failed to decode response: {:?}", e),
            )
        }),
        Err(e) => Err((
            ic_cdk::call::RejectCode::CanisterError,
            format!("Call failed: {:?}", e),
        )),
    };

    match check_result {
        Ok((allowance,)) => {
            let req = candid::Nat::from(amount);
            if allowance.allowance < req {
                return Err(format!(
                    "Insufficient allowance. Required: {}, Current: {}. Please approve the backend canister: dfx canister call {} icrc2_approve '(record{{spender=record{{owner=principal\\\"{}\\\";subaccount=null}};amount={}}})' ",
                    req, allowance.allowance, ledger_canister.to_text(), backend_canister.to_text(), amount
                ));
            }

            log_event(
                EventType::AssetUpdated,
                caller,
                format!(
                    "ICRC-2 allowance verified: {} tokens approved on {}",
                    allowance.allowance,
                    ledger_canister.to_text()
                ),
            );
            Ok(())
        }
        Err(e) => Err(format!(
            "Could not verify allowance from ledger {}. Error: {:?}. Please ensure the ledger supports ICRC-2.",
            ledger_canister.to_text(),
            e
        )),
    }
}
