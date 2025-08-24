use super::base::PayoutPreference;
use candid::{CandidType, Deserialize};

// Structured transfer error taxonomy (schema v5+).
// We retain the legacy `error: Option<String>` field on `TransferRecord` for backward
// compatibility / human detail, while new logic should rely on `error_kind`.
#[derive(Clone, CandidType, Deserialize, Debug)]
pub enum TransferErrorKind {
    MissingApproval,
    AllowanceNotFoundOnChain,
    InvalidOwnerPrincipal,
    MissingDestinationPrincipal,
    NftDip721,
    NftExt,
    NftUnsupported,
    TransferCallFailed,
    Other,
}

impl TransferErrorKind {
    pub fn from_legacy(code: &str) -> (Self, Option<String>) {
        match code {
            // Normalized legacy markers produced by prior map_error_code()
            "ERR_MISSING_APPROVAL" => (Self::MissingApproval, None),
            "ERR_ALLOWANCE_CHAIN_MISSING" => (Self::AllowanceNotFoundOnChain, None),
            "ERR_INVALID_OWNER_PRINCIPAL" => (Self::InvalidOwnerPrincipal, None),
            "ERR_MISSING_DESTINATION" => (Self::MissingDestinationPrincipal, None),
            c if c.starts_with("NFT_DIP721:") => (Self::NftDip721, Some(c[11..].to_string())),
            c if c.starts_with("NFT_EXT:") => (Self::NftExt, Some(c[8..].to_string())),
            c if c.starts_with("NFT_UNSUPPORTED:") => {
                (Self::NftUnsupported, Some(c[15..].to_string()))
            }
            c if c.starts_with("call failed") || c.starts_with("decode err") => {
                (Self::TransferCallFailed, Some(code.to_string()))
            }
            _ => (Self::Other, Some(code.to_string())),
        }
    }
}

#[derive(Clone, CandidType, Deserialize)]
pub enum TransferKind {
    Fungible,
    Nft,
    Document,
}

#[derive(Clone, CandidType, Deserialize)]
pub struct TransferRecord {
    pub id: u64,
    pub timestamp: u64,
    pub asset_id: Option<u64>,
    pub heir_id: Option<u64>,
    pub kind: TransferKind,
    pub amount: Option<u128>,
    pub payout_preference: Option<PayoutPreference>,
    pub note: Option<String>,
    pub tx_index: Option<u128>,
    // Legacy string detail (may be None for new records or contain message / raw code).
    pub error: Option<String>,
    // Structured error classification (schema v5+).
    pub error_kind: Option<TransferErrorKind>,
}

#[derive(Clone, CandidType, Deserialize, Debug)]
pub struct LedgerAttestation {
    pub merkle_root: Vec<u8>,
    pub computed_at: u64,
    pub transfer_count: u64,
}
