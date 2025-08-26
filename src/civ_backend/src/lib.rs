// (chunked document upload APIs added near document functions below)
// Public canister interface. Logic in modular api/; data structures split under models/.

mod api; // directory with modular submodules
mod audit;
mod crypto;
mod rng; // secure raw_rand-seeded ChaCha20 CSPRNG
mod models; // now a directory (was single file previously)
mod storage;
mod time;

use crate::api::{
    assets, ckbridge, custody, distributions, documents, escrow, executor, heirs, reconciliation,
};
// Re-export types needed in public interface for new on-chain approval helper
use crate::api::escrow::ApprovalSetOnChainInput;
use crate::models::*;

// There are still a few deprecated calls here so older generated bindings won't break.

#[ic_cdk_macros::update]
pub fn add_asset(new_asset: AssetInput) -> Result<(), CivError> {
    assets::add_asset(new_asset)
}

#[ic_cdk_macros::update]
pub fn add_heir(new_heir: HeirInput) -> Result<(), CivError> {
    heirs::add_heir(new_heir)
}

// Optional legacy distribution APIs (disabled by default). Enable with feature "legacy-distributions" during build.
#[cfg(feature = "legacy-distributions")]
#[deprecated(note = "Use set_asset_distributions/delete_distribution instead.")]
#[ic_cdk_macros::update]
pub fn assign_distributions(distributions_vec: Vec<AssetDistribution>) -> Result<(), CivError> {
    distributions::assign_distributions(distributions_vec)
}
#[cfg(feature = "legacy-distributions")]
#[deprecated(note = "Use get_asset_distributions instead.")]
#[ic_cdk::query]
pub fn get_distribution() -> Vec<(String, u64)> {
    distributions::get_distribution()
}
#[cfg(feature = "legacy-distributions")]
#[deprecated(note = "Use get_asset_distributions instead.")]
#[ic_cdk::query]
pub fn list_distributions() -> Vec<AssetDistribution> {
    distributions::list_distributions()
}

#[ic_cdk::query]
pub fn get_asset_distributions(asset_id: u64) -> Vec<AssetDistribution> {
    distributions::get_asset_distributions(asset_id)
}
#[ic_cdk::query]
pub fn get_asset_distributions_v2(asset_id: u64) -> Vec<DistributionShare> {
    distributions::get_asset_distributions_v2(asset_id)
}

#[ic_cdk_macros::update]
pub fn set_asset_distributions(
    asset_id: u64,
    dists: Vec<AssetDistribution>,
) -> Result<(), CivError> {
    distributions::set_asset_distributions(asset_id, dists)
}

#[ic_cdk_macros::update]
pub fn delete_distribution(asset_id: u64, heir_id: u64) -> Result<(), CivError> {
    distributions::delete_distribution(asset_id, heir_id)
}

#[ic_cdk::query]
pub fn get_timer() -> i64 {
    distributions::get_timer()
}

#[ic_cdk::query]
pub fn get_user() -> Option<User> {
    distributions::get_user()
}

#[ic_cdk::query]
pub fn list_assets() -> Vec<Asset> {
    assets::list_assets()
}

#[ic_cdk::query]
pub fn list_heirs() -> Vec<Heir> {
    heirs::list_heirs()
}
#[ic_cdk::query]
pub fn list_heirs_v2() -> Vec<HeirEx> {
    heirs::list_heirs_v2()
}

#[ic_cdk_macros::update]
pub fn remove_asset(asset_id: u64) -> Result<(), CivError> {
    assets::remove_asset(asset_id)
}

#[ic_cdk_macros::update]
pub fn remove_heir(heir_id: u64) -> Result<(), CivError> {
    heirs::remove_heir(heir_id)
}

#[ic_cdk_macros::update]
pub fn reset_timer() -> Result<(), CivError> {
    executor::reset_timer()
}

#[ic_cdk_macros::update]
pub fn update_asset(asset_id: u64, new_asset: AssetInput) -> Result<(), CivError> {
    assets::update_asset(asset_id, new_asset)
}

