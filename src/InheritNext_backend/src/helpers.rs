use candid::Principal;

use crate::{
    storage,
    types::{AuditEvent, EventType},
};

pub const NANOS_PER_DAY: u64 = 86_400_000_000_000;
pub const DEFAULT_HEARTBEAT_INTERVAL: u64 = 30 * NANOS_PER_DAY;
pub const DEFAULT_GRACE_PERIOD: u64 = 7 * NANOS_PER_DAY;

pub fn now() -> u64 {
    ic_cdk::api::time()
}

pub fn log_event(event_type: EventType, blame: &Principal, details: String) {
    let event = AuditEvent {
        blame: blame.clone(),
        timestamp: now(),
        event_type,
        details,
    };
    storage::log_event(event);
}
