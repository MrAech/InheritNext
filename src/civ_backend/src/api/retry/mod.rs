pub mod adaptive;
pub mod bridge_retry;
pub mod control;
pub mod item;
pub mod kind;
pub mod policy;
pub mod processor;
pub mod stats;

use crate::audit::push_audit;
use crate::models::*;
use crate::storage::USERS;
use crate::time::now_secs;

pub use adaptive::AdaptiveKindStats;
pub use stats::RetryStats;

// Facade functions (public API unchanged vs previous single-file implementation)

pub fn enqueue_retry(kind: RetryKind, initial_delay_secs: u64) -> u64 {
    let id = processor::next_retry_id();
    let caller = crate::api::common::user_id();
    // Apply adaptive adjustment to initial delay based on prior outcomes of this kind (if any)
    let adjusted_delay = adaptive::adjust_delay(kind.name(), initial_delay_secs, &caller);
    let item = RetryItem {
        id,
        created_at: now_secs(),
        next_attempt_after: now_secs().saturating_add(adjusted_delay),
        attempts: 0,
        kind,
        last_error: None,
        terminal: false,
    };
    USERS.with(|users| {
        let mut users = users.borrow_mut();
        if let Some(u) = users.get_mut(&caller) {
            u.retry_queue.get_or_insert_with(|| Vec::new()).push(item);
            push_audit(
                u,
                AuditEventKind::RetryAttempt {
                    retry_id: id,
                    attempt: 0,
                    kind: "Enqueued".into(),
                },
            );
        }
    });
    id
}

#[allow(dead_code)] // Admin dashboard (pending) – expose pending & terminal retries for operator insight
pub fn list_retries() -> Vec<RetryItem> {
    let caller = crate::api::common::user_id();
    USERS.with(|users| {
        let users = users.borrow();
        users
            .get(&caller)
            .and_then(|u| u.retry_queue.clone())
            .unwrap_or_default()
    })
}

pub async fn process_retries_for(principal: &str, max_attempts: u32) -> u32 {
    processor::process(principal, max_attempts).await
}

#[allow(dead_code)] // Admin dashboard (pending) – aggregated retry outcome & aging stats
pub fn retry_stats() -> RetryStats {
    let caller = crate::api::common::user_id();
    stats::stats_for(&caller)
}

#[allow(dead_code)] // Admin dashboard (pending) – manual requeue of a specific retry item
pub fn force_retry(id: u64) -> Result<(), CivError> {
    let caller = crate::api::common::user_id();
    control::force_retry(id, &caller)
}

#[allow(dead_code)] // Admin dashboard (pending) – force all retries to become due immediately
pub fn force_all_due() {
    let caller = crate::api::common::user_id();
    control::force_all_due(&caller)
}

pub use item::RetryItem;
pub use kind::RetryKind; // re-export kind type // re-export item type for external visibility
