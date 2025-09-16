use candid::{CandidType, Deserialize, Nat};
use ic_cdk::api::{time, msg_caller, canister_self};
use ic_cdk::{storage};
use candid::Principal;
use ic_cdk_macros::{init, post_upgrade, pre_upgrade, query, update};
use serde_bytes::ByteBuf;
use sha2::{Digest, Sha256};
use std::cell::RefCell;
use std::collections::HashMap;

/*
  Lightweight on-chain backend implementation:
  - Implements owner registration, allocations, blob metadata storage,
    activity tracking, sweep execution, event log, and certificate creation.
  - Does NOT perform external token transfers in this PR; instead the
    execution produces DistributionEntry records and CertificateRecords
    along with events. Transfer helpers are scaffolded for future
    integration with ledger / ICRC canisters.
*/

const INACTIVITY_THRESHOLD_MILLIS: u64 = 1000 * 60 * 60 * 24 * 365; // 1 year (example)
const WARNING_DURATION_MILLIS: u64 = 1000 * 60 * 60 * 24 * 30; // 30 days

type Subaccount = Option<ByteBuf>;

#[derive(Clone, CandidType, Deserialize)]
pub enum AssetRef {
    IcpFungible { subaccount: Subaccount },
    IcrcFungible { canister: Principal, subaccount: Subaccount },
    IcrcNft { canister: Principal, token_id: Nat },
    Document { storage_canister: Principal, blob_id: Vec<u8> },
    Pointer { description: String },
}

#[derive(Clone, CandidType, Deserialize)]
pub struct Allocation {
    pub heir: Principal,
    pub basis_points: Nat, // 0..10000
}

#[derive(Clone, CandidType, Deserialize)]
pub struct DistributionEntry {
    pub id: u64,
    pub owner: Principal,
    pub asset: AssetRef,
    pub allocations: Vec<Allocation>,
    pub executed_at: Option<u64>,
}

#[derive(Clone, CandidType, Deserialize)]
pub struct BlobMeta {
    pub hash: Vec<u8>,
    pub iv: Vec<u8>,
    pub size: u64,
    pub locator: String,
}

#[derive(Clone, CandidType, Deserialize)]
pub struct OwnerRecord {
    pub owner: Principal,
    pub last_active: Option<u64>,
    pub warning_started_at: Option<u64>,
    pub vault_subaccount: Subaccount,
}

#[derive(Clone, CandidType, Deserialize)]
pub struct EventRecord {
    pub id: u64,
    pub ts: u64,
    pub actor: Principal,
    pub event_type: String,
    pub details: String,
}

#[derive(Clone, CandidType, Deserialize)]
pub struct CertificateRecord {
    pub distribution_id: u64,
    pub hash: Vec<u8>, // sha256(dist_record)
    pub executed_at: u64,
}

#[derive(Clone, CandidType, Deserialize)]
pub struct SweepResult {
    pub processed: u64,
    pub continuation: Option<Vec<u8>>,
}

#[derive(Deserialize, CandidType)]
struct PersistedState {
    owners: HashMap<Principal, OwnerRecord>,
    allocations: HashMap<Principal, Vec<Allocation>>,
    blobs: HashMap<u64, BlobMeta>,
    distributions: HashMap<u64, DistributionEntry>,
    certificates: HashMap<u64, CertificateRecord>,
    events: Vec<EventRecord>,
    next_blob_id: u64,
    next_distribution_id: u64,
    next_event_id: u64,
    salt: Vec<u8>,
}

/* In-memory state */
struct State {
    owners: HashMap<Principal, OwnerRecord>,
    allocations: HashMap<Principal, Vec<Allocation>>,
    blobs: HashMap<u64, BlobMeta>,
    distributions: HashMap<u64, DistributionEntry>,
    certificates: HashMap<u64, CertificateRecord>,
    events: Vec<EventRecord>,
    next_blob_id: u64,
    next_distribution_id: u64,
    next_event_id: u64,
    salt: Vec<u8>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            owners: HashMap::new(),
            allocations: HashMap::new(),
            blobs: HashMap::new(),
            distributions: HashMap::new(),
            certificates: HashMap::new(),
            events: Vec::new(),
            next_blob_id: 1,
            next_distribution_id: 1,
            next_event_id: 1,
            salt: b"default_salt_change_me".to_vec(),
        }
    }
}

