// This is the public interface for the canister.
// Most of the actual logic lives in api.rs, and all the data structures are in models.rs.

mod models;
mod api;

use crate::api::*;
use crate::models::*;


// There are still a few deprecated calls here so older generated bindings won't break.

#[ic_cdk_macros::update]
pub fn add_asset(new_asset: AssetInput) -> Result<(), CivError> {
    api::add_asset(new_asset)
}

#[ic_cdk_macros::update]
pub fn add_heir(new_heir: HeirInput) -> Result<(), CivError> {
    api::add_heir(new_heir)
}


// DEPRECATED: Use set_asset_distributions and delete_distribution instead.
#[deprecated(note = "Use set_asset_distributions/delete_distribution instead.")]
#[ic_cdk_macros::update]
pub fn assign_distributions(distributions: Vec<AssetDistribution>) -> Result<(), CivError> {
    api::assign_distributions(distributions)
}

// DEPRECATED: Use get_asset_distributions instead.
#[deprecated(note = "Use get_asset_distributions instead.")]
#[ic_cdk::query]
pub fn get_distribution() -> Vec<(String, u64)> {
    api::get_distribution()
}

// DEPRECATED: Prefer per-asset queries (get_asset_distributions). This will be removed soon.
#[deprecated(note = "Use get_asset_distributions instead.")]
#[ic_cdk::query]
pub fn list_distributions() -> Vec<AssetDistribution> {
    api::list_distributions()
}

#[ic_cdk::query]
pub fn get_asset_distributions(asset_id: u64) -> Vec<AssetDistribution> {
    api::get_asset_distributions(asset_id)
}

#[ic_cdk_macros::update]
pub fn set_asset_distributions(asset_id: u64, distributions: Vec<AssetDistribution>) -> Result<(), CivError> {
    api::set_asset_distributions(asset_id, distributions)
}

#[ic_cdk_macros::update]
pub fn delete_distribution(asset_id: u64, heir_id: u64) -> Result<(), CivError> {
    api::delete_distribution(asset_id, heir_id)
}

#[ic_cdk::query]
pub fn get_timer() -> i64 {
    api::get_timer()
}

#[ic_cdk::query]
pub fn get_user() -> Option<User> {
    api::get_user()
}

#[ic_cdk::query]
pub fn list_assets() -> Vec<Asset> {
    api::list_assets()
}

#[ic_cdk::query]
pub fn list_heirs() -> Vec<Heir> {
    api::list_heirs()
}

#[ic_cdk_macros::update]
pub fn remove_asset(asset_id: u64) -> Result<(), CivError> {
    api::remove_asset(asset_id)
}

#[ic_cdk_macros::update]
pub fn remove_heir(heir_id: u64) -> Result<(), CivError> {
    api::remove_heir(heir_id)
}

#[ic_cdk_macros::update]
pub fn reset_timer() -> Result<(), CivError> {
    api::reset_timer()
}

#[ic_cdk_macros::update]
pub fn update_asset(asset_id: u64, new_asset: AssetInput) -> Result<(), CivError> {
    api::update_asset(asset_id, new_asset)
}

#[ic_cdk_macros::update]
pub fn update_heir(heir_id: u64, new_heir: HeirInput) -> Result<(), CivError> {
    api::update_heir(heir_id, new_heir)
}

// Returns an integrity report for the caller's data, including invariants and allocation health.
#[ic_cdk::query]
pub fn check_integrity() -> IntegrityReport {
    api::check_integrity()
}

#[ic_cdk::query(name = "__get_candid_interface_tmp_hack")]
fn export_did() -> String {
    candid::export_service!();
    __export_service()
}

// remember to regenerate candid.
