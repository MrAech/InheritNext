mod helpers;
mod storage;
mod types;
mod vault;

use candid::Principal;
use ic_cdk_macros::{query, update};

use crate::{
    helpers::{
        check_is_anonymous, log_event, now, validate_asset_input, verify_asset_type,
        MAX_NAME_LENGTH,
    },
    storage::{
        create_user, get_asset, get_heirs, get_user, get_vault, insert_asset, insert_heirs,
        is_user_registered, list_user_assets, next_asset_id, remove_asset,
    },
    types::{Asset, AssetType, Heir, UserProfile, Vault, VaultStatus},
};

#[query]
fn is_registered() -> bool {
    let caller = ic_cdk::api::msg_caller();
    is_user_registered(&caller)
}

#[update]
fn register_user(first_name: String, last_name: String) -> Result<(), String> {
    let caller = ic_cdk::api::msg_caller();

    if check_is_anonymous(&caller) {
        return Err("Anonymous user cannot register".to_string());
    }

    if first_name.is_empty() || last_name.is_empty() {
        return Err("Name fields cannot be empty".to_string());
    }

    // Not Doing .len() cause it returns bytes and didnt knew that
    if first_name.chars().count() > MAX_NAME_LENGTH {
        return Err(format!(
            "First name too long (max {} bytes)",
            MAX_NAME_LENGTH
        ));
    }
    if last_name.chars().count() > MAX_NAME_LENGTH {
        return Err(format!(
            "Last name too long (max {} bytes)",
            MAX_NAME_LENGTH
        ));
    }

    let profile = UserProfile {
        first_name,
        last_name,
        created_at: now(),
    };

    create_user(&caller, profile)
}

#[query]
fn get_profile() -> Result<UserProfile, String> {
    let caller = ic_cdk::api::msg_caller();

    get_user(&caller).ok_or("User not registered".to_string())
}

#[update]
fn create_vault() -> Result<(), String> {
    let caller = ic_cdk::api::msg_caller();

    if check_is_anonymous(&caller) {
        return Err("Anonymous User Cannot Create Vault".to_string());
    }

    if !is_user_registered(&caller) {
        return Err("User must first register before Creating Vault".to_string());
    }

    vault::create_new_vault(&caller)?;

    log_event(
        types::EventType::VaultCreated,
        &caller,
        "Vault Created".to_string(),
    );

    Ok(())
}

#[update]
fn configure_dms(hearbeat_interval_d: u32, grace_period_d: u32) -> Result<(), String> {
    let caller = &ic_cdk::api::msg_caller();
    vault::configure_switch(caller, hearbeat_interval_d, grace_period_d)
}

#[query]
fn get_my_vault() -> Result<Vault, String> {
    let caller = &ic_cdk::api::msg_caller();
    get_vault(caller).ok_or("No vault found".to_string())
}

#[update]
fn heartbeat() -> Result<(), String> {
    let caller = ic_cdk::api::msg_caller();

    if check_is_anonymous(&caller) {
        return Err("Anonymous User NOT ALLOWED".to_string());
    }

    vault::send_heartbeat(&caller)?;

    log_event(
        types::EventType::Heartbeat,
        &caller,
        "Heartbeat Received".to_string(),
    );
    Ok(())
}

#[update]
async fn add_asset(
    name: String,
    desc: String,
    asset_type: AssetType,
    heir_assingment: Vec<types::HeirAssignment>,
) -> Result<u64, String> {
    let caller = ic_cdk::api::msg_caller();

    if check_is_anonymous(&caller) {
        return Err("Anonymous principal not allowed".to_string());
    }

    if !is_user_registered(&caller) {
        return Err("User must be registered before adding assets".to_string());
    }
    validate_asset_input(&name, &desc)?;

    verify_asset_type(&caller, &asset_type).await?;

    let vault_ref = get_vault(&caller).ok_or("Vault not found".to_string())?;
    if vault_ref.status == types::VaultStatus::Released {
        return Err("Cannot add assets to released vault".to_string());
    }

    let asset_id = next_asset_id();

    let asset = Asset {
        id: asset_id,
        owner: caller,
        asset_type,
        name: name.clone(),
        description: desc,
        created_at: now(),
        heir_assingment,
    };

    insert_asset(asset);

    log_event(
        types::EventType::AssetCreated,
        &caller,
        format!("Asset created: {}", name),
    );

    Ok(asset_id)
}

#[query]
fn list_my_assets() -> Vec<Asset> {
    let caller = ic_cdk::api::msg_caller();

    list_user_assets(&caller)
}

#[query]
fn get_asset_by_id(asset_id: u64) -> Result<Asset, String> {
    let caller = ic_cdk::api::msg_caller();

    let asset = get_asset(asset_id).ok_or("Asset not found".to_string())?;

    if asset.owner != caller {
        return Err("Not authorized to view this asset".to_string());
    }

    Ok(asset)
}

