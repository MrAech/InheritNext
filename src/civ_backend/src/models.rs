use candid::{CandidType, Deserialize};


#[derive(Clone, CandidType, Deserialize)]
pub struct Asset {
    pub id: u64,
    pub name: String,
    pub asset_type: String,
    pub value: u64,
    pub description: String,
    pub created_at: u64,
    pub updated_at: u64,
}
#[derive(Clone, CandidType, Deserialize)]
pub struct AssetInput {
    pub id: u64,
    pub name: String,
    pub asset_type: String,
    pub value: u64,
    pub description: String,
}


#[derive(Clone, CandidType, Deserialize)]
pub struct HeirInput {
    pub id: u64,
    pub name: String,
    pub relationship: String,
    pub email: String,
    pub phone: String,
    pub address: String,
}


#[derive(Clone, CandidType, Deserialize)]
pub struct AssetDistribution {
    pub asset_id: u64,
    pub heir_id: u64,
    pub percentage: u8,
}


#[derive(Clone, CandidType, Deserialize)]
pub struct Heir {
    pub id: u64,
    pub name: String,
    pub relationship: String,
    pub email: String,
    pub phone: String,
    pub address: String,
    pub created_at: u64,
    pub updated_at: u64,
}


#[derive(Clone, CandidType, Deserialize)]
pub struct User {
    pub user: String,
    pub assets: Vec<Asset>,
    pub heirs: Vec<Heir>,
    pub distributions: Vec<AssetDistribution>,
    pub timer: u64,
}


#[derive(CandidType, Deserialize)]
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
}