#[ic_cdk_macros::update]
pub fn update_asset_token_meta(asset_id: u64, meta: AssetTokenMetaInput) -> Result<(), CivError> {
    assets::update_asset_token_meta(asset_id, meta)
}

#[ic_cdk_macros::update]
pub fn update_heir(heir_id: u64, new_heir: HeirInput) -> Result<(), CivError> {
    heirs::update_heir(heir_id, new_heir)
}

// V2 APIs
#[ic_cdk_macros::update]
pub fn add_heir_v2(input: HeirAddInputV2) -> Result<u64, CivError> {
    heirs::add_heir_v2(input)
}
#[ic_cdk_macros::update]
pub fn set_distribution_v2(asset_id: u64, shares: Vec<DistributionShare>) -> Result<(), CivError> {
    distributions::set_distribution_v2(asset_id, shares)
}
#[ic_cdk_macros::update]
pub fn verify_heir_secret(heir_id: u64, secret_plain: String) -> Result<bool, CivError> {
    heirs::verify_heir_secret(heir_id, secret_plain)
}
#[ic_cdk_macros::update]
pub fn bind_heir_principal(heir_id: u64, principal: String) -> Result<(), CivError> {
    heirs::bind_heir_principal(heir_id, principal)
}
#[ic_cdk::query]
pub fn list_audit_log() -> Vec<AuditEvent> {
    distributions::list_audit_log()
}
#[ic_cdk::query]
pub fn list_audit_log_paged(offset: u64, limit: u64) -> Vec<AuditEvent> {
    // simple bounds & prune invocation
    distributions::prune_audit_log(2_000, 30 * 24 * 3600); // keep last 2000 or 30 days
    let off = offset as usize;
    let lim = limit.min(500) as usize; // cap page size
    distributions::list_audit_paged(off, lim)
}

#[ic_cdk::query]
pub fn list_audit_log_filtered(
    offset: u64,
    limit: u64,
    asset_id: Option<u64>,
    heir_id: Option<u64>,
) -> Vec<AuditEvent> {
    api::distributions::list_audit_filtered(offset, limit, asset_id, heir_id)
}

#[ic_cdk::query]
pub fn estate_readiness() -> api::ReadinessReport {
    distributions::estate_readiness()
}

