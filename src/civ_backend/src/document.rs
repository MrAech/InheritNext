// src/civ_backend/src/document.rs
use candid::{CandidType, Deserialize};
use ic_cdk::export::Principal;
use ic_cdk_macros::{query, update};
use serde::Serialize;
use std::cell::RefCell;
use std::collections::BTreeMap;

/// Free tier document limit: 5 MiB
const FREE_TIER_DOC_LIMIT_BYTES: u64 = 5 * 1024 * 1024;

#[derive(CandidType, Serialize, Deserialize, Clone)]
pub struct DocumentMeta {
    /// Unique id chosen by client (e.g., uuid)
    pub id: String,
    /// SHA-256 hash of plaintext or ciphertext as chosen by client
    pub hash: Vec<u8>,
    /// Initialization vector or other encryption metadata (opaque)
    pub iv: Vec<u8>,
    /// Size in bytes of the uploaded ciphertext
    pub size: u64,
    /// Locator (e.g., blob store key, canister id, or URL). Frontend is responsible for uploading blob.
    pub locator: String,
    /// Optional description provided by owner
    pub description: Option<String>,
    /// uploaded timestamp (seconds since epoch)
    pub uploaded_at: u64,
}

#[derive(Default)]
struct DocState {
    /// owner -> list of document metadata
    docs: BTreeMap<Principal, Vec<DocumentMeta>>,
}

thread_local! {
    static DOC_STATE: RefCell<DocState> = RefCell::new(DocState::default());
}

/// Commit document metadata for the caller's document.
/// Enforces the free-tier 5MB quota per owner (sum of sizes).
#[update]
pub fn commit_blob(meta: DocumentMeta) -> Result<(), String> {
    let caller = ic_cdk::caller();
    let now_s = (ic_cdk::api::time() / 1_000_000_000) as u64;

    // basic sanity checks
    if meta.id.is_empty() {
        return Err("Document id cannot be empty".to_string());
    }
    if meta.locator.is_empty() {
        return Err("Locator cannot be empty".to_string());
    }

    DOC_STATE.with(|s| {
        let mut st = s.borrow_mut();
        let list = st.docs.entry(caller).or_default();

        // compute current usage
        let current_usage: u64 = list.iter().map(|d| d.size).sum();
        // if doc with same id exists, subtract its size (we allow overwrite)
        let existing_index = list.iter().position(|d| d.id == meta.id);
        let existing_size = existing_index.map(|i| list[i].size).unwrap_or(0);

        let new_usage = current_usage - existing_size + meta.size;
        if new_usage > FREE_TIER_DOC_LIMIT_BYTES {
            return Err(format!(
                "Free-tier document storage limit exceeded ({} bytes). Remove files or upgrade tier",
                FREE_TIER_DOC_LIMIT_BYTES
            ));
        }

        // remove existing with same id if present
        if let Some(idx) = existing_index {
            list.remove(idx);
        }

        let mut stored = meta.clone();
        // If uploaded_at not provided (0), set to now
        if stored.uploaded_at == 0 {
            stored.uploaded_at = now_s;
        }
        list.push(stored);
        Ok(())
    })
}

/// List committed document metadata for given owner (caller can query their own or others' metadata)
#[query]
pub fn list_blobs(owner: Principal) -> Vec<DocumentMeta> {
    DOC_STATE.with(|s| s.borrow().docs.get(&owner).cloned().unwrap_or_default())
}

/// Get single document metadata by id for owner
#[query]
pub fn get_blob(owner: Principal, id: String) -> Option<DocumentMeta> {
    DOC_STATE.with(|s| {
        s.borrow()
            .docs
            .get(&owner)
            .and_then(|list| list.iter().find(|d| d.id == id).cloned())
    })
}

/// Return current storage usage (bytes) for owner (sum of committed ciphertext sizes)
#[query]
pub fn get_owner_storage_usage(owner: Principal) -> u64 {
    DOC_STATE.with(|s| s.borrow().docs.get(&owner).map(|l| l.iter().map(|d| d.size).sum()).unwrap_or(0))
}