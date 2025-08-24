use candid::CandidType;
use serde::Deserialize;

#[derive(Clone, CandidType, Deserialize)]
pub enum RetryKind {
    FungibleCustodyRelease {
        asset_id: u64,
        heir_id: u64,
    },
    NftCustodyRelease {
        asset_id: u64,
        heir_id: u64,
        token_id: u64,
    },
    BridgeSubmit {
        asset_id: u64,
        heir_id: u64,
    },
    BridgePoll {
        asset_id: u64,
        heir_id: u64,
    },
    EscrowRelease {
        asset_id: u64,
        heir_id: u64,
    },
}

impl RetryKind {
    pub fn name(&self) -> &'static str {
        match self {
            RetryKind::FungibleCustodyRelease { .. } => "FungibleCustodyRelease",
            RetryKind::NftCustodyRelease { .. } => "NftCustodyRelease",
            RetryKind::BridgeSubmit { .. } => "BridgeSubmit",
            RetryKind::BridgePoll { .. } => "BridgePoll",
            RetryKind::EscrowRelease { .. } => "EscrowRelease",
        }
    }
}