#[ic_cdk::query]
pub fn metrics_snapshot() -> distributions::MetricsSnapshot {
    distributions::metrics_snapshot()
}
#[ic_cdk::query]
pub fn estate_status() -> EstateStatus {
    executor::estate_status()
}
#[ic_cdk_macros::update]
pub async fn execute_trigger() -> Result<(), CivError> {
    executor::execute_trigger().await
}
#[ic_cdk_macros::update]
pub fn lock_estate() -> Result<(), CivError> {
    executor::lock_estate()
}
#[ic_cdk_macros::update]
pub fn compute_ledger_attestation() -> Result<Vec<u8>, CivError> {
    executor::compute_ledger_attestation()
}
#[ic_cdk_macros::update]
pub fn add_document(input: DocumentAddInput) -> Result<u64, CivError> {
    documents::add_document(input)
}
#[ic_cdk::query]
pub fn list_documents() -> Vec<DocumentEntry> {
    documents::list_documents()
}
#[ic_cdk::query]
pub fn heir_get_document(
    heir_id: u64,
    doc_id: u64,
) -> Result<Option<(DocumentEntry, Vec<u8>)>, CivError> {
    documents::heir_get_document(heir_id, doc_id)
}
#[ic_cdk_macros::update]
pub fn start_document_upload(init: DocumentUploadInit) -> Result<u64, CivError> {
    documents::start_document_upload(init)
}
#[ic_cdk_macros::update]
pub fn upload_document_chunk(chunk: DocumentChunk) -> Result<u64, CivError> {
    documents::upload_document_chunk(chunk)
}
#[ic_cdk_macros::update]
pub fn finalize_document_upload(upload_id: u64) -> Result<u64, CivError> {
    documents::finalize_document_upload(upload_id)
}
#[ic_cdk_macros::update]
pub fn abort_document_upload(upload_id: u64, reason: String) -> Result<(), CivError> {
    documents::abort_document_upload(upload_id, reason)
}
// Notification scaffold public APIs
#[ic_cdk_macros::update]
pub fn enqueue_notification(
    channel: String,
    template: String,
    payload: String,
) -> Result<u64, CivError> {
    use crate::models::NotificationChannel as NC;
    let ch = match channel.to_ascii_lowercase().as_str() {
        "email" => NC::Email,
        "sms" => NC::Sms,
        _ => return Err(CivError::Other("invalid_channel".into())),
    };
    api::notify::enqueue_notification(ch, template, payload)
}
#[ic_cdk::query]
pub fn list_notifications() -> Vec<NotificationRecord> {
    let caller = api::common::user_id();
    crate::storage::USERS.with(|users| {
        users
            .borrow()
            .get(&caller)
            .map(|u| u.notifications.clone())
            .unwrap_or_default()
    })
}
#[ic_cdk::query]
pub fn custody_subaccount_for_heir(heir_id: u64) -> Result<Vec<u8>, CivError> {
    custody::custody_subaccount_for_heir(heir_id)
}
#[ic_cdk_macros::update]
pub fn heir_claim(input: api::HeirClaimInput) -> Result<api::HeirClaimResult, CivError> {
    heirs::heir_claim(input)
}
#[ic_cdk_macros::update]
pub fn heir_set_payout_preference_session(
    session_id: u64,
    asset_id: u64,
    preference: PayoutPreference,
) -> Result<(), CivError> {
    heirs::heir_set_payout_preference_session(session_id, asset_id, preference)
}
// Session onboarding APIs (claim link flow)
#[ic_cdk_macros::update]
pub fn create_claim_link(heir_id: u64) -> Result<ClaimLinkUnsealed, CivError> {
    heirs::create_claim_link(heir_id)
}
#[ic_cdk_macros::update]
pub fn heir_begin_claim(link_id: u64, code_plain: String) -> Result<u64, CivError> {
    heirs::heir_begin_claim(link_id, code_plain)
}
#[ic_cdk_macros::update]
pub fn heir_verify_secret_session(session_id: u64, secret_plain: String) -> Result<bool, CivError> {
    heirs::heir_verify_secret_session(session_id, secret_plain)
}
#[ic_cdk_macros::update]
pub fn heir_bind_principal_session(session_id: u64, principal: String) -> Result<(), CivError> {
    heirs::heir_bind_principal_session(session_id, principal)
}
// Optional identity claim verification
#[ic_cdk_macros::update]
pub fn heir_verify_identity_session(
    session_id: u64,
    identity_claim: String,
) -> Result<bool, CivError> {
    heirs::heir_verify_identity_session(session_id, identity_claim)
}
// Custody withdraw (post-execution principal-bound release)
#[ic_cdk_macros::update]
pub fn withdraw_custody(asset_id: u64, heir_id: u64) -> Result<TransferRecord, CivError> {
    custody::withdraw_custody(asset_id, heir_id)
}
#[allow(dead_code)] // Admin dashboard (pending) – manual trigger of custody reconciliation audit snapshot
#[ic_cdk_macros::update]
pub async fn reconcile_custody() -> Vec<crate::models::custody::CustodyReconEntry> {
    reconciliation::reconcile_custody().await
}
#[ic_cdk::query]
pub fn get_custody_reconciliation() -> Option<Vec<crate::models::custody::CustodyReconEntry>> {
    reconciliation::get_custody_reconciliation()
}
// Optional lifecycle helpers
#[ic_cdk_macros::update]
pub fn start_warning() -> Result<(), CivError> {
    executor::start_warning()
}
#[ic_cdk_macros::update]
pub fn perform_maintenance() -> Result<(), CivError> {
    executor::perform_maintenance()
}
#[ic_cdk::query]
pub fn list_transfers() -> Vec<TransferRecord> {
    executor::list_transfers()
}
#[ic_cdk::query]
pub fn last_execution_summary() -> Option<ExecutionSummary> {
    executor::last_execution_summary()
}
#[ic_cdk::query]
pub fn list_ck_withdraws() -> Vec<CkWithdrawRecord> {
    ckbridge::list_ck_withdraws()
}
#[ic_cdk_macros::update]
pub fn request_ck_withdraw(session_id: u64, asset_id: u64, heir_id: u64) -> Result<(), CivError> {
    ckbridge::request_ck_withdraw(session_id, asset_id, heir_id)
}
#[ic_cdk_macros::update]
pub async fn submit_ck_withdraw(
    session_id: u64,
    asset_id: u64,
    heir_id: u64,
    l1_address: String,
) -> Result<BridgeTxInfo, CivError> {
    ckbridge::submit_ck_withdraw(session_id, asset_id, heir_id, l1_address).await
}
#[ic_cdk_macros::update]
pub async fn poll_ck_withdraw(
    session_id: u64,
    asset_id: u64,
    heir_id: u64,
) -> Result<(), CivError> {
    ckbridge::poll_ck_withdraw(session_id, asset_id, heir_id).await
}

