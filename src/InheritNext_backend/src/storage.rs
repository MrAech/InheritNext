use candid::Principal;
use ic_stable_structures::{
    memory_manager::{MemoryId, MemoryManager, VirtualMemory},
    DefaultMemoryImpl, StableBTreeMap,
};
use std::cell::RefCell;

use crate::types::{AuditEvent, EventId, StablePrincipal, UserProfile, Vault};
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


    static NEXT_EVENT_ID: RefCell<u64> = RefCell::new(0);
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
        let current = *id.borrow();
        *id.borrow_mut() = current + 1;
        EventId(current)
    });

    AUDIT_LOG.with(|log| {
        log.borrow_mut().insert(event_id, event);
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
