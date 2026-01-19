mod helpers;
mod storage;
mod types;
mod vault;

use ic_cdk_macros::{query, update};

use crate::{
    helpers::{log_event, now},
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

    let profile = UserProfile {
        first_name,
        last_name,
        created_at: now(),
    };

    create_user(&caller, profile)
}

#[query]
fn gt_profile() -> Result<UserProfile, String> {
    let caller = ic_cdk::api::msg_caller();

    get_user(&caller).ok_or("User not registered".to_string())
}

#[update]
fn create_vault() -> Result<(), String> {
    let caller = ic_cdk::api::msg_caller();

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

ic_cdk::export_candid!();
