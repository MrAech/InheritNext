use super::kind::RetryKind;
use candid::CandidType;
use serde::Deserialize;

#[derive(Clone, CandidType, Deserialize)]
pub struct RetryItem {
    pub id: u64,
    pub created_at: u64,
    pub next_attempt_after: u64,
    pub attempts: u32,
    pub kind: RetryKind,
    pub last_error: Option<String>,
    pub terminal: bool,
}
