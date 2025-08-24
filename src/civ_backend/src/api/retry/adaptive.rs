use crate::audit::push_audit;
use crate::models::AuditEventKind;
use crate::storage::USERS;
use crate::time::now_secs;

#[derive(Clone, candid::CandidType, serde::Deserialize, Debug)]
pub struct AdaptiveKindStats {
    pub successes: u64,
    pub failures: u64,
    pub last_update: u64,
    pub dynamic_factor_bps: u32,
}

impl AdaptiveKindStats {
    pub fn new() -> Self {
        AdaptiveKindStats {
            successes: 0,
            failures: 0,
            last_update: now_secs(),
            dynamic_factor_bps: 10_000,
        }
    } // 10_000 bps = 1.0x
    pub fn record(&mut self, ok: bool) {
        if ok {
            self.successes = self.successes.saturating_add(1);
        } else {
            self.failures = self.failures.saturating_add(1);
        }
        self.last_update = now_secs();
        self.recompute();
    }
    fn recompute(&mut self) {
        // Very simple heuristic: base factor starts at 1.0 (10000 bps). Increase delay (factor up) if failure ratio high, decrease if low.
        let total = self.successes + self.failures;
        if total < 5 {
            return;
        }
        let failure_ratio_bps = if total > 0 {
            (self.failures.saturating_mul(10_000) / total) as u32
        } else {
            0
        };
        // Map: 0% failures -> 8000 bps (0.8x), 50% -> 10000 bps, 80%+ -> 20000 bps (2.0x)
        let factor = if failure_ratio_bps < 5000 {
            // below 50%
            8_000u32 + (failure_ratio_bps * (2_000 / 5_000)) // linear towards 10_000
        } else {
            // 50%..100% range -> 10_000 .. 20_000
            let over = failure_ratio_bps - 5_000;
            10_000 + (over * (10_000 / 5_000))
        }
        .clamp(5_000, 25_000); // clamp 0.5x .. 2.5x
        self.dynamic_factor_bps = factor;
    }
}

pub fn adjust_delay(kind_name: &str, base_delay: u64, principal: &str) -> u64 {
    USERS.with(|users| {
        let mut users = users.borrow_mut();
        if let Some(u) = users.get_mut(principal) {
            if let Some(map) = &mut u.retry_adaptive {
                if let Some(st) = map.get(kind_name) {
                    return apply_factor(base_delay, st.dynamic_factor_bps);
                }
            }
        }
        base_delay
    })
}

pub fn record_outcome(kind_name: &str, success: bool, principal: &str) {
    USERS.with(|users| {
        let mut users = users.borrow_mut();
        if let Some(u) = users.get_mut(principal) {
            let map = u
                .retry_adaptive
                .get_or_insert_with(|| std::collections::HashMap::new());
            let stats = map
                .entry(kind_name.to_string())
                .or_insert_with(AdaptiveKindStats::new);
            let before = stats.dynamic_factor_bps;
            stats.record(success);
            let after = stats.dynamic_factor_bps;
            if before != after {
                push_audit(
                    u,
                    AuditEventKind::RetryAttempt {
                        retry_id: 0,
                        attempt: 0,
                        kind: format!("AdaptiveFactor:{}->{}:{}", before, after, kind_name),
                    },
                );
            }
        }
    });
}

fn apply_factor(base: u64, bps: u32) -> u64 {
    ((base as u128) * (bps as u128) / 10_000u128).max(1) as u64
}
