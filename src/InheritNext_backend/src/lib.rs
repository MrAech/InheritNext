mod helpers;
mod storage;
mod types;
mod vault;

use ic_cdk_macros::{query, update};

use crate::{
    helpers::{check_is_anonymous, log_event, now, MAX_NAME_LENGTH},
    storage::{create_user, get_user, get_vault, is_user_registered},
    types::{UserProfile, Vault},
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

ic_cdk::export_candid!();
