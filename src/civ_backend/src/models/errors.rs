use super::base::AssetKind;
use candid::{CandidType, Deserialize};

#[derive(CandidType, Deserialize, Clone, Debug)]
pub enum BridgeErrorKind {
    FeeShortfall,
    Rejected,
    Timeout,
    Network,
    InvalidCanister,
    Reimbursed,
    RateLimited,
    UnauthorizedChain,
    Other,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct BridgeErrorInfo {
    pub kind: BridgeErrorKind,
    pub message: String,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
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
    SecretInvalid,
    InvalidPayoutPreference,
    EstateLocked,
    EstateNotReady,
    TransferFailed,
    AlreadyExecuted,
    AlreadyTransferred,
    Unauthorized,
    SessionExpired,
    MissingApproval,
    AllowanceNotFoundOnChain,
    AllowanceInsufficient { needed: u128, found: u128 },
    InvalidOwnerPrincipal,
    TransferCallFailed(String),
    NftStandardUnsupported(String),
    NftTransferFailed(String),
    Bridge(BridgeErrorInfo),
    RateLimited,
    ReadinessCacheStale,
    ReadinessIssue(String),
    HeirSessionUnauthorized,
}

pub fn infer_asset_kind(asset_type: &str) -> AssetKind {
    let t = asset_type.to_ascii_lowercase();
    if t == "doc" || t == "document" {
        return AssetKind::Document;
    }
    if t == "nft" || t.contains("dip721") {
        return AssetKind::Nft;
    }
    if t.starts_with("ckbtc") || t.starts_with("cketh") {
        return AssetKind::ChainWrapped;
    }
    AssetKind::Fungible
}

// Convenience helpers for bridge errors
impl CivError {
    pub fn bridge_err(kind: BridgeErrorKind, msg: impl Into<String>) -> Self {
        CivError::Bridge(BridgeErrorInfo {
            kind,
            message: msg.into(),
        })
    }
}
