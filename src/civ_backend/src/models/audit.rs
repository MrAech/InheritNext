use super::base::PayoutPreference;
use super::bridge::BridgeStatus;
use super::user::EstatePhase;
use candid::{CandidType, Deserialize};

#[derive(Clone, CandidType, Deserialize)]
pub enum AuditEventKind {
    UserCreated,
    AssetAdded {
        asset_id: u64,
    },
    AssetUpdated {
        asset_id: u64,
    },
    AssetMetadataUpdated {
        asset_id: u64,
    },
    AssetRemoved {
        asset_id: u64,
    },
    HeirAdded {
        heir_id: u64,
    },
    HeirUpdated {
        heir_id: u64,
    },
    HeirRemoved {
        heir_id: u64,
    },
    DistributionSet {
        asset_id: u64,
    },
    DistributionDeleted {
        asset_id: u64,
        heir_id: u64,
    },
    TimerReset,
    TriggerExecuted,
    HeirSecretVerified {
        heir_id: u64,
    },
    HeirPrincipalBound {
        heir_id: u64,
    },
    PhaseChanged {
        from: EstatePhase,
        to: EstatePhase,
    },
    EscrowDeposited {
        asset_id: u64,
        amount: Option<u128>,
    },
    EscrowWithdrawn {
        asset_id: u64,
        amount: Option<u128>,
    },
    EscrowReleaseAttempt {
        asset_id: u64,
        heir_id: u64,
        amount: u128,
        attempt: u32,
    },
    EscrowReleased {
        asset_id: u64,
        heir_id: u64,
        amount: u128,
    },
    EscrowReleaseFailed {
        asset_id: u64,
        heir_id: u64,
        amount: u128,
        attempt: u32,
        error: String,
    },
    ApprovalSet {
        asset_id: u64,
    },
    ApprovalRevoked {
        asset_id: u64,
    },
    DocAccessed {
        doc_id: u64,
        heir_id: u64,
    },
    DocUploadStarted {
        upload_id: u64,
        name: String,
    },
    DocUploadChunkAppended {
        upload_id: u64,
        bytes: u64,
    },
    DocUploadFinalized {
        upload_id: u64,
        doc_id: u64,
        size: u64,
    },
    DocUploadAborted {
        upload_id: u64,
        reason: String,
    },
    CkWithdrawStaged {
        asset_id: u64,
        heir_id: u64,
        amount: u128,
    },
    CkWithdrawRequested {
        asset_id: u64,
        heir_id: u64,
    },
    CkWithdrawSubmitted {
        asset_id: u64,
        heir_id: u64,
    },
    CkWithdrawFeeQuoted {
        asset_id: u64,
        heir_id: u64,
        fee: u128,
    },
    CkWithdrawCompleted {
        asset_id: u64,
        heir_id: u64,
    },
    CkWithdrawReimbursed {
        asset_id: u64,
        heir_id: u64,
    },
    CkWithdrawFailed {
        asset_id: u64,
        heir_id: u64,
        error: String,
    },
    CustodyWithdrawExecuted {
        asset_id: u64,
        heir_id: u64,
    },
    ClaimLinkCreated {
        heir_id: u64,
        link_id: u64,
    },
    HeirSessionStarted {
        heir_id: u64,
        session_id: u64,
    },
    HeirSessionSecretVerified {
        heir_id: u64,
        session_id: u64,
    },
    HeirPayoutPreferenceSet {
        heir_id: u64,
        asset_id: u64,
        from: PayoutPreference,
        to: PayoutPreference,
    },
    NftCustodyStaged {
        asset_id: u64,
        heir_id: u64,
        token_id: u64,
    },
    NftCustodyReleaseAttempt {
        asset_id: u64,
        heir_id: u64,
        token_id: u64,
        attempt: u32,
    },
    NftCustodyReleased {
        asset_id: u64,
        heir_id: u64,
        token_id: u64,
    },
    NftCustodyReleaseFailed {
        asset_id: u64,
        heir_id: u64,
        token_id: u64,
        attempt: u32,
        error: String,
    },
    FungibleCustodyStaged {
        asset_id: u64,
        heir_id: u64,
        amount: u128,
    },
    FungibleCustodyReleaseAttempt {
        asset_id: u64,
        heir_id: u64,
        amount: u128,
        attempt: u32,
    },
    FungibleCustodyReleased {
        asset_id: u64,
        heir_id: u64,
        amount: u128,
    },
    FungibleCustodyReleaseFailed {
        asset_id: u64,
        heir_id: u64,
        amount: u128,
        attempt: u32,
        error: String,
    },
    CustodyReconciliationDiscrepancy {
        heir_id: u64,
        expected_total: u128,
        note: String,
    },
    EscrowReconciliationDiscrepancy {
        asset_id: u64,
        expected_total: u128,
        note: String,
    },
    RetryAttempt {
        retry_id: u64,
        attempt: u32,
        kind: String,
    },
    RetryTerminal {
        retry_id: u64,
        attempts: u32,
        kind: String,
    },
    RetrySucceeded {
        retry_id: u64,
        attempts: u32,
        kind: String,
    },
    AuditPruned {
        removed: u64,
        remaining: u64,
    },
    HeirSecretAttemptRateLimited {
        heir_id: u64,
        attempts: u32,
    },
    HeirSecretBackoffRateLimited {
        heir_id: u64,
        attempts: u32,
        wait_secs: u64,
    },
    NotificationQueued {
        channel: String,
        template: String,
    },
    NotificationSent {
        channel: String,
        template: String,
        success: bool,
    },
    EscrowAutoTopUp {
        asset_id: u64,
        amount: u128,
    },
    EscrowAutoReclaim {
        asset_id: u64,
        amount: u128,
    },
    LedgerAttested {
        merkle_root: Vec<u8>,
    },
    HeirSessionExpired {
        heir_id: u64,
        session_id: u64,
    },
    BridgePollNotFoundTerminal {
        asset_id: u64,
        heir_id: u64,
    },
    HeirPayoutPreferenceRateLimited {
        heir_id: u64,
        asset_id: u64,
    },
}

#[derive(Clone, CandidType, Deserialize)]
pub struct AuditEvent {
    pub id: u64,
    pub timestamp: u64,
    pub kind: AuditEventKind,
}
