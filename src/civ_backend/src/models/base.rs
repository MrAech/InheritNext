use candid::{CandidType, Deserialize};

// Core cross-cutting enums & small structs
#[derive(Clone, CandidType, Deserialize, PartialEq, Eq)]
pub enum AssetKind {
    Fungible,
    ChainWrapped,
    Nft,
    Document,
}

#[derive(Clone, CandidType, Deserialize, PartialEq, Eq)]
pub enum HoldingMode {
    Escrow,
    Approval,
}

#[derive(Clone, CandidType, Deserialize, PartialEq, Eq)]
pub enum PayoutPreference {
    ToPrincipal,
    ToCustody,
    CkWithdraw,
}

#[derive(Clone, CandidType, Deserialize, PartialEq, Eq)]
pub enum HeirSecretStatus {
    Pending,
    Verified,
}

#[derive(Clone, CandidType, Deserialize)]
pub struct HeirIdentitySecret {
    pub kind: String,
    pub hash: Vec<u8>,
    pub salt: Vec<u8>,
    pub status: HeirSecretStatus,
    pub updated_at: u64,
    #[serde(default)]
    pub attempts: u32,
    #[serde(default)]
    pub last_attempt_at: Option<u64>,
    #[serde(default)]
    pub next_allowed_attempt_at: Option<u64>, // exponential backoff gate
}
