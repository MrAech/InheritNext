use candid::Principal;

use crate::{
    storage,
    types::{AssetType, AuditEvent, EventType, Vault, VaultStatus},
    vault,
};

pub const NANOS_PER_DAY: u64 = 86_400_000_000_000;
pub const DEFAULT_HEARTBEAT_INTERVAL: u64 = 30 * NANOS_PER_DAY;
pub const DEFAULT_GRACE_PERIOD: u64 = 7 * NANOS_PER_DAY;
pub const MAX_NAME_LENGTH: usize = 100;
pub const MAX_AUDIT_EVENT: u64 = 10_000;
pub const MAX_DESCRIPTION_LENGTH: usize = 500;

pub fn now() -> u64 {
    ic_cdk::api::time()
}

pub fn log_event(event_type: EventType, blame: &Principal, details: String) {
    let event = AuditEvent {
        blame: *blame,
        timestamp: now(),
        event_type,
        details,
    };
    storage::log_event(event);
}

pub fn check_is_anonymous(caller: &Principal) -> bool {
    *caller == candid::Principal::anonymous()
}

pub fn is_vault_released(vault: &Vault) -> bool {
    vault.status == VaultStatus::Released
}

pub fn validate_asset_input(name: &str, desc: &str) -> Result<(), String> {
    if name.is_empty() {
        return Err("Asset name cannot be empty".to_string());
    }

    if name.chars().count() > MAX_NAME_LENGTH {
        return Err(format!(
            "Asset name too long (max {} characters)",
            MAX_NAME_LENGTH
        ));
    }

    if desc.chars().count() > MAX_DESCRIPTION_LENGTH {
        return Err(format!(
            "Description too long (max {} characters)",
            MAX_DESCRIPTION_LENGTH
        ));
    }

    Ok(())
}

pub async fn verify_asset_type(caller: &Principal, asset_type: &AssetType) -> Result<(), String> {
    match asset_type {
        AssetType::ICRC2Token {
            ledger_canister,
            amount,
        } => {
            if *amount == 0 {
                return Err("Asset amount must be greater than 0".to_string());
            }
            vault::verify_icrc2_allowance(caller, ledger_canister, *amount).await
        }
    }
}
