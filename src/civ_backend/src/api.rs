use crate::models::*;
use ic_cdk::api::msg_caller;


// All time values here are in seconds (not nanoseconds)— makes the math easier to follow
// If we ever need more precision, we can always add another field
// For now, everything lives in a single in-memory HashMap keyed by the caller's principal as text
// Will Add Stable Storage later


// Returns the current time as whole seconds since the UNIX epoch, based on IC system time.
// --helper fun--
fn now_secs() -> u64 {
    ic_cdk::api::time() / 1_000_000_000
}

fn user_id() -> String {
    msg_caller().to_text()
}

thread_local! {
    // Single map for now. If this grows huge we can revisit structure.
    // @MrAech @sharmayash2805 thoughts ??
    static USERS: std::cell::RefCell<std::collections::HashMap<String, User>> =
        std::cell::RefCell::new(std::collections::HashMap::new());
}

// Asset CRUD operations.
// Note: Adding the first asset will kick off the inactivity timer for the user.
// Will update to wait till first distribution later
pub fn add_asset(new_asset: AssetInput) -> Result<(), CivError> {
    let user = user_id();
    USERS.with(|users| {
        let mut users = users.borrow_mut();
    let now = now_secs();
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
        // TODO: need to change to look for first distribution instead
        if user.assets.len() >= 1 && user.timer_expiry == 0 {
            user.timer_expiry = now + 30 * 24 * 60 * 60;
            user.distributed = false;
        }
        Ok(())
    })
}

