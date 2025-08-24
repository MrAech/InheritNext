use candid::{CandidType, Deserialize};

#[derive(Clone, CandidType, Deserialize)]
pub struct ClaimLink {
    pub id: u64,
    pub heir_id: u64,
    pub code_hash: Vec<u8>,
    pub salt: Vec<u8>,
    pub created_at: u64,
    pub used: bool,
}

#[derive(Clone, CandidType, Deserialize)]
pub struct ClaimLinkUnsealed {
    pub link_id: u64,
    pub code_plain: String,
}

#[derive(Clone, CandidType, Deserialize)]
pub struct HeirSession {
    pub id: u64,
    pub heir_id: u64,
    pub started_at: u64,
    pub verified_secret: bool,
    pub bound_principal: bool,
    pub expires_at: u64,
}

#[derive(Clone, CandidType, Deserialize)]
pub struct IntegrityReport {
    pub asset_count: u64,
    pub distribution_count: u64,
    pub over_allocated_assets: Vec<u64>,
    pub fully_allocated_assets: Vec<u64>,
    pub partially_allocated_assets: Vec<u64>,
    pub unallocated_assets: Vec<u64>,
    pub issues: Vec<String>,
}
