use super::base::PayoutPreference;
use candid::{CandidType, Deserialize};

#[derive(Clone, CandidType, Deserialize)]
pub struct AssetDistribution {
    pub asset_id: u64,
    pub heir_id: u64,
    pub percentage: u8,
}

#[derive(Clone, CandidType, Deserialize)]
pub struct DistributionShare {
    pub asset_id: u64,
    pub heir_id: u64,
    pub percentage: u8,
    pub payout_preference: PayoutPreference,
}

#[derive(Clone, CandidType, Deserialize)]
pub struct HeirPayoutOverride {
    pub heir_id: u64,
    pub asset_id: u64,
    pub payout_preference: PayoutPreference,
    pub set_at: u64,
}