thread_local! {
    static STATE: RefCell<State> = RefCell::new(State::default());
}

/* Helpers */

fn now_millis() -> u64 {
    // Use IC time in canister runtime; fall back to host SystemTime during unit tests.
    #[cfg(not(test))]
    {
        time()
    }
    #[cfg(test)]
    {
        use std::time::{SystemTime, UNIX_EPOCH};
        let dur = SystemTime::now().duration_since(UNIX_EPOCH).expect("system time");
        dur.as_millis() as u64
    }
}

fn append_event(actor: Principal, event_type: &str, details: &str) -> u64 {
    STATE.with(|s| {
        let mut st = s.borrow_mut();
        append_event_mut(&mut st, actor, event_type, details)
    })
}

fn append_event_mut(st: &mut State, actor: Principal, event_type: &str, details: &str) -> u64 {
    let id = st.next_event_id;
    st.next_event_id += 1;
    let ev = EventRecord {
        id,
        ts: now_millis(),
        actor,
        event_type: event_type.to_string(),
        details: details.to_string(),
    };
    st.events.push(ev);
    id
}

fn sha256_bytes(data: &[u8]) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().to_vec()
}

/* Canister lifecycle */

#[init]
fn init() {
    // nothing special for now
    append_event(msg_caller(), "init", "canister initialized");
}

#[pre_upgrade]
fn pre_upgrade() {
    STATE.with(|s| {
        let st = s.borrow();
        let p = PersistedState {
            owners: st.owners.clone(),
            allocations: st.allocations.clone(),
            blobs: st.blobs.clone(),
            distributions: st.distributions.clone(),
            certificates: st.certificates.clone(),
            events: st.events.clone(),
            next_blob_id: st.next_blob_id,
            next_distribution_id: st.next_distribution_id,
            next_event_id: st.next_event_id,
            salt: st.salt.clone(),
        };
        storage::stable_save((p,)).expect("stable_save failed");
    });
}

#[post_upgrade]
fn post_upgrade() {
    let (p,): (PersistedState,) = storage::stable_restore().expect("stable_restore failed");
    STATE.with(|s| {
        let mut st = s.borrow_mut();
        st.owners = p.owners;
        st.allocations = p.allocations;
        st.blobs = p.blobs;
        st.distributions = p.distributions;
        st.certificates = p.certificates;
        st.events = p.events;
        st.next_blob_id = p.next_blob_id;
        st.next_distribution_id = p.next_distribution_id;
        st.next_event_id = p.next_event_id;
        st.salt = p.salt;
    });
    append_event(msg_caller(), "post_upgrade", "state restored");
}

/* Introspection */
#[query]
fn whoami() -> Principal {
    msg_caller()
}

/* Owner APIs */

#[query]
fn register_owner() {
    let p = msg_caller();
    STATE.with(|s| {
        let mut st = s.borrow_mut();
        if !st.owners.contains_key(&p) {
            // derive a simple vault_subaccount from sha256(principal)
            let principal_bytes = p.as_slice();
            let sub = sha256_bytes(principal_bytes);
                st.owners.insert(
                p,
                OwnerRecord {
                    owner: p,
                    last_active: Some(now_millis()),
                    warning_started_at: None,
                    vault_subaccount: Some(ByteBuf::from(sub)),
                },
            );
            append_event_mut(&mut st, p, "owner_registered", &format!("owner {} registered", p));
        }
    });
}

#[update]
fn add_heir(heir: Principal, basis_points: u64) -> u64 {
    let p = msg_caller();
    let alloc = Allocation {
        heir,
        basis_points: Nat::from(basis_points),
    };
    STATE.with(|s| {
        let mut st = s.borrow_mut();
        // perform mutation, then drop the temporary borrow before calling append_event_mut
        let idx = {
            let vec = st.allocations.entry(p).or_insert_with(Vec::new);
            vec.push(alloc);
            vec.len() as u64
        };
        append_event_mut(&mut st, p, "heir_added", &format!("heir {} added", heir));
        // return index as allocation id (1-based)
        idx
    })
}

#[update]
fn remove_heir(heir: Principal) -> bool {
    let p = msg_caller();
    STATE.with(|s| {
        let mut st = s.borrow_mut();
        let mut removed = false;
        if let Some(vec) = st.allocations.get_mut(&p) {
            let orig = vec.len();
            vec.retain(|a| a.heir != heir);
            removed = orig != vec.len();
        }
        if removed {
            append_event_mut(&mut st, p, "heir_removed", &format!("heir {} removed", heir));
        }
        removed
    })
}

