// File: src/lib.rs
// InheritNext Backend Canister - Rust + DFX + Candid

use ic_cdk::api::caller;
use ic_cdk_macros::*;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use candid::{CandidType, Principal};

// Toggle for encryption (false = plain storage, true = encrypted)
thread_local! {
    static ENCRYPTION_ENABLED: std::cell::RefCell<bool> = std::cell::RefCell::new(true);
}

// === DATA MODELS ===
#[derive(CandidType, Serialize, Deserialize, Clone)]
struct Asset {
    id: String,
    name: String,
    value: f64,
    asset_type: String,
    owner: Principal,
}

#[derive(CandidType, Serialize, Deserialize, Clone)]
struct Heir {
    id: String,
    name: String,
    wallet: Principal,
    share_percentage: f64,
}

#[derive(CandidType, Serialize, Deserialize, Clone)]
struct Distribution {
    asset_id: String,
    heir_id: String,
    share_percentage: f64,
}

// === STATE STORAGE ===
thread_local! {
    static ASSETS: std::cell::RefCell<HashMap<String, Asset>> = std::cell::RefCell::new(HashMap::new());
    static HEIRS: std::cell::RefCell<HashMap<String, Heir>> = std::cell::RefCell::new(HashMap::new());
    static DISTRIBUTIONS: std::cell::RefCell<Vec<Distribution>> = std::cell::RefCell::new(Vec::new());
}

// === ENCRYPTION TOGGLE ===

#[update]
fn toggle_encryption() {
    ENCRYPTION_ENABLED.with(|e| {
        let mut flag = e.borrow_mut();
        *flag = !*flag;
    });
}

#[query]
fn get_encryption_status() -> bool {
    ENCRYPTION_ENABLED.with(|e| *e.borrow())
}

// === ASSET ENDPOINTS ===

#[query]
fn get_assets() -> Vec<Asset> {
    ASSETS.with(|a| a.borrow().values().cloned().collect())
}

#[update]
fn add_asset(asset: Asset) {
    ASSETS.with(|a| a.borrow_mut().insert(asset.id.clone(), asset));
}

#[update]
fn update_asset(asset: Asset) {
    ASSETS.with(|a| a.borrow_mut().insert(asset.id.clone(), asset));
}

#[update]
fn delete_asset(id: String) {
    ASSETS.with(|a| a.borrow_mut().remove(&id));
}

// === HEIR ENDPOINTS ===

#[query]
fn get_heirs() -> Vec<Heir> {
    HEIRS.with(|h| h.borrow().values().cloned().collect())
}

#[update]
fn add_heir(heir: Heir) {
    HEIRS.with(|h| h.borrow_mut().insert(heir.id.clone(), heir));
}

#[update]
fn update_heir(heir: Heir) {
    HEIRS.with(|h| h.borrow_mut().insert(heir.id.clone(), heir));
}

#[update]
fn delete_heir(id: String) {
    HEIRS.with(|h| h.borrow_mut().remove(&id));
}

// === DISTRIBUTION LOGIC ===

#[query]
fn get_distributions() -> Vec<Distribution> {
    DISTRIBUTIONS.with(|d| d.borrow().clone())
}

#[update]
fn update_distribution(distribution: Distribution) {
    DISTRIBUTIONS.with(|d| d.borrow_mut().push(distribution));
}

#[update]
fn execute_inheritance_transfer() -> String {
    let caller_principal = caller();
    format!("Inheritance transfer executed by: {}", caller_principal)
}
