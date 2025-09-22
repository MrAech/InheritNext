
use candid::{CandidType, Deserialize, Principal};

#[derive(Clone, CandidType, Deserialize)]
pub struct UserProfile {
    pub principal: Principal,
    pub name: String,
    pub gov_id_hash: String,
    pub pbkdf2_salt: String,
    pub terms_accepted: bool,
    pub plan_type: PlanType,
    pub activated: bool,
    pub activation_timestamp: Option<u64>,
    pub expiry_timer: Option<u64>,
    pub warning_days: u64,
    pub inactivity_days: u64,
}

#[derive(Clone, CandidType, Deserialize)]
pub enum PlanType {
    Basic,
    Tier1,
    Custom,
}

#[derive(Clone, CandidType, Deserialize)]
pub struct Asset {
    pub asset_id: String, // ICRC-1/2/37 asset id
    pub asset_type: String, // e.g. "ckBTC", "ckETH", "NFT"
    pub approved: bool,
    pub value: u64,
    pub name: String,
    pub description: String,
}

#[derive(Clone, CandidType, Deserialize)]
pub struct Heir {
    pub name: String,
    pub gov_id_hash: String,
    pub security_question_hash: Option<String>,
}

#[derive(Clone, CandidType, Deserialize)]
pub struct Distribution {
    pub asset_id: String,
    pub heir_name: String,
    pub heir_gov_id: String,
    pub percent: u32,
}

#[derive(Clone, CandidType, Deserialize)]
pub struct UserState {
    pub profile: UserProfile,
    pub assets: Vec<Asset>,
    pub heirs: Vec<Heir>,
    pub distributions: Vec<Distribution>,
    // New: approvals for fungible tokens
    pub approvals: Vec<AssetApproval>,
    // New: vaulted NFTs owned by this user (transferred into custodian vault)
    pub vaulted_nfts: Vec<VaultedNFT>,
}

#[derive(Clone, CandidType, Deserialize)]
pub struct AssetApproval {
    pub owner: Principal,
    pub asset_type: String,
    pub token_canister: Principal,
    pub approved_amount: u64,
    pub approval_expiry: Option<u64>,
    pub auto_renew: bool,
}

#[derive(Clone, CandidType, Deserialize)]
pub struct VaultedNFT {
    pub owner: Principal,
    pub collection_canister: Principal,
    pub token_id: String,
    pub assigned_heir_hash: String,
}

#[derive(Clone, CandidType, Deserialize)]
pub enum JournalStatus {
    Pending,
    Success,
    Failed,
}

#[derive(Clone, CandidType, Deserialize)]
pub struct JournalEntry {
    pub id: u64,
    pub asset_id: String,
    pub action: String,
    pub details: String, // JSON or text-encoded details for later decoding
    pub status: JournalStatus,
    pub attempts: u32,
    pub created_at: u64,
    pub updated_at: u64,
}

#[derive(CandidType, Deserialize)]
pub enum CustodianError {
    NotAuthorized,
    NotFound,
    InvalidInput,
    DistributionInvalid,
    AlreadyExists,
    NotActivated,
    NotApproved,
    Other(String),
}

