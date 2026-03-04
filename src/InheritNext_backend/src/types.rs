use std::borrow::Cow;

use candid::{CandidType, Principal};
use ic_stable_structures::{storable::Bound, Storable};
use serde::{Deserialize, Serialize};

#[derive(
    Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Hash, Debug, CandidType, Serialize, Deserialize,
)]
pub struct StablePrincipal(pub Principal);

impl Storable for StablePrincipal {
    fn to_bytes(&self) -> Cow<'_, [u8]> {
        Cow::Owned(self.0.as_slice().to_vec())
    }

    fn into_bytes(self) -> Vec<u8> {
        self.0.as_slice().to_vec()
    }

    fn from_bytes(bytes: Cow<'_, [u8]>) -> Self {
        StablePrincipal(Principal::from_slice(&bytes))
    }

    const BOUND: Bound = Bound::Bounded {
        max_size: 29,
        is_fixed_size: false,
    };
}

#[derive(Clone, Debug, CandidType, Serialize, Deserialize)]
pub struct UserProfile {
    pub first_name: String,
    pub last_name: String,
    pub created_at: u64,
}

impl Storable for UserProfile {
    fn to_bytes(&self) -> Cow<'_, [u8]> {
        Cow::Owned(candid::encode_one(self).unwrap())
    }

    fn into_bytes(self) -> Vec<u8> {
        candid::encode_one(self).unwrap()
    }

    fn from_bytes(bytes: Cow<'_, [u8]>) -> Self {
        candid::decode_one(&bytes).unwrap()
    }

    const BOUND: Bound = Bound::Unbounded;
}

#[derive(Clone, Serialize, Deserialize, CandidType, PartialEq, Debug)]
pub enum VaultStatus {
    NotCreated,
    Active,
    Pending,
    Released,
}

#[derive(Clone, Serialize, Deserialize, CandidType, PartialEq, Debug)]
pub struct DeadManSwitch {
    pub last_heartbeat: u64,
    pub heartbeat_interval: u64, // nanoseconds
    pub grace_period: u64,       // nanoseconds
    pub pending_since: Option<u64>,
}

#[derive(Clone, Serialize, Deserialize, CandidType, PartialEq, Debug)]
pub struct RecoveryConfig {
    pub recovery_principals: Vec<Principal>,
    pub threshold: u32,
}

#[derive(Clone, Serialize, Deserialize, CandidType, PartialEq, Debug)]
pub struct Vault {
    pub owner: Principal,
    pub created_at: u64,
    pub status: VaultStatus,
    pub dms: DeadManSwitch,
    pub recovery_config: Option<RecoveryConfig>,
    pub next_asset_id: u64,
}

impl Storable for Vault {
    fn to_bytes(&self) -> Cow<'_, [u8]> {
        Cow::Owned(candid::encode_one(self).unwrap())
    }

    fn into_bytes(self) -> Vec<u8> {
        candid::encode_one(self).unwrap()
    }

    fn from_bytes(bytes: Cow<'_, [u8]>) -> Self {
        candid::decode_one(&bytes).unwrap()
    }

    const BOUND: Bound = Bound::Unbounded;
}

#[derive(Clone, Serialize, Deserialize, CandidType, PartialEq, Debug)]
pub enum EventType {
    VaultCreated,
    AssetCreated,
    AssetUpdated,
    AssetDeleted,
    HeirAdded,
    HeirRemoved,
    Heartbeat,
    SwitchPending,
    VaultReleased,
    RecoveryInitiated,
}

#[derive(Clone, Serialize, Deserialize, CandidType, PartialEq, Debug)]
pub struct AuditEvent {
    pub timestamp: u64,
    pub event_type: EventType,
    pub blame: Principal,
    pub details: String,
}

impl Storable for AuditEvent {
    fn to_bytes(&self) -> Cow<'_, [u8]> {
        Cow::Owned(candid::encode_one(self).unwrap())
    }

    fn into_bytes(self) -> Vec<u8> {
        candid::encode_one(self).unwrap()
    }

    fn from_bytes(bytes: Cow<'_, [u8]>) -> Self {
        candid::decode_one(&bytes).unwrap()
    }

    const BOUND: Bound = Bound::Unbounded;
}

#[derive(
    Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, CandidType, Serialize, Deserialize, Debug,
)]
pub struct EventId(pub u64);

impl Storable for EventId {
    fn to_bytes(&self) -> Cow<'_, [u8]> {
        Cow::Owned(self.0.to_be_bytes().to_vec())
    }

    fn into_bytes(self) -> Vec<u8> {
        self.0.to_be_bytes().to_vec()
    }

    fn from_bytes(bytes: Cow<'_, [u8]>) -> Self {
        let mut arr = [0u8; 8];
        arr.copy_from_slice(&bytes);
        EventId(u64::from_be_bytes(arr))
    }

    const BOUND: Bound = Bound::Bounded {
        max_size: 8,
        is_fixed_size: true,
    };
}

#[derive(
    Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, CandidType, Serialize, Deserialize, Debug,
)]
pub struct AssetId(pub u64);

impl Storable for AssetId {
    fn to_bytes(&self) -> Cow<'_, [u8]> {
        Cow::Owned(self.0.to_be_bytes().to_vec())
    }

    fn into_bytes(self) -> Vec<u8> {
        self.0.to_be_bytes().to_vec()
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        let mut arr = [0u8; 8];
        arr.copy_from_slice(&bytes);
        AssetId(u64::from_be_bytes(arr))
    }

    const BOUND: Bound = Bound::Bounded {
        max_size: 8,
        is_fixed_size: true,
    };
}

#[derive(Clone, Serialize, Deserialize, CandidType, PartialEq, Debug)]
pub enum AssetType {
    ICRC2Token {
        ledger_canister: Principal,
        amount: u64,
    },
}

#[derive(Clone, Serialize, Deserialize, CandidType, PartialEq, Debug)]
pub struct HeirAssignment {
    pub heir_principal: Principal,
    pub percentage: u8,
}

#[derive(Clone, Serialize, Deserialize, CandidType, PartialEq, Debug)]
pub struct Asset {
    pub id: u64,
    pub owner: Principal,
    pub asset_type: AssetType,
    pub name: String,
    pub description: String,
    pub created_at: u64,
    pub heir_assingment: Vec<HeirAssignment>,
}

impl Storable for Asset {
    fn to_bytes(&self) -> Cow<'_, [u8]> {
        Cow::Owned(candid::encode_one(self).expect("Failed to encode Asset"))
    }

    fn into_bytes(self) -> Vec<u8> {
        candid::encode_one(self).expect("Failed to encode Asset")
    }

    fn from_bytes(bytes: Cow<'_, [u8]>) -> Self {
        candid::decode_one(&bytes).expect("Failed to decode Asset - storage corruption detected")
    }

    const BOUND: Bound = Bound::Unbounded;
}