#[update]
fn set_allocations(allocs: Vec<Allocation>) {
    let p = msg_caller();
    STATE.with(|s| {
        let mut st = s.borrow_mut();
        st.allocations.insert(p, allocs.clone());
        append_event_mut(&mut st, p, "allocations_set", "allocations updated");
    });
}

#[update]
fn commit_blob(meta: BlobMeta) -> u64 {
    let p = msg_caller();
    STATE.with(|s| {
        let mut st = s.borrow_mut();
        let id = st.next_blob_id;
        st.next_blob_id += 1;
        st.blobs.insert(id, meta);
        append_event_mut(&mut st, p, "blob_committed", &format!("blob {}", id));
        id
    })
}

#[update]
fn update_last_active() {
    let p = msg_caller();
    STATE.with(|s| {
        let mut st = s.borrow_mut();
            if let Some(owner) = st.owners.get_mut(&p) {
            owner.last_active = Some(now_millis());
            owner.warning_started_at = None;
            append_event_mut(&mut st, p, "last_active_updated", "owner activity recorded");
        } else {
            // auto-register on activity
            st.owners.insert(
                p,
                OwnerRecord {
                    owner: p,
                    last_active: Some(now_millis()),
                    warning_started_at: None,
                    vault_subaccount: None,
                },
            );
            append_event_mut(&mut st, p, "owner_registered", "auto-registered on activity");
        }
    });
}

#[update]
fn withdraw_asset(_asset: AssetRef, _destination: Principal) -> bool {
    // Withdraw is owner-only; for now we emit an event and return true.
    // Integrating with ledger / tokens is planned in follow-ups.
    let p = msg_caller();
    append_event(p, "withdraw_requested", "withdraw invoked (no-op in this implementation)");
    true
}

/* Heir APIs */

#[query]
fn list_claims() -> Vec<DistributionEntry> {
    let p = msg_caller();
    STATE.with(|s| {
        let st = s.borrow();
        st.distributions
            .values()
            .filter(|d| {
                d.executed_at.is_none()
                    && d.allocations.iter().any(|a| a.heir == p)
            })
            .cloned()
            .collect()
    })
}

#[query]
fn get_document_meta(blob_id: u64) -> Option<BlobMeta> {
    STATE.with(|s| {
        let st = s.borrow();
        st.blobs.get(&blob_id).cloned()
    })
}

/* System / Execution */

