use super::base::{HoldingMode, AssetKind, PayoutPreference};
use candid::{CandidType, Deserialize};

#[derive(Clone, CandidType, Deserialize)]
pub enum ChainWrappedKind {
    CkBtc,
    CkEth,
}

#[derive(Clone, CandidType, Deserialize)]
pub enum NftStandard {
    Dip721,
    Ext,
    Other(String),
}

/// Canonical asset record stored per-user.
/// Fields that are not relevant for a particular AssetKind should be None/null.
#[derive(Clone, CandidType, Deserialize)]
pub struct Asset {
    pub id: u64,
    pub name: String,
    pub asset_type: String, // human-facing type label (e.g. "Fungible", "NFT", "Document", ...)
    pub kind: AssetKind,
    pub value: u64, // estimated monetary value in smallest units (frontend uses integer currency)
    pub decimals: Option<u8>, // for fungible tokens / chain wrapped
    pub description: String,
    pub created_at: u64,
    pub updated_at: u64,
    pub token_canister: Option<String>, // optional canister id for tokens/NFTs
    pub token_id: Option<u64>, // token id for NFTs or token-specific id when applicable
    pub holding_mode: Option<HoldingMode>, // escrow / approval
    pub nft_standard: Option<NftStandard>,
    pub chain_wrapped: Option<ChainWrappedKind>,
    pub file_path: Option<String>, // for Document assets store uploaded file path (temporary)
}

#[derive(Clone, CandidType, Deserialize)]
pub struct AssetInput {
    pub name: String,
    pub asset_type: String,
    pub kind: AssetKind,
    pub value: Option<u64>,
    pub decimals: Option<u8>,
    pub description: String,
    // optional upfront metadata; token specifics are set via update_asset_token_meta
    pub token_canister: Option<String>,
    pub token_id: Option<u64>,
    pub file_path: Option<String>,
    pub holding_mode: Option<HoldingMode>,
}

#[derive(Clone, CandidType, Deserialize)]
pub struct AssetTokenMetaInput {
    pub token_canister: Option<String>,
    pub token_id: Option<u64>,
    pub holding_mode: Option<HoldingMode>,
    pub decimals: Option<u8>,
    pub nft_standard: Option<NftStandard>,
    pub chain_wrapped: Option<ChainWrappedKind>,
}
