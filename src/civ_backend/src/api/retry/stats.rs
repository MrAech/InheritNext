use crate::models::*;
use crate::storage::USERS;
use crate::time::now_secs;

#[derive(Clone, candid::CandidType, serde::Deserialize)]
pub struct RetryStats {
    pub total: u64,
    pub terminal: u64,
    pub due_now: u64,
    pub earliest_next: Option<u64>,
}

pub fn stats_for(caller: &str) -> RetryStats {
    USERS.with(|users| {
        let users = users.borrow();
        if let Some(u) = users.get(caller) {
            if let Some(q) = &u.retry_queue {
                let now = now_secs();
                let total = q.len() as u64;
                let terminal = q.iter().filter(|r| r.terminal).count() as u64;
                let due_now = q
                    .iter()
                    .filter(|r| !r.terminal && now >= r.next_attempt_after)
                    .count() as u64;
                let earliest_next = q
                    .iter()
                    .filter(|r| !r.terminal && r.next_attempt_after > now)
                    .map(|r| r.next_attempt_after)
                    .min();
                return RetryStats {
                    total,
                    terminal,
                    due_now,
                    earliest_next,
                };
            }
        }
        RetryStats {
            total: 0,
            terminal: 0,
            due_now: 0,
            earliest_next: None,
        }
    })
}
