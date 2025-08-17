use candid::{CandidType, Deserialize};

// A single asset the user has registered (house, stock, collectible, etc.).
// All times are stored in SECONDS (not nanos) to keep arithmetic sane.
#[derive(Clone, CandidType, Deserialize)]
pub struct Asset {
    pub id: u64,
    pub name: String,
    pub asset_type: String,
    pub value: u64,
    pub description: String,
    pub created_at: u64,
    pub updated_at: u64,
}
// Payload the frontend sends when creating or editing an asset.
// ID + timestamps get assigned server-side; keeps the public API clean.
#[derive(Clone, CandidType, Deserialize)]
pub struct AssetInput {
    pub name: String,
    pub asset_type: String,
    pub value: u64,
    pub description: String,
}


// Raw input for a new (or updated) heir record; again server fills id + times.
#[derive(Clone, CandidType, Deserialize)]
pub struct HeirInput {
    pub name: String,
    pub relationship: String,
    pub email: String,
    pub phone: String,
    pub address: String,
}


// One slice of an inheritance pie: how much (percentage) of a given asset a specific heir gets.
// Percent is u8 because we only allow 0..=100 and we sum per asset.
#[derive(Clone, CandidType, Deserialize)]
pub struct AssetDistribution {
    pub asset_id: u64,
    pub heir_id: u64,
    pub percentage: u8,
}


// A stored heir / beneficiary. Contact fields are plain strings on purpose (no validation regex here; keep canister small).
#[derive(Clone, CandidType, Deserialize)]
pub struct Heir {
    pub id: u64,
    pub name: String,
    pub relationship: String,
    pub email: String,
    pub phone: String,
    pub address: String,
    pub created_at: u64,
    pub updated_at: u64,
}


// Everything we keep per user (principal). In-memory only right now. @sharmayash2805 @MrAech need to consiter stable mem.
// timer_expiry: when the inactivity timer would trigger (seconds since epoch).
// distributed: flag so we don't auto-clear twice if user returns after expiry. highly unlikely unless manually distributied <- the future 
#[derive(Clone, CandidType, Deserialize)]
pub struct User {
    pub user: String,
    pub assets: Vec<Asset>,
    pub heirs: Vec<Heir>,
    pub distributions: Vec<AssetDistribution>,
    pub timer_expiry: u64,
    pub distributed: bool,
}

// Quick snapshot to let the UI (or a dev) see if allocations look healthy.
// Not over‑engineering: just enough fields to answer "did I assign things sensibly?". cause its breaking too much 
#[derive(Clone, CandidType, Deserialize)]
pub struct IntegrityReport {
    // Total number of assets examined.
    pub asset_count: u64,
    // Number of distribution entries examined.
    pub distribution_count: u64,
    // Assets whose summed percentages exceeded 100.
    pub over_allocated_assets: Vec<u64>,
    // Assets whose summed percentages are exactly 100.
    pub fully_allocated_assets: Vec<u64>,
    // Assets with partial (0 < sum < 100) allocation.
    pub partially_allocated_assets: Vec<u64>,
    // Assets with no distribution entries.
    pub unallocated_assets: Vec<u64>,
    // Free-form issues discovered (e.g., references to non-existent heirs/assets).
    pub issues: Vec<String>,
}


// Error types returned to the frontend.
// Some variants (like *Exists) are included for future uniqueness checks.
#[derive(CandidType, Deserialize)]
pub enum CivError {
    AssetExists,
    AssetNotFound,
    HeirExists,
    HeirNotFound,
    UserNotFound,
    InvalidHeirPercentage,
    Other(String),
    DistributionAssetNotFound,
    DistributionHeirNotFound,
}