#[update]
fn remove_asset_by_id(asset_id: u64) -> Result<(), String> {
    let caller = ic_cdk::api::msg_caller();

    let asset = get_asset(asset_id).ok_or("Asset not found".to_string())?;

    if asset.owner != caller {
        return Err("Not authorized to remove this asset".to_string());
    }

    let vault = get_vault(&caller).ok_or("Vault not found".to_string())?;
    if vault.status == types::VaultStatus::Released {
        return Err("Cannot remove assets from released vault".to_string());
    }

    remove_asset(asset_id);

    log_event(
        types::EventType::AssetDeleted,
        &caller,
        format!("Asset deleted: {}", asset.name),
    );

    Ok(())
}

#[update]
fn add_heir(heir_prin: Principal, name: String, alloc_percentage: u8) -> Result<(), String> {
    let caller = ic_cdk::api::msg_caller();

    if check_is_anonymous(&caller) {
        return Err("Anonymous principal not allowed".to_string());
    }

    if !is_user_registered(&caller) {
        return Err("User must be registered before adding heirs".to_string());
    }

    if name.is_empty() {
        return Err("Heir name cannot be empty".to_string());
    }
    if name.len() > MAX_NAME_LENGTH {
        return Err(format!(
            "Heir name too long (max {} characters)",
            MAX_NAME_LENGTH
        ));
    }

    if heir_prin == caller {
        return Err("Cannot add yourself as heir".to_string());
    }

    if alloc_percentage == 0 || alloc_percentage > 100 {
        return Err("Allocation must be between 1 and 100".to_string());
    }

    let vault = get_vault(&caller).ok_or("Vault not found".to_string())?;
    if vault.status == types::VaultStatus::Released {
        return Err("Cannot modify heirs of released vault".to_string());
    }

    let mut heirs = get_heirs(&caller);

    if heirs.iter().any(|h| h.heir_prin == heir_prin) {
        return Err("Heir already exists".to_string());
    }

    let total_alloc: u32 = heirs.iter().map(|h| h.allocation_percentage as u32).sum();
    if total_alloc + (alloc_percentage as u32) > 100 {
        return Err(format!(
            "Total allocation would exceed 100%. Current: {}%, Attempting to add: {}%",
            total_alloc, alloc_percentage
        ));
    }

    heirs.push(Heir {
        heir_prin,
        name: name.clone(),
        allocation_percentage: alloc_percentage,
    });

    insert_heirs(&caller, heirs);

    log_event(
        types::EventType::HeirAdded,
        &caller,
        format!("Heir added: {} ({}%)", name, alloc_percentage),
    );
    Ok(())
}

#[update]
fn remove_heir(heir: Principal) -> Result<(), String> {
    let caller = ic_cdk::api::msg_caller();

    let vault = get_vault(&caller).ok_or("Vault not found".to_string())?;
    if vault.status == types::VaultStatus::Released {
        return Err("Cannot modify heirs of released vault".to_string());
    }

    let mut heirs = get_heirs(&caller);

    let heir_name = heirs
        .iter()
        .find(|h| h.heir_prin == heir)
        .map(|h| h.name.clone())
        .ok_or("Heir Not Found".to_string())?;

    heirs.retain(|h| h.heir_prin != heir);

    insert_heirs(&caller, heirs);

    log_event(
        types::EventType::HeirRemoved,
        &caller,
        format!("Heir removed: {}", heir_name),
    );

    Ok(())
}

#[update]
fn update_heir(
    heir_prin: Principal,
    name: Option<String>,
    alloc_percentage: Option<u8>,
) -> Result<(), String> {
    let caller = ic_cdk::api::msg_caller();

    let vault = get_vault(&caller).ok_or("Vault not found".to_string())?;
    if vault.status == VaultStatus::Released {
        return Err("Cannot modify heirs of released vault".to_string());
    }

    let mut heirs = get_heirs(&caller);

    let heir_idx = heirs
        .iter()
        .position(|h| h.heir_prin == heir_prin)
        .ok_or("Heir Not found".to_string())?;

    if let Some(new_alloc) = alloc_percentage {
        if new_alloc == 0 || new_alloc > 100 {
            return Err("Allocation must be between 1 and 100".to_string());
        }

        let total_others: u32 = heirs
            .iter()
            .enumerate()
            .filter(|(i, _)| *i != heir_idx)
            .map(|(_, h)| h.allocation_percentage as u32)
            .sum();

        if total_others + (new_alloc as u32) > 100 {
            return Err(format!(
                "Total allocation would exceed 100%. Other heirs: {}%, Attempting: {}%",
                total_others, new_alloc
            ));
        }

        heirs[heir_idx].allocation_percentage = new_alloc;
    }

    if let Some(new_name) = name {
        if new_name.is_empty() {
            return Err("Heir name cannot be empty".to_string());
        }
        if new_name.len() > MAX_NAME_LENGTH {
            return Err(format!(
                "Heir name too long (max {} characters)",
                MAX_NAME_LENGTH
            ));
        }
        heirs[heir_idx].name = new_name;
    }

    let updated_heir = &heirs[heir_idx];
    let log_msg = format!(
        "Heir updated: {} ({}%)",
        updated_heir.name, updated_heir.allocation_percentage
    );

    insert_heirs(&caller, heirs);

    log_event(types::EventType::HeirUpdated, &caller, log_msg);

    Ok(())
}

ic_cdk::export_candid!();
