use crate::models::*;
use ic_cdk::api::msg_caller;

fn user_id() -> String {
    msg_caller().to_text()
}

thread_local! {
    static USERS: std::cell::RefCell<std::collections::HashMap<String, User>> = std::cell::RefCell::new(std::collections::HashMap::new());
}

pub fn add_asset(new_asset: AssetInput) -> Result<(), CivError> {
    let user = user_id();
    USERS.with(|users| {
        let mut users = users.borrow_mut();
        let _now = ic_cdk::api::time();
        let user = users.entry(user.clone()).or_insert(User {
            user: user.clone(),
            assets: Vec::new(),
            heirs: Vec::new(),
            distributions: Vec::new(),
            timer_expiry: 0,
            distributed: false,
        });
        // Auto-generate unique asset ID
        let next_id = user.assets.iter().map(|a| a.id).max().unwrap_or(0) + 1;
        let now_ns = ic_cdk::api::time();
        let now = now_ns / 1_000_000_000; // convert nanoseconds to seconds
        user.assets.push(Asset {
            id: next_id,
            name: new_asset.name,
            asset_type: new_asset.asset_type,
            value: new_asset.value,
            description: new_asset.description,
            created_at: now,
            updated_at: now,
        });
        // If asset count is 1 or greater and timer not set, start the timer
        if user.assets.len() >= 1 && user.timer_expiry == 0 {
            user.timer_expiry = now + 30 * 24 * 60 * 60;
            user.distributed = false;
        }
        Ok(())
    })
}

pub fn add_heir(new_heir: HeirInput) -> Result<(), CivError> {
    let user = user_id();
    USERS.with(|users| {
        let mut users = users.borrow_mut();
        let now = ic_cdk::api::time();
        let user = users.entry(user.clone()).or_insert(User {
            user: user.clone(),
            assets: Vec::new(),
            heirs: Vec::new(),
            distributions: Vec::new(),
            timer_expiry: now + 30 * 24 * 60 * 60,
            distributed: false,
        });
        // Auto-generate unique heir ID
        let next_id = user.heirs.iter().map(|h| h.id).max().unwrap_or(0) + 1;
        ic_cdk::println!("add_heir: generated heir id = {}", next_id);
        let now = ic_cdk::api::time();
        user.heirs.push(Heir {
            id: next_id,
            name: new_heir.name,
            relationship: new_heir.relationship,
            email: new_heir.email,
            phone: new_heir.phone,
            address: new_heir.address,
            created_at: now,
            updated_at: now,
        });
        Ok(())
    })
}

pub fn assign_distributions(distributions: Vec<AssetDistribution>) -> Result<(), CivError> {
    let user = user_id();
    USERS.with(|users| {
        let mut users = users.borrow_mut();
        let now = ic_cdk::api::time();
        let user = users.entry(user.clone()).or_insert(User {
            user: user.clone(),
            assets: Vec::new(),
            heirs: Vec::new(),
            distributions: Vec::new(),
            timer_expiry: now + 30 * 24 * 60 * 60,
            distributed: false,
        });


        use std::collections::HashMap;
        let mut asset_map: HashMap<u64, u32> = HashMap::new();
        for dist in &distributions {
            if !user.assets.iter().any(|a| a.id == dist.asset_id) {
                return Err(CivError::DistributionAssetNotFound);
            }
            if !user.heirs.iter().any(|h| h.id == dist.heir_id) {
                return Err(CivError::DistributionHeirNotFound);
            }
            *asset_map.entry(dist.asset_id).or_insert(0) += dist.percentage as u32;
        }
        if asset_map.values().any(|&sum| sum != 100) {
            return Err(CivError::InvalidHeirPercentage);
        }


        user.distributions = distributions;
        Ok(())
    })
}

pub fn get_distribution() -> Vec<(String, u64)> {
    let user = user_id();
    USERS.with(|users| {
        let users = users.borrow();
        if let Some(user) = users.get(&user) {
            user.distributions.iter().map(|d| (d.asset_id.to_string(), d.heir_id)).collect()
        } else {
            vec![]
        }
    })
}

