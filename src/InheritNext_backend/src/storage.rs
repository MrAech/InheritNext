use candid::Principal;

use ic_stable_structures::{
    memory_manager::{MemoryId, MemoryManager, VirtualMemory},
    DefaultMemoryImpl, StableBTreeMap, StableCell,
};
use std::cell::RefCell;

use crate::{
    helpers::MAX_AUDIT_EVENT,
    types::{Asset, AssetId, AuditEvent, EventId, StablePrincipal, UserProfile, Vault},
};
type Memory = VirtualMemory<DefaultMemoryImpl>;

thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> =
    RefCell::new(
        MemoryManager::init(DefaultMemoryImpl::default())
    );

    static USERS: RefCell<StableBTreeMap<StablePrincipal, UserProfile, Memory>> =
    RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0)))
        )
    );

    static VAULTS: RefCell<StableBTreeMap<StablePrincipal, Vault, Memory >> =
    RefCell::new(
        StableBTreeMap::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(1))))
    );


    static AUDIT_LOG: RefCell<StableBTreeMap<EventId, AuditEvent, Memory>> =
    RefCell::new(
        StableBTreeMap::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(2))))
    );

    static ASSETS: RefCell<StableBTreeMap<AssetId, Asset, Memory >> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(3)))
        )
    );



    static NEXT_EVENT_ID: RefCell<StableCell<u64, Memory>> =
    RefCell::new(
        StableCell::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(5))), 0)
    );

    static NEXT_ASSET_ID: RefCell<StableCell<u64, Memory>> =
    RefCell::new(
        StableCell::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(6))), 0)
    );

}

fn return_stable_prin(p: &Principal) -> StablePrincipal {
    StablePrincipal(*p)
}

pub fn is_user_registered(visitor: &Principal) -> bool {
    USERS.with(|users| users.borrow().contains_key(&return_stable_prin(visitor)))
}

pub fn create_user(owner: &Principal, profile: UserProfile) -> Result<(), String> {
    USERS.with(|users| {
        let mut users = users.borrow_mut();

        if users.contains_key(&return_stable_prin(owner)) {
            return Err("User Already registered".to_string());
        }

        users.insert(return_stable_prin(owner), profile);

        Ok(())
    })
}

pub fn get_user(user: &Principal) -> Option<UserProfile> {
    USERS.with(|users| users.borrow().get(&return_stable_prin(user)))
}

pub fn vault_exists(user: &Principal) -> bool {
    VAULTS.with(|vaults| vaults.borrow().contains_key(&return_stable_prin(user)))
}

pub fn get_vault(owner: &Principal) -> Option<Vault> {
    VAULTS.with(|vaults| vaults.borrow().get(&return_stable_prin(owner)))
}

pub fn insert_vault(owner: &Principal, vault: Vault) {
    VAULTS.with(|vaults| {
        vaults.borrow_mut().insert(return_stable_prin(owner), vault);
    });
}

pub fn log_event(event: AuditEvent) -> EventId {
    let event_id = NEXT_EVENT_ID.with(|id| {
        let mut cell = id.borrow_mut();
        let current = *cell.get();
        let _ = cell.set(current + 1);
        EventId(current)
    });

    AUDIT_LOG.with(|log| {
        let mut log = log.borrow_mut();
        log.insert(event_id, event);

        if log.len() > MAX_AUDIT_EVENT {
            let oldest_id = log.iter().next().map(|entry| *entry.key());
            if let Some(id) = oldest_id {
                log.remove(&id);
            }
        }
    });
    event_id
}

pub fn update_vault<F, R>(blame: &Principal, f: F) -> Result<R, String>
where
    F: FnOnce(&mut Vault) -> Result<R, String>,
{
    VAULTS.with(|vaults| {
        let mut vaults = vaults.borrow_mut();
        let key = &return_stable_prin(blame);

        let mut vault = vaults
            .get(key)
            .ok_or_else(|| "No vaults founds".to_string())?;

        let res = f(&mut vault)?;

        vaults.insert(*key, vault);
        Ok(res)
    })
}

pub fn insert_asset(asset: Asset) {
    ASSETS.with(|assets| {
        assets.borrow_mut().insert(AssetId(asset.id), asset);
    });
}

pub fn get_asset(asset_id: u64) -> Option<Asset> {
    ASSETS.with(|assets| assets.borrow().get(&AssetId(asset_id)))
}

pub fn remove_asset(asset_id: u64) -> Option<Asset> {
    ASSETS.with(|assets| assets.borrow_mut().remove(&AssetId(asset_id)))
}

pub fn list_user_assets(owner: &Principal) -> Vec<Asset> {
    let mut result = Vec::new();
    ASSETS.with(|assets| {
        let assets = assets.borrow();
        for item in assets.iter() {
            let value = item.value();
            if value.owner == *owner {
                result.push(value);
            }
        }
    });
    result
}

// Global Counter to prevent assetid to being same

pub fn next_asset_id() -> u64 {
    NEXT_ASSET_ID.with(|id| {
        let mut cell = id.borrow_mut();
        let current = *cell.get();
        let _ = cell.set(current + 1);
        current
    })
}
