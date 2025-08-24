use crate::models::{AuditEvent, AuditEventKind, User};
use crate::time::now_secs;

pub fn push_audit(user: &mut User, kind: AuditEventKind) {
    let next_id = user.audit_log.iter().map(|e| e.id).max().unwrap_or(0) + 1;
    user.audit_log.push(AuditEvent {
        id: next_id,
        timestamp: now_secs(),
        kind,
    });
    // Retention cap (simple FIFO trim) to prevent unbounded growth
    const MAX_AUDIT_EVENTS: usize = 10_000;
    if user.audit_log.len() > MAX_AUDIT_EVENTS {
        let overflow = user.audit_log.len() - MAX_AUDIT_EVENTS;
        user.audit_log.drain(0..overflow);
    }
}
