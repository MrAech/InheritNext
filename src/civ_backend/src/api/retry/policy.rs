use super::kind::RetryKind;
use std::collections::BTreeMap;

#[derive(Clone)]
pub struct KindPolicy {
    pub base_secs: u64,
    pub max_secs: u64,
    pub growth: GrowthKind,
}

#[derive(Clone)]
pub enum GrowthKind {
    Exponential,
    Linear,
}

thread_local! { static POLICY: std::cell::RefCell<RetryPolicy> = std::cell::RefCell::new(RetryPolicy::default()); }

#[derive(Clone)]
pub struct RetryPolicy {
    pub per_kind: BTreeMap<&'static str, KindPolicy>,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        let mut m = BTreeMap::new();
        m.insert(
            "FungibleCustodyRelease",
            KindPolicy {
                base_secs: 60,
                max_secs: 30 * 60,
                growth: GrowthKind::Exponential,
            },
        );
        m.insert(
            "NftCustodyRelease",
            KindPolicy {
                base_secs: 120,
                max_secs: 2 * 3600,
                growth: GrowthKind::Exponential,
            },
        );
        m.insert(
            "BridgeSubmit",
            KindPolicy {
                base_secs: 5,
                max_secs: 5 * 60,
                growth: GrowthKind::Exponential,
            },
        );
        m.insert(
            "BridgePoll",
            KindPolicy {
                base_secs: 5,
                max_secs: 10 * 60,
                growth: GrowthKind::Exponential,
            },
        );
        m.insert(
            "EscrowRelease",
            KindPolicy {
                base_secs: 60,
                max_secs: 30 * 60,
                growth: GrowthKind::Exponential,
            },
        );
        RetryPolicy { per_kind: m }
    }
}

pub fn with_policy<R>(f: impl FnOnce(&RetryPolicy) -> R) -> R {
    POLICY.with(|p| f(&p.borrow()))
}

pub fn compute_backoff(kind: &RetryKind, attempts: u32) -> u64 {
    let mut base = with_policy(|pol| {
        let name = kind.name();
        if let Some(kp) = pol.per_kind.get(name) {
            match kp.growth {
                GrowthKind::Exponential => {
                    let exp = attempts.saturating_sub(1).min(16);
                    let delay = kp.base_secs.saturating_mul(1u64 << exp);
                    delay.min(kp.max_secs)
                }
                GrowthKind::Linear => kp
                    .base_secs
                    .saturating_mul(attempts as u64)
                    .min(kp.max_secs),
            }
        } else {
            60
        }
    });

    // Apply +/-20% jitter using RNG if available (non-fatal if not initialized yet).
    let span = base / 5; // 20% of base
    if span > 0 {
        if let Some(r) = crate::rng::try_u64() {
            let offset = r % (2 * span + 1); // 0..2span
            let signed = offset as i64 - span as i64; // -span .. +span
            let adjusted = base as i64 + signed;
            base = adjusted.max(1) as u64;
        }
    }
    base.max(1)
}

pub const RETRY_PRUNE_MAX: usize = 200;
pub const RETRY_PRUNE_AGE_SECS: u64 = 24 * 3600;