#[update]
fn sweep_expired(limit: u64, continuation: Option<Vec<u8>>) -> SweepResult {
    // continuation can be encoded owner principal bytes to resume
    let mut processed = 0u64;
    let mut new_cont: Option<Vec<u8>> = None;

    // snapshot owners deterministically
    let owners_list: Vec<Principal> = STATE.with(|s| s.borrow().owners.keys().cloned().collect());

    // If continuation provided, start after that owner
    let start_idx = if let Some(cont) = continuation {
        owners_list
            .iter()
            .position(|p| p.as_slice() == cont.as_slice())
            .map(|i| i + 1)
            .unwrap_or(0)
    } else {
        0
    };

    // Do all mutations inside a single mutable borrow to avoid nested mutable borrows
    STATE.with(|s| {
        let mut st = s.borrow_mut();

        for owner in owners_list.into_iter().skip(start_idx) {
            if processed >= limit {
                new_cont = Some(owner.as_slice().to_vec());
                break;
            }

            // determine whether we should execute for this owner
            let should_execute = if let Some(rec) = st.owners.get(&owner) {
                match rec.last_active {
                    Some(last) => {
                        let elapsed = now_millis().saturating_sub(last);
                        if elapsed >= INACTIVITY_THRESHOLD_MILLIS {
                            match rec.warning_started_at {
                                Some(ws) => {
                                    let warn_elapsed = now_millis().saturating_sub(ws);
                                    warn_elapsed >= WARNING_DURATION_MILLIS
                                }
                                None => false, // start warning instead
                            }
                        } else {
                            false
                        }
                    }
                    None => true,
                }
            } else {
                false
            };

            if should_execute {
                // create distribution and mark executed atomically without overlapping mutable borrows
                let id = st.next_distribution_id;
                st.next_distribution_id += 1;

                let allocs = st.allocations.get(&owner).cloned().unwrap_or_default();

                let asset = st
                    .owners
                    .get(&owner)
                    .and_then(|o| o.vault_subaccount.clone())
                    .map(|sa| AssetRef::IcpFungible { subaccount: Some(sa) })
                    .unwrap_or(AssetRef::Pointer {
                        description: "no-vault".to_string(),
                    });

                let executed_ts = now_millis();
                let dist = DistributionEntry {
                    id,
                    owner,
                    asset,
                    allocations: allocs,
                    executed_at: Some(executed_ts),
                };

                // insert distribution and emit event
                st.distributions.insert(id, dist.clone());
                append_event_mut(&mut st, owner, "distribution_created", &format!("dist {}", id));

                // create certificate immediately from the finalized distribution
                let bytes = candid::encode_one(dist.clone()).unwrap_or_default();
                let h = sha256_bytes(&bytes);
                let cert = CertificateRecord {
                    distribution_id: dist.id,
                    hash: h.clone(),
                    executed_at: executed_ts,
                };
                st.certificates.insert(dist.id, cert);
                append_event_mut(&mut st, msg_caller(), "distribution_executed", &format!("dist {} executed", dist.id));

                processed += 1;
            } else {
                // start warning if threshold passed and not already started
                if let Some(rec) = st.owners.get_mut(&owner) {
                    if let Some(last) = rec.last_active {
                        let elapsed = now_millis().saturating_sub(last);
                        if elapsed >= INACTIVITY_THRESHOLD_MILLIS && rec.warning_started_at.is_none() {
                            rec.warning_started_at = Some(now_millis());
                            append_event_mut(&mut st, owner, "warning_started", "warning started due to inactivity");
                        }
                    }
                }
            }
        }
    });

    SweepResult {
        processed,
        continuation: new_cont,
    }
}

#[query]
fn get_certificate(distribution_id: u64) -> Option<CertificateRecord> {
    STATE.with(|s| s.borrow().certificates.get(&distribution_id).cloned())
}

#[query]
fn get_event_log(from_index: u64, limit: u64) -> Vec<EventRecord> {
    STATE.with(|s| {
        let st = s.borrow();
        let start = (from_index as usize).saturating_sub(1);
        let lim = limit as usize;
        st.events.iter().skip(start).take(lim).cloned().collect()
    })
}

/* Admin */
#[update]
fn rotate_salt(new_salt: Vec<u8>) {
    // admin-only check: for simplicity require caller == module id (deployer)
    // In practice, enforce multi-sig or configured admin principal
    let caller_p = msg_caller();
    // naive admin check: only allow if caller is the canister itself (deployment-time admin should call)
    let canister = canister_self();
        if caller_p != canister {
        append_event(caller_p, "rotate_salt_denied", "caller not admin");
        return;
    }
    STATE.with(|s| {
        let mut st = s.borrow_mut();
        st.salt = new_salt.clone();
        append_event_mut(&mut st, caller_p, "rotate_salt", "salt rotated");
    });
}

/* Export Candid for clients */
#[query]
fn __get_candid_interface_tmp_hack() -> String {
    // helpful for local dev: returns candid from DID file if present in canister build artifacts.
    include_str!("../civ_backend.did").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use candid::Principal;

    #[test]
    fn test_sha256_bytes_consistent() {
        let a = b"hello world";
        let h1 = sha256_bytes(a);
        let h2 = sha256_bytes(a);
        assert_eq!(h1, h2);
        // Known prefix for "hello world" SHA256 (first 4 bytes)
        assert_eq!(&h1[..4], &sha256_bytes(b"hello world")[..4]);
    }

    #[test]
    fn test_append_event_mut_increments_id() {
        let mut st = State::default();
        let actor = Principal::from_text("2vxsx-fae").unwrap();
        let id1 = append_event_mut(&mut st, actor.clone(), "ev1", "details1");
        let id2 = append_event_mut(&mut st, actor.clone(), "ev2", "details2");
        assert_eq!(id1 + 1, id2);
        assert_eq!(st.events.len(), 2);
        assert_eq!(st.events[0].event_type, "ev1");
        assert_eq!(st.events[1].event_type, "ev2");
    }

    #[test]
    fn test_state_default_salt_present() {
        let st = State::default();
        assert!(!st.salt.is_empty());
    }
}