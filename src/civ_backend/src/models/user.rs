use super::asset::Asset;
use super::audit::AuditEvent;
use super::bridge::{BridgeTxInfo, CkWithdrawRecord};
use super::custody::{
    ApprovalRecord, CustodyReconEntry, CustodyRecord, EscrowReconEntry, EscrowRecord,
    FungibleCustodyRecord, NftCustodyRecord,
};
use super::distribution::{AssetDistribution, DistributionShare, HeirPayoutOverride};
use super::document::DocumentEntry;
use super::heir::{Heir, HeirEx};
use super::integrity::{ClaimLink, HeirSession};
use super::payout::TransferRecord;
use candid::{CandidType, Deserialize};

#[derive(Clone, CandidType, Deserialize, PartialEq, Eq, Debug)]
pub enum NotificationChannel {
    Email,
    Sms,
}

#[derive(Clone, CandidType, Deserialize, Debug)]
pub struct NotificationRecord {
    pub id: u64,
    pub channel: NotificationChannel,
    pub template: String,
    pub payload: String,
    pub queued_at: u64,
    pub sent_at: Option<u64>,
    pub success: Option<bool>,
    pub attempts: u32,
}

#[derive(Clone, CandidType, Deserialize, Debug)]
pub struct MetricsFrame {
    pub ts: u64,
    pub retry_counts: std::collections::HashMap<String, u64>,
    pub custody_totals: std::collections::HashMap<u64, u128>,
    pub escrow_totals: std::collections::HashMap<u64, u128>,
    pub custody_discrepancies: u64,
    pub escrow_discrepancies: u64,
}

#[derive(Clone, CandidType, Deserialize, PartialEq, Eq)]
pub enum EstatePhase {
    Draft,
    Warning,
    Locked,
    Executed,
}

#[derive(Clone, CandidType, Deserialize)]
pub struct ExecutionSummary {
    pub started_at: u64,
    pub finished_at: u64,
    pub total_items: u64,
    pub success_count: u64,
    pub failure_count: u64,
    pub skipped_count: u64,
    pub ck_staged_count: u64,
    pub auto: bool,
}

#[derive(Clone, CandidType, Deserialize)]
pub struct EstateStatus {
    pub phase: EstatePhase,
    pub seconds_to_expiry: i64,
    pub warning_started_at: Option<u64>,
    pub locked_at: Option<u64>,
    pub executed_at: Option<u64>,
}

#[derive(Clone, CandidType, Deserialize, Debug)]
pub struct PayoutOverrideRate {
    pub heir_id: u64,
    pub asset_id: u64,
    pub day_epoch: u64,
    pub count: u32,
    pub last_set_at: u64,
}

#[derive(Clone, CandidType, Deserialize, Debug)]
pub struct ReadinessCache {
    pub computed_at: u64,
    pub ready: bool,
    pub issues: Vec<String>,
}

#[derive(Clone, CandidType, Deserialize)]
pub struct User {
    pub user: String,
    pub assets: Vec<Asset>,
    pub heirs: Vec<Heir>,
    pub distributions: Vec<AssetDistribution>,
    pub timer_expiry: u64,
    pub distributed: bool,
    pub last_timer_reset: u64,
    pub distributions_v2: Vec<DistributionShare>,
    pub heirs_v2: Vec<HeirEx>,
    pub documents: Vec<DocumentEntry>,
    pub custody: Vec<CustodyRecord>,
    pub escrow: Vec<EscrowRecord>,
    pub approvals: Vec<ApprovalRecord>,
    pub ck_withdraws: Vec<CkWithdrawRecord>,
    pub audit_log: Vec<AuditEvent>,
    pub phase: EstatePhase,
    pub warning_started_at: Option<u64>,
    pub locked_at: Option<u64>,
    pub executed_at: Option<u64>,
    pub transfers: Vec<TransferRecord>,
    pub doc_master_key: Option<Vec<u8>>,
    pub claim_links: Vec<ClaimLink>,
    pub sessions: Vec<HeirSession>,
    pub bridge_txs: Vec<BridgeTxInfo>,
    pub payout_overrides: Vec<HeirPayoutOverride>,
    pub execution_nonce: Option<u64>,
    pub last_execution_summary: Option<ExecutionSummary>,
    pub nft_custody: Vec<NftCustodyRecord>,
    pub fungible_custody: Vec<FungibleCustodyRecord>,
    pub retry_queue: Option<Vec<crate::api::retry::RetryItem>>,
    pub schema_version: u16,
    pub custody_recon: Option<Vec<CustodyReconEntry>>,
    pub escrow_recon: Option<Vec<EscrowReconEntry>>,
    pub notifications: Vec<NotificationRecord>,
    pub metrics_history: Vec<MetricsFrame>,
    pub retry_adaptive:
        Option<std::collections::HashMap<String, crate::api::retry::AdaptiveKindStats>>,
    pub payout_override_rates: Option<Vec<PayoutOverrideRate>>,
    pub doc_uploads: Option<Vec<crate::models::document::DocumentUploadSession>>,
    pub readiness_cache: Option<ReadinessCache>,
    pub ledger_attestation: Option<crate::models::payout::LedgerAttestation>,
    pub audit_prune_in_progress: bool,
}

pub const CURRENT_SCHEMA_VERSION: u16 = 9; // v9 adds retry_adaptive stats map
impl User {
    pub fn new(principal: &str, timer_expiry: u64) -> Self {
        User {
            user: principal.to_string(),
            assets: vec![],
            heirs: vec![],
            distributions: vec![],
            timer_expiry,
            distributed: false,
            last_timer_reset: 0,
            distributions_v2: vec![],
            heirs_v2: vec![],
            documents: vec![],
            custody: vec![],
            escrow: vec![],
            approvals: vec![],
            ck_withdraws: vec![],
            audit_log: vec![],
            phase: EstatePhase::Draft,
            warning_started_at: None,
            locked_at: None,
            executed_at: None,
            transfers: vec![],
            doc_master_key: None,
            claim_links: vec![],
            sessions: vec![],
            bridge_txs: vec![],
            payout_overrides: vec![],
            execution_nonce: None,
            last_execution_summary: None,
            nft_custody: vec![],
            fungible_custody: vec![],
            retry_queue: None,
            schema_version: CURRENT_SCHEMA_VERSION,
            custody_recon: None,
            escrow_recon: None,
            notifications: vec![],
            metrics_history: vec![],
            retry_adaptive: None,
            payout_override_rates: None,
            doc_uploads: None,
            readiness_cache: None,
            ledger_attestation: None,
            audit_prune_in_progress: false,
        }
    }
}

pub const INACTIVITY_PERIOD_SECS: u64 = 30 * 24 * 60 * 60;
pub const WARNING_WINDOW_SECS: u64 = 7 * 24 * 60 * 60;
