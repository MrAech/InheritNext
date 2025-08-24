use super::base::HoldingMode;
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

#[derive(Clone, CandidType, Deserialize)]
pub struct Asset {
    pub id: u64,
    pub name: String,
    pub asset_type: String,
    pub value: u64,
    pub decimals: Option<u8>,
    pub description: String,
    pub created_at: u64,
    pub updated_at: u64,
    pub token_canister: Option<String>,
    pub token_id: Option<u64>,
    pub holding_mode: Option<HoldingMode>,
    pub nft_standard: Option<NftStandard>,
    pub chain_wrapped: Option<ChainWrappedKind>,
}

#[derive(Clone, CandidType, Deserialize)]
pub struct AssetInput {
    pub name: String,
    pub asset_type: String,
    pub value: u64,
    pub decimals: Option<u8>,
    pub description: String,
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
