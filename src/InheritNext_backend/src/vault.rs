use candid::Principal;

use crate::{
    helpers::{now, DEFAULT_GRACE_PERIOD, DEFAULT_HEARTBEAT_INTERVAL, NANOS_PER_DAY},
    storage::{self, insert_vault, vault_exists},
    types::{DeadManSwitch, Vault, VaultStatus},
};

pub fn create_new_vault(caller: &Principal) -> Result<(), String> {
    if vault_exists(caller) {
        return Err("Vault Already Exists".to_string());
    }

    let cur_time = now();
    insert_vault(
        caller,
        Vault {
            owner: caller.clone(),
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
