// src/civ_backend/src/heirs.rs
use ic_cdk::export::Principal;
use ic_cdk_macros::{query, update};
use candid::{CandidType, Deserialize};
use serde::Serialize;

use crate::{Heir, Allocation, STATE};

#[update]
pub fn add_heir(heir_principal: Principal, name: Option<String>, gov_id_hash: String) -> Result<(), String> {
    let caller = ic_cdk::caller();
    STATE.with(|s| {
        let mut st = s.borrow_mut();
        if !st.owners.contains_key(&caller) {
            return Err("Caller is not a registered owner".to_string());
        }
        let now_ns = ic_cdk::api::time();
        let now_s = (now_ns / 1_000_000_000) as u64;
        let heirs = st.heirs.entry(caller).or_default();
        if heirs.iter().any(|h| h.principal == heir_principal) {
            return Err("Heir already exists".to_string());
        }
        heirs.push(Heir {
            principal: heir_principal,
            name,
            gov_id_hash,
            added_at: now_s,
        });
        Ok(())
    })
}

#[update]
pub fn remove_heir(heir_principal: Principal) -> Result<(), String> {
    let caller = ic_cdk::caller();
    STATE.with(|s| {
        let mut st = s.borrow_mut();
        if !st.owners.contains_key(&caller) {
            return Err("Caller is not a registered owner".to_string());
        }
        if let Some(heirs) = st.heirs.get_mut(&caller) {
            let orig = heirs.len();
            heirs.retain(|h| h.principal != heir_principal);
            if heirs.len() == orig {
                return Err("Heir not found".to_string());
            }
            // remove any allocations referencing this heir
            if let Some(map) = st.allocations.get_mut(&caller) {
                for (_asset, allocs) in map.iter_mut() {
                    allocs.retain(|a| a.heir != heir_principal);
                }
            }
            Ok(())
        } else {
            Err("No heirs for caller".to_string())
        }
    })
}

#[query]
pub fn list_heirs(owner: Principal) -> Vec<Heir> {
    STATE.with(|s| s.borrow().heirs.get(&owner).cloned().unwrap_or_default())
}

#[update]
pub fn set_allocation(asset: String, heir_principal: Principal, basis_points: u32) -> Result<(), String> {
    if basis_points > 10000 {
        return Err("basis_points must be <= 10000".to_string());
    }
    let caller = ic_cdk::caller();
    STATE.with(|s| {
        let mut st = s.borrow_mut();
        if !st.owners.contains_key(&caller) {
            return Err("Caller is not a registered owner".to_string());
        }
        let heirs = st.heirs.get(&caller).cloned().unwrap_or_default();
        if !heirs.iter().any(|h| h.principal == heir_principal) {
            return Err("Heir not registered for caller".to_string());
        }
        let asset_map = st.allocations.entry(caller).or_default();
        let allocs = asset_map.entry(asset.clone()).or_default();
        // replace or insert
        if let Some(existing) = allocs.iter_mut().find(|a| a.heir == heir_principal && a.asset == asset) {
            existing.basis_points = basis_points;
        } else {
            allocs.push(Allocation { asset: asset.clone(), heir: heir_principal, basis_points });
        }
        // validate total for this asset
        let total: u32 = allocs.iter().map(|a| a.basis_points).sum();
        if total > 10000 {
            // revert change: remove the entry we just added/modified
            allocs.retain(|a| !(a.heir == heir_principal && a.asset == asset && a.basis_points == basis_points));
            return Err("Total allocation for asset exceeds 10000 basis points".to_string());
        }
        Ok(())
    })
}

#[query]
pub fn get_allocations(owner: Principal, asset: String) -> Vec<Allocation> {
    STATE.with(|s| {
        s.borrow()
            .allocations
            .get(&owner)
            .and_then(|m| m.get(&asset).cloned())
            .unwrap_or_default()
    })
}