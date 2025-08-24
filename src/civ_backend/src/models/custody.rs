use candid::{CandidType, Deserialize};

#[derive(Clone, CandidType, Deserialize)]
pub struct CustodyRecord {
    pub heir_id: u64,
    pub subaccount: Vec<u8>,
}

#[derive(Clone, CandidType, Deserialize)]
pub struct EscrowRecord {
    pub asset_id: u64,
    pub amount: Option<u128>,
    pub token_id: Option<u64>,
    pub deposited_at: u64,
}

#[derive(Clone, CandidType, Deserialize)]
pub struct ApprovalRecord {
    pub asset_id: u64,
    pub allowance: Option<u128>,
    pub token_id: Option<u64>,
    pub granted_at: u64,
}

#[derive(Clone, CandidType, Deserialize)]
pub struct NftCustodyRecord {
    pub asset_id: u64,
    pub heir_id: u64,
    pub token_id: u64,
    pub staged_at: u64,
    pub released_at: Option<u64>,
    pub attempts: u32,
    pub last_error: Option<String>,
    pub releasing: bool,
    pub next_attempt_after: Option<u64>,
}

#[derive(Clone, CandidType, Deserialize)]
pub struct FungibleCustodyRecord {
    pub asset_id: u64,
    pub heir_id: u64,
    pub amount: u128,
    pub staged_at: u64,
    pub released_at: Option<u64>,
    pub attempts: u32,
    pub last_error: Option<String>,
    pub releasing: bool,
    pub next_attempt_after: Option<u64>,
}

#[derive(Copy, Clone, CandidType, Deserialize)]
pub enum ReconStatus {
    Exact,
    Shortfall,
    Surplus,
    QueryError,
}

#[derive(Clone, CandidType, Deserialize)]
pub struct CustodyReconEntry {
    pub asset_id: u64,
    pub heir_id: u64,
    pub on_chain: Option<u128>,
    pub staged_sum: u128,
    pub delta: Option<i128>,
    pub status: ReconStatus,
    pub last_checked: u64,
}

#[derive(Clone, CandidType, Deserialize)]
pub struct EscrowReconEntry {
    pub asset_id: u64,
    pub on_chain: Option<u128>,
    pub logical_remaining: u128,
    pub delta: Option<i128>,
    pub status: ReconStatus,
    pub last_checked: u64,
}

// Escrow auto-management thresholds (amounts are in smallest token units; tune via governance later)
pub const ESCROW_TOP_UP_MIN_SHORTFALL: u128 = 10; // if shortfall >= this, schedule auto top-up
pub const ESCROW_RECLAIM_MIN_SURPLUS: u128 = 10; // if surplus >= this, schedule reclaim
pub const ESCROW_AUTO_ACTION_COOLDOWN_SECS: u64 = 6 * 3600; // avoid thrashing (6h)
