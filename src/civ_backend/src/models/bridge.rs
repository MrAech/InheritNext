use super::errors::BridgeErrorInfo;
use candid::{CandidType, Deserialize};

// BridgeStatus extended to capture richer lifecycle:
// - FeeQuoted: fee fetched but submission not yet attempted
// - InProgress: submission accepted and awaiting finalization
// - Reimbursed: chain refunded (treat as terminal distinct from generic Failed)
#[derive(Clone, CandidType, Deserialize)]
pub enum BridgeStatus {
    Staged,
    Requested,
    FeeQuoted,
    Submitted,
    InProgress,
    Completed,
    Reimbursed,
    Failed(String),
}

#[derive(Clone, CandidType, Deserialize)]
pub struct CkWithdrawRecord {
    pub asset_id: u64,
    pub heir_id: u64,
    pub amount: u128,
    pub staged_at: u64,
    pub requested_at: Option<u64>,
    pub completed_at: Option<u64>,
    pub tx_index: Option<u128>,
    pub bridge_status: Option<BridgeStatus>,
    pub bridge_tx_id: Option<String>,
    pub bridge_error: Option<BridgeErrorInfo>,
    pub tx_hash: Option<String>,
    pub effective_fee: Option<u128>,
    pub quoted_fee: Option<u128>,
}

#[derive(Clone, CandidType, Deserialize)]
pub struct BridgeTxInfo {
    pub asset_id: u64,
    pub heir_id: u64,
    pub l1_address: String,
    pub submitted_at: u64,
    pub tx_id: Option<String>,
    pub consecutive_misses: Option<u32>,
    pub notfound_terminal: Option<bool>,
}
