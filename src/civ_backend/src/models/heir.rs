use super::base::{HeirIdentitySecret, HeirSecretStatus};
use candid::{CandidType, Deserialize};

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
pub struct HeirInput {
    pub name: String,
    pub relationship: String,
    pub email: String,
    pub phone: String,
    pub address: String,
}

#[derive(Clone, CandidType, Deserialize)]
pub struct HeirEx {
    pub id: u64,
    pub name: String,
    pub relationship: String,
    pub email: String,
    pub phone: String,
    pub address: String,
    pub principal: Option<String>,
    pub identity_secret: HeirIdentitySecret,
    pub identity_hash: Option<Vec<u8>>,
    pub identity_salt: Option<Vec<u8>>,
    pub notes: Option<String>,
    pub created_at: u64,
    pub updated_at: u64,
}

#[derive(Clone, CandidType, Deserialize)]
pub struct HeirAddInputV2 {
    pub name: String,
    pub relationship: String,
    pub email: String,
    pub phone: String,
    pub address: String,
    // Renamed field: external Candid name `heirPrincipal` to avoid raw `principal` identifier conflict in some generators.
    // Backward compatibility: accept legacy `principal` field via serde alias.
    #[serde(alias = "principal", rename = "heirPrincipal")]
    pub heir_principal: Option<String>,
    pub secret_kind: String,
    pub secret_plain: String,
    pub identity_claim: Option<String>,
    pub notes: Option<String>,
}