// Heir Realted operations
pub fn add_heir(new_heir: HeirInput) -> Result<(), CivError> {
    let user = user_id();
    USERS.with(|users| {
        let mut users = users.borrow_mut();
        let now = now_secs();
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

// Distribution logic. Legacy bulk API kept temporarily while frontend migrates. @MrAech Please migrate and remove them 

// DEPRECATED: Use set_asset_distributions and delete_distribution instead—these allow partial totals.
#[deprecated(note = "Use set_asset_distributions (per-asset partial) and delete_distribution instead.")]
pub fn assign_distributions(distributions: Vec<AssetDistribution>) -> Result<(), CivError> {
    let user = user_id();
    USERS.with(|users| {
        let mut users = users.borrow_mut();
    let now = now_secs();
        let user = users.entry(user.clone()).or_insert(User {
            user: user.clone(),
            assets: Vec::new(),
            heirs: Vec::new(),
            distributions: Vec::new(),
            timer_expiry: now + 30 * 24 * 60 * 60,
            distributed: false,
        });


        ic_cdk::println!(
            "assign_distributions: caller={}, incoming_count={}",
            user.user,
            distributions.len()
        );

        use std::collections::HashMap;
        let mut asset_map: HashMap<u64, u32> = HashMap::new();
        for dist in &distributions {
            ic_cdk::println!(
                "assign_distributions: item asset_id={}, heir_id={}, percentage={}",
                dist.asset_id,
                dist.heir_id,
                dist.percentage
            );
            if !user.assets.iter().any(|a| a.id == dist.asset_id) {
                return Err(CivError::DistributionAssetNotFound);
            }
            if !user.heirs.iter().any(|h| h.id == dist.heir_id) {
                return Err(CivError::DistributionHeirNotFound);
            }
            *asset_map.entry(dist.asset_id).or_insert(0) += dist.percentage as u32;
        }
        if asset_map.values().any(|&sum| sum != 100) {
            ic_cdk::println!("assign_distributions: invalid totals per asset: {:?}", asset_map);
            return Err(CivError::InvalidHeirPercentage);
        }

        ic_cdk::println!("assign_distributions: accepted, storing {} entries", distributions.len());
        user.distributions = distributions;
        Ok(())
    })
}


// Use get_asset_distributions for more detail.
// DEPRECATED: Old minimal distribution format (didn't keep percentages).
#[deprecated(note = "Use get_asset_distributions instead.")]
pub fn get_distribution() -> Vec<(String, u64)> {
    let user = user_id();
    USERS.with(|users| {
        let users = users.borrow();
        if let Some(user) = users.get(&user) {
            ic_cdk::println!(
                "get_distribution: caller={}, stored_count={}",
                user.user,
                user.distributions.len()
            );
            user
                .distributions
                .iter()
                .map(|d| (d.asset_id.to_string(), d.heir_id))
                .collect()
        } else {
            vec![]
        }
    })
}

// DEPRECATED: Use get_asset_distributions for per-asset queries, or batch on the client side.
#[deprecated(note = "Use get_asset_distributions; this will be removed.")]
pub fn list_distributions() -> Vec<AssetDistribution> {
    let user = user_id();
    USERS.with(|users| {
        let users = users.borrow();
        users
            .get(&user)
            .map(|u| u.distributions.clone())
            .unwrap_or_default()
    })
}


// Returns all distribution entries for a single asset.
// This is the most precise way to query distributions for a specific asset. atleast for now 
pub fn get_asset_distributions(asset_id: u64) -> Vec<AssetDistribution> {
    let user = user_id();
    USERS.with(|users| {
        let users = users.borrow();
        users
            .get(&user)
            .map(|u| u.distributions.iter().filter(|d| d.asset_id == asset_id).cloned().collect())
            .unwrap_or_else(|| Vec::new())
    })
}


// Replaces all distribution entries for a given asset.
// Allows the total to be anywhere from 0 to 100.
pub fn set_asset_distributions(asset_id: u64, distributions: Vec<AssetDistribution>) -> Result<(), CivError> {
    let user = user_id();
    USERS.with(|users| {
        let mut users = users.borrow_mut();
        if let Some(user) = users.get_mut(&user) {
            // Validate asset exists
            if !user.assets.iter().any(|a| a.id == asset_id) {
                return Err(CivError::DistributionAssetNotFound);
            }
            // Validate all entries target same asset and heirs exist; compute total
            let mut total: u32 = 0;
            for d in &distributions {
                if d.asset_id != asset_id {
                    return Err(CivError::DistributionAssetNotFound);
                }
                if !user.heirs.iter().any(|h| h.id == d.heir_id) {
                    return Err(CivError::DistributionHeirNotFound);
                }
                total += d.percentage as u32;
            }
            // Allow partial (including empty) distributions; enforce only upper bound 100
            if total > 100 {
                return Err(CivError::InvalidHeirPercentage);
            }
            // Replace existing for this asset
            user.distributions.retain(|d| d.asset_id != asset_id);
            user.distributions.extend(distributions);
            Ok(())
        } else {
            Err(CivError::UserNotFound)
        }
    })
}

// Removes a single (asset, heir) distribution entry.
// Returns an error if nothing was removed, so the UI can handle stale state.  @MrAech as requested do 
pub fn delete_distribution(asset_id: u64, heir_id: u64) -> Result<(), CivError> {
    let user = user_id();
    USERS.with(|users| {
        let mut users = users.borrow_mut();
        if let Some(user) = users.get_mut(&user) {
            let before = user.distributions.len();
            user.distributions.retain(|d| !(d.asset_id == asset_id && d.heir_id == heir_id));
            if before == user.distributions.len() {
                return Err(CivError::DistributionHeirNotFound);
            }
            Ok(())
        } else {
            Err(CivError::UserNotFound)
        }
    })
}

// Inactivity timer logic:
// Timer starts when the first asset is added. <- needs to change TODO
// When the timer expires, we clear out assets and distributions.
// Returns seconds remaining, or -1 if the timer was never started.
pub fn get_timer() -> i64 {
    let user = user_id();
    USERS.with(|users| {
        let mut users = users.borrow_mut();
        if let Some(user) = users.get_mut(&user) {
            let now = now_secs();
            ic_cdk::println!("get_timer: asset_count={}, timer_expiry={}", user.assets.len(), user.timer_expiry);
            // If no assets, timer not started
            if user.assets.is_empty() {
                ic_cdk::println!("get_timer: no assets, returning -1");
                return -1; // "not started"
            }
            // If asset count is 1 or greater and timer not set, start the timer
            // TODO change to distribution
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

// Convenience query functions for user, assets, and heirs.
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


// Resets the inactivity timer to a fresh 30-day window.
// Used when the user explicitly requests a reset.
pub fn reset_timer() -> Result<(), CivError> {
    let user = user_id();
    USERS.with(|users| {
        let mut users = users.borrow_mut();
    let now = now_secs();
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
                existing.updated_at = now_secs();
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
                existing.updated_at = now_secs();
                Ok(())
            } else {
                Err(CivError::HeirNotFound)
            }
        } else {
            Err(CivError::UserNotFound)
        }
    })
}


// Integrity snapshot: Sums up percentages and checks for over/under allocations,
// as well as any stale references.

// Checks the overall health and invariants for the caller's data.
pub fn check_integrity() -> IntegrityReport {
    let user = user_id();
    USERS.with(|users| {
        let users = users.borrow();
        if let Some(user) = users.get(&user) {
            use std::collections::HashMap;
            let mut map: HashMap<u64, u32> = HashMap::new();
            let mut issues: Vec<String> = Vec::new();
            for d in &user.distributions {
                // Tally percentages
                *map.entry(d.asset_id).or_insert(0) += d.percentage as u32;
                // Validate referenced asset
                if !user.assets.iter().any(|a| a.id == d.asset_id) {
                    issues.push(format!("Distribution references missing asset {}", d.asset_id));
                }
                // Validate referenced heir
                if !user.heirs.iter().any(|h| h.id == d.heir_id) {
                    issues.push(format!("Distribution references missing heir {}", d.heir_id));
                }
                if d.percentage == 0 { issues.push(format!("Zero percentage entry asset {} heir {}", d.asset_id, d.heir_id)); }
            }
            let mut over = Vec::new();
            let mut full = Vec::new();
            let mut partial = Vec::new();
            for asset in &user.assets {
                let sum = map.get(&asset.id).copied().unwrap_or(0);
                if sum > 100 { over.push(asset.id); }
                else if sum == 100 { full.push(asset.id); }
                else if sum > 0 { partial.push(asset.id); }
            }
            let unallocated: Vec<u64> = user.assets.iter().filter(|a| !map.contains_key(&a.id)).map(|a| a.id).collect();
            IntegrityReport {
                asset_count: user.assets.len() as u64,
                distribution_count: user.distributions.len() as u64,
                over_allocated_assets: over,
                fully_allocated_assets: full,
                partially_allocated_assets: partial,
                unallocated_assets: unallocated,
                issues,
            }
        } else {
            IntegrityReport {
                asset_count: 0,
                distribution_count: 0,
            over_allocated_assets: vec![],
                fully_allocated_assets: vec![],
                partially_allocated_assets: vec![],
                unallocated_assets: vec![],
                issues: vec!["User not initialized".to_string()],
            }
        }
    })
}