pub fn get_timer() -> i64 {
    let user = user_id();
    USERS.with(|users| {
        let mut users = users.borrow_mut();
        if let Some(user) = users.get_mut(&user) {
            let now_ns = ic_cdk::api::time();
            let now = now_ns / 1_000_000_000; // convert nanoseconds to seconds
            ic_cdk::println!("get_timer: asset_count={}, timer_expiry={}", user.assets.len(), user.timer_expiry);
            // If no assets, timer not started
            if user.assets.is_empty() {
                ic_cdk::println!("get_timer: no assets, returning -1");
                return -1; // Sentinel value for "not started"
            }
            // If asset count is 1 or greater and timer not set, start the timer
            if user.assets.len() >= 1 && user.timer_expiry == 0 {
                user.timer_expiry = now + 30 * 24 * 60 * 60;
                user.distributed = false;
                ic_cdk::println!("get_timer: timer started, expiry={}", user.timer_expiry);
            }
            if user.timer_expiry < now && !user.distributed && !user.assets.is_empty() {
                // Timer expired, auto-distribute: clear assets/distributions and set distributed flag
                user.assets.clear();
                user.distributions.clear();
                user.distributed = true;
                ic_cdk::println!("get_timer: timer expired, assets cleared");
            }
            let remaining = user.timer_expiry.saturating_sub(now) as i64;
            ic_cdk::println!("get_timer: returning remaining={}", remaining);
            remaining
        } else {
            -1
        }
    })
}

pub fn get_user() -> Option<User> {
    let user = user_id();
    USERS.with(|users| {
        let users = users.borrow();
        users.get(&user).cloned()
    })
}

pub fn list_assets() -> Vec<Asset> {
    let user = user_id();
    USERS.with(|users| {
        let users = users.borrow();
        users.get(&user).map(|u| u.assets.clone()).unwrap_or_default()
    })
}

pub fn list_heirs() -> Vec<Heir> {
    let user = user_id();
    USERS.with(|users| {
        let users = users.borrow();
        users.get(&user).map(|u| u.heirs.clone()).unwrap_or_default()
    })
}

pub fn remove_asset(asset_id: u64) -> Result<(), CivError> {
    let user = user_id();
    USERS.with(|users| {
        let mut users = users.borrow_mut();
        if let Some(user) = users.get_mut(&user) {
            let orig_len = user.assets.len();
            user.assets.retain(|a| a.id != asset_id);
            if user.assets.len() == orig_len {
                return Err(CivError::AssetNotFound);
            }
            Ok(())
        } else {
            Err(CivError::UserNotFound)
        }
    })
}

pub fn remove_heir(heir_id: u64) -> Result<(), CivError> {
    let user = user_id();
    USERS.with(|users| {
        let mut users = users.borrow_mut();
        if let Some(user) = users.get_mut(&user) {
            let orig_len = user.heirs.len();
            user.heirs.retain(|h| h.id != heir_id);
            if user.heirs.len() == orig_len {
                return Err(CivError::HeirNotFound);
            }
            Ok(())
        } else {
            Err(CivError::UserNotFound)
        }
    })
}

pub fn reset_timer() -> Result<(), CivError> {
    let user = user_id();
    USERS.with(|users| {
        let mut users = users.borrow_mut();
        let now_ns = ic_cdk::api::time();
        let now = now_ns / 1_000_000_000; // convert nanoseconds to seconds
        if let Some(user) = users.get_mut(&user) {
            user.timer_expiry = now + 30 * 24 * 60 * 60;
            user.distributed = false;
            Ok(())
        } else {
            Err(CivError::UserNotFound)
        }
    })
}

pub fn update_asset(asset_id: u64, new_asset: AssetInput) -> Result<(), CivError> {
    let user = user_id();
    USERS.with(|users| {
        let mut users = users.borrow_mut();
        if let Some(user) = users.get_mut(&user) {
            if let Some(existing) = user.assets.iter_mut().find(|a| a.id == asset_id) {
                existing.name = new_asset.name;
                existing.asset_type = new_asset.asset_type;
                existing.value = new_asset.value;
                existing.description = new_asset.description;
                existing.updated_at = ic_cdk::api::time();
                Ok(())
            } else {
                Err(CivError::AssetNotFound)
            }
        } else {
            Err(CivError::UserNotFound)
        }
    })
}

pub fn update_heir(heir_id: u64, new_heir: HeirInput) -> Result<(), CivError> {
    let user = user_id();
    USERS.with(|users| {
        let mut users = users.borrow_mut();
        if let Some(user) = users.get_mut(&user) {
            if let Some(existing) = user.heirs.iter_mut().find(|h| h.id == heir_id) {
                existing.name = new_heir.name;
                existing.relationship = new_heir.relationship;
                existing.email = new_heir.email;
                existing.phone = new_heir.phone;
                existing.address = new_heir.address;
                existing.updated_at = ic_cdk::api::time();
                Ok(())
            } else {
                Err(CivError::HeirNotFound)
            }
        } else {
            Err(CivError::UserNotFound)
        }
    })
}