// Escrow & Approval public APIs
#[ic_cdk_macros::update]
pub async fn deposit_escrow(input: api::EscrowDepositInput) -> Result<(), CivError> {
    escrow::deposit_escrow(input).await
}
#[ic_cdk::query]
pub fn list_escrow() -> Vec<EscrowRecord> {
    escrow::list_escrow()
}
#[ic_cdk_macros::update]
pub fn withdraw_escrow(asset_id: u64) -> Result<(), CivError> {
    escrow::withdraw_escrow(asset_id)
}
#[ic_cdk_macros::update]
pub async fn withdraw_escrow_icrc1(asset_id: u64, amount: Option<u128>) -> Result<u128, CivError> {
    escrow::withdraw_escrow_icrc1(crate::api::escrow::EscrowWithdrawOnChainInput {
        asset_id,
        amount,
    })
    .await
}
#[ic_cdk_macros::update]
pub fn approval_set(input: api::ApprovalSetInput) -> Result<(), CivError> {
    escrow::approval_set(input)
}
#[ic_cdk_macros::update]
pub fn approval_revoke(asset_id: u64) -> Result<(), CivError> {
    escrow::approval_revoke(asset_id)
}
#[ic_cdk::query]
pub fn list_approvals() -> Vec<ApprovalRecord> {
    escrow::list_approvals()
}
#[ic_cdk_macros::update]
pub async fn approval_set_icrc2(input: ApprovalSetOnChainInput) -> Result<(), CivError> {
    escrow::approval_set_icrc2(input).await
}

#[ic_cdk_macros::pre_upgrade]
fn pre_upgrade() {
    executor::pre_upgrade();
}
#[ic_cdk_macros::post_upgrade]
fn post_upgrade() {
    executor::post_upgrade();
}

// Ensure RNG seeded on fresh install
#[ic_cdk_macros::init]
fn init() {
}

// Provide explicit initialization entrypoint to avoid calling management canister
// APIs during the synchronous `init`/install phase which is not permitted.
#[ic_cdk_macros::update]
pub fn initialize_rng() {
    // Schedule async init outside install mode
    ic_cdk::spawn(rng::init_rng());
}

#[ic_cdk::query]
pub fn rng_ready() -> bool {
    rng::is_initialized()
}

// Removed duplicate verify_heir_secret, bind_heir_principal, list_audit_events definitions (already exposed above).

// Returns an integrity report for the caller's data, including invariants and allocation health.
#[ic_cdk::query]
pub fn check_integrity() -> IntegrityReport {
    distributions::check_integrity()
}

#[ic_cdk::query(name = "__get_candid_interface_tmp_hack")]
fn export_did() -> String {
    candid::export_service!();
    __export_service()
}

// remember to regenerate candid.
