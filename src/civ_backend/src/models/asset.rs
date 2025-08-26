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
    pub decimals: u8,
    pub description: String,
    pub created_at: u64,
    pub updated_at: u64,
    pub token_canister: Option<String>,
    pub token_id: Option<u64>,
    pub holding_mode: Option<HoldingMode>,
    pub nft_standard: Option<NftStandard>,
    pub chain_wrapped: Option<ChainWrappedKind>,
    pub file_path: Option<String>,
}

#[derive(Clone, CandidType, Deserialize)]
pub struct AssetInput {
    pub name: String,
    pub asset_type: String,
    pub description: String,
    // Optional friendly kind string sent by the frontend (Fungible | NFT | ChainWrapped | Document)
    pub kind: Option<String>,
    // Optional chain/token references that the frontend may provide when linking an on-chain asset
    pub token_canister: Option<String>,
    pub token_id: Option<u64>,
    // For document assets the frontend may include an optional file path placeholder
    pub file_path: Option<String>,
    // Optional fields the frontend may send to indicate how the asset should be treated
    pub holding_mode: Option<HoldingMode>,
    pub nft_standard: Option<NftStandard>,
    pub chain_wrapped: Option<ChainWrappedKind>,
}

#[derive(Clone, CandidType, Deserialize)]
pub struct AssetTokenMetaInput {
    pub token_canister: Option<String>,
    pub token_id: Option<u64>,
    pub holding_mode: Option<HoldingMode>,
    // Optional explicit decimals provided by a trusted metadata update path.
    pub decimals: Option<u8>,
    // Optional explicit value (smallest units) provided by a trusted metadata update path.
    pub value: Option<u64>,
    pub nft_standard: Option<NftStandard>,
    pub chain_wrapped: Option<ChainWrappedKind>,
}
