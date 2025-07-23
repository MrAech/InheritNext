mod models;
mod api;

use crate::api::*;
use crate::models::*;

#[ic_cdk_macros::update]
pub fn add_asset(new_asset: AssetInput) -> Result<(), CivError> {
    api::add_asset(new_asset)
}

#[ic_cdk_macros::update]
pub fn add_heir(new_heir: HeirInput) -> Result<(), CivError> {
    api::add_heir(new_heir)
}

#[ic_cdk_macros::update]
pub fn assign_distributions(distributions: Vec<AssetDistribution>) -> Result<(), CivError> {
    api::assign_distributions(distributions)
}

#[ic_cdk::query]
pub fn get_distribution() -> Vec<(String, u64)> {
    api::get_distribution()
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

#[ic_cdk::query(name = "__get_candid_interface_tmp_hack")]
fn export_did() -> String {
    candid::export_service!();
    __export_service()
}