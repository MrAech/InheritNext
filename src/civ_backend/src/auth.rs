// // src/civ_backend/src/auth.rs
// use ic_cdk::export::Principal;
// use ic_cdk_macros::{query, update};
// use candid::{CandidType, Deserialize};
// use serde::Serialize;
// use std::collections::BTreeMap;

// use crate::{OwnerProfile, STATE};

// #[update]
// pub fn register_owner(name: String, gov_id_hash: String) -> Result<(), String> {
//     let caller = ic_cdk::caller();
//     STATE.with(|s| {
//         let mut st = s.borrow_mut();
//         if st.owners.contains_key(&caller) {
//             return Err("Owner already registered".to_string());
//         }
//         let profile = OwnerProfile::new(caller, name, gov_id_hash);
//         st.owners.insert(caller, profile);
//         Ok(())
//     })
// }

// #[update]
// pub fn set_last_active() -> Result<(), String> {
//     let caller = ic_cdk::caller();
//     let now_ns = ic_cdk::api::time();
//     let now_s = (now_ns / 1_000_000_000) as u64;
//     STATE.with(|s| {
//         let mut st = s.borrow_mut();
//         match st.owners.get_mut(&caller) {
//             Some(p) => {
//                 p.last_active = Some(now_s);
//                 Ok(())
//             }
//             None => Err("Owner not registered".to_string()),
//         }
//     })
// }

// #[query]
// pub fn get_owner_profile(owner: Principal) -> Option<OwnerProfile> {
//     STATE.with(|s| s.borrow().owners.get(&owner).cloned())
// }

// #[update]
// pub fn set_thresholds(inactivity_days: Option<u64>, warning_days: Option<u64>) -> Result<(), String> {
//     let caller = ic_cdk::caller();
//     STATE.with(|s| {
//         let mut st = s.borrow_mut();
//         match st.owners.get_mut(&caller) {
//             Some(p) => {
//                 if let Some(i) = inactivity_days { p.inactivity_days = i; }
//                 if let Some(w) = warning_days { p.warning_days = w; }
//                 Ok(())
//             }
//             None => Err("Owner not registered".to_string()),
//         }
//     })
// }

// #[update]
// pub fn update_owner_details(name: String, gov_id_hash: String, inactivity_days: Option<u64>, warning_days: Option<u64>) -> Result<(), String> {
//     let caller = ic_cdk::caller();
//     STATE.with(|s| {
//         let mut st = s.borrow_mut();
//         match st.owners.get_mut(&caller) {
//             Some(p) => {
//                 p.name = name;
//                 p.gov_id_hash = gov_id_hash;
//                 if let Some(i) = inactivity_days { p.inactivity_days = i; }
//                 if let Some(w) = warning_days { p.warning_days = w; }
//                 let now_s = (ic_cdk::api::time() / 1_000_000_000) as u64;
//                 p.last_active = Some(now_s);
//                 Ok(())
//             }
//             None => Err("Owner not registered".to_string()),
//         }
//     })
// }

// #[query]
// pub fn get_me() -> Option<OwnerProfile> {
//     let caller = ic_cdk::caller();
//     STATE.with(|s| s.borrow().owners.get(&caller).cloned())
// }

// fn get_salt() -> Vec<u8> {
//     STATE.with(|s| s.borrow().salt.clone())
// }

// #[query]
// pub fn list_owners() -> Vec<Principal> {
//     STATE.with(|s| s.borrow().owners.keys().cloned().collect())
// }