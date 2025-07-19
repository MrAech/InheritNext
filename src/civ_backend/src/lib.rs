use ic_cdk::api::msg_caller;
use ic_cdk_macros::{query, update};
use candid::{CandidType, Deserialize};
use std::collections::HashMap;

/// Asset model: represents an estate asset.
#[derive(Clone, CandidType, Deserialize)]
pub struct Asset {
    pub id: u64,
    pub name: String,
    pub asset_type: String,
    pub value: u64,
    pub description: String,
}

/// Heir model: represents a beneficiary.
#[derive(Clone, CandidType, Deserialize)]
pub struct Heir {
    pub id: u64,
    pub name: String,
    pub relationship: String,
    pub percentage: u8,
    pub email: String,
    pub phone: String,
    pub address: String,
}

/// User model: links user to assets and heirs.
#[derive(Clone, CandidType, Deserialize)]
pub struct User {
    pub user: String,
    pub assets: Vec<Asset>,
    pub heirs: Vec<Heir>,
    pub timer: u64,
}

/// Custom error type for backend API responses.
#[derive(CandidType, Deserialize)]
pub enum CivError {
    /// Asset with given ID already exists.
    AssetExists,
    /// Asset not found.
    AssetNotFound,
    /// Heir with given ID already exists.
    HeirExists,
    /// Heir not found.
    HeirNotFound,
    /// User not found.
    UserNotFound,
    /// Total heir percentage is not 100.
    InvalidHeirPercentage,
    /// Other error.
    Other(String),
}

// Stable storage for user data.
thread_local! {
    static USERS: std::cell::RefCell<HashMap<String, User>> = std::cell::RefCell::new(HashMap::new());
}

// Helper to get user as string.
fn user_id() -> String {
    msg_caller().to_text()
}

// Helper to validate total heir percentages.
fn validate_heir_percentages(heirs: &[Heir]) -> bool {
    heirs.iter().map(|h| h.percentage as u32).sum::<u32>() == 100
}

/// Add a new asset for the msg_caller.
/// Returns error if asset ID already exists.
#[update]
pub fn add_asset(asset: Asset) -> Result<(), CivError> {
    let user = user_id();
    USERS.with(|users| {
        let mut users = users.borrow_mut();
        let user = users.entry(user.clone()).or_insert(User {
            user: user.clone(),
            assets: Vec::new(),
            heirs: Vec::new(),
            timer: 0,
        });
        if user.assets.iter().any(|a| a.id == asset.id) {
            return Err(CivError::AssetExists);
        }
        user.assets.push(asset);
        Ok(())
    })
}

/// Update an existing asset for the msg_caller.
/// Returns error if asset not found.
#[update]
pub fn update_asset(asset: Asset) -> Result<(), CivError> {
    let user = user_id();
    USERS.with(|users| {
        let mut users = users.borrow_mut();
        if let Some(user) = users.get_mut(&user) {
            if let Some(existing) = user.assets.iter_mut().find(|a| a.id == asset.id) {
                *existing = asset;
                Ok(())
            } else {
                Err(CivError::AssetNotFound)
            }
        } else {
            Err(CivError::UserNotFound)
        }
    })
}

/// Remove an asset by ID for the msg_caller.
/// Returns error if asset not found.
#[update]
pub fn remove_asset(asset_id: u64) -> Result<(), CivError> {
    let user = user_id();
    USERS.with(|users| {
        let mut users = users.borrow_mut();
        if let Some(user) = users.get_mut(&user) {
            let len_before = user.assets.len();
            user.assets.retain(|a| a.id != asset_id);
            if user.assets.len() < len_before {
                Ok(())
            } else {
                Err(CivError::AssetNotFound)
            }
        } else {
            Err(CivError::UserNotFound)
        }
    })
}

/// List all assets for the msg_caller.
#[query]
pub fn list_assets() -> Vec<Asset> {
    let user = user_id();
    USERS.with(|users| {
        users.borrow().get(&user).map(|u| u.assets.clone()).unwrap_or_default()
    })
}

/// Add a new heir for the msg_caller.
/// Returns error if heir ID already exists or total percentage is not 100.
#[update]
pub fn add_heir(heir: Heir) -> Result<(), CivError> {
    let user = user_id();
    USERS.with(|users| {
        let mut users = users.borrow_mut();
        let user = users.entry(user.clone()).or_insert(User {
            user: user.clone(),
            assets: Vec::new(),
            heirs: Vec::new(),
            timer: 0,
        });
        if user.heirs.iter().any(|h| h.id == heir.id) {
            return Err(CivError::HeirExists);
        }
        user.heirs.push(heir);
        if !validate_heir_percentages(&user.heirs) {
            user.heirs.pop();
            return Err(CivError::InvalidHeirPercentage);
        }
        Ok(())
    })
}

/// Update an existing heir for the msg_caller.
/// Returns error if heir not found or total percentage is not 100.
#[update]
pub fn update_heir(heir: Heir) -> Result<(), CivError> {
    let user = user_id();
    USERS.with(|users| {
        let mut users = users.borrow_mut();
        if let Some(user) = users.get_mut(&user) {
            if let Some(existing) = user.heirs.iter_mut().find(|h| h.id == heir.id) {
                *existing = heir;
                if !validate_heir_percentages(&user.heirs) {
                    return Err(CivError::InvalidHeirPercentage);
                }
                Ok(())
            } else {
                Err(CivError::HeirNotFound)
            }
        } else {
            Err(CivError::UserNotFound)
        }
    })
}

/// Remove a heir by ID for the msg_caller.
/// Returns error if heir not found or total percentage is not 100.
#[update]
pub fn remove_heir(heir_id: u64) -> Result<(), CivError> {
    let user = user_id();
    USERS.with(|users| {
        let mut users = users.borrow_mut();
        if let Some(user) = users.get_mut(&user) {
            let len_before = user.heirs.len();
            user.heirs.retain(|h| h.id != heir_id);
            if !validate_heir_percentages(&user.heirs) && !user.heirs.is_empty() {
                return Err(CivError::InvalidHeirPercentage);
            }
            if user.heirs.len() < len_before {
                Ok(())
            } else {
                Err(CivError::HeirNotFound)
            }
        } else {
            Err(CivError::UserNotFound)
        }
    })
}

/// List all heirs for the msg_caller.
#[query]
pub fn list_heirs() -> Vec<Heir> {
    let user = user_id();
    USERS.with(|users| {
        users.borrow().get(&user).map(|u| u.heirs.clone()).unwrap_or_default()
    })
}

/// Get asset distribution for the msg_caller.
#[query]
pub fn get_distribution() -> Vec<(String, u64)> {
    let user = user_id();
    USERS.with(|users| {
        users.borrow().get(&user)
            .map(|u| {
                u.heirs.iter().map(|h| (h.name.clone(), h.percentage as u64)).collect()
            })
            .unwrap_or_default()
    })
}

/// Get timer value for the msg_caller.
#[query]
pub fn get_timer() -> u64 {
    let user = user_id();
    USERS.with(|users| {
        users.borrow().get(&user).map(|u| u.timer).unwrap_or(0)
    })
}

/// Reset timer for the msg_caller.
#[update]
pub fn reset_timer() -> Result<(), CivError> {
    let user = user_id();
    USERS.with(|users| {
        let mut users = users.borrow_mut();
        if let Some(user) = users.get_mut(&user) {
            user.timer = 0;
            Ok(())
        } else {
            Err(CivError::UserNotFound)
        }
    })
}

/// Get user data for the msg_caller.
#[query]
pub fn get_user() -> Option<User> {
    let user = user_id();
    USERS.with(|users| users.borrow().get(&user).cloned())
}

#[ic_cdk::query(name = "__get_candid_interface_tmp_hack")]
fn export_did() -> String {
    candid::export_service!();
    __export_service()
}