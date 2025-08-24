use crate::api::common::{assert_mutable, user_id};
use crate::audit::push_audit;
use crate::models::*;
use crate::storage::USERS;
use crate::time::now_secs;

#[derive(Clone, candid::CandidType, serde::Deserialize)]
pub struct ReadinessReport {
    pub ready: bool,
    pub issues: Vec<String>,
}

pub fn estate_readiness() -> ReadinessReport {
    const FRESH_SECS: u64 = 30; // caching window
    let caller = user_id();
    // First quick path: if cache present & fresh, return it
    if let Some(cached) = USERS.with(|users| {
        let users = users.borrow();
        users.get(&caller).and_then(|u| u.readiness_cache.clone())
    }) {
        if now_secs().saturating_sub(cached.computed_at) <= FRESH_SECS {
            return ReadinessReport {
                ready: cached.ready,
                issues: cached.issues,
            };
        }
    }
    // Compute new report and store into cache
    USERS.with(|users| {
        let mut users_mut = users.borrow_mut();
        if let Some(u) = users_mut.get_mut(&caller) {
            let mut issues = Vec::new();
            use std::collections::HashMap;
            let mut per_asset: HashMap<u64, u32> = HashMap::new();
            for d in &u.distributions_v2 {
                *per_asset.entry(d.asset_id).or_insert(0) += d.percentage as u32;
            }
            for (asset_id, total) in per_asset.iter() {
                if *total != 100 {
                    issues.push(format!(
                        "asset_{}_not_fully_allocated_{}pct",
                        asset_id, total
                    ));
                }
            }
            for a in u.assets.iter() {
                let kind = crate::models::infer_asset_kind(&a.asset_type);
                if matches!(kind, AssetKind::Fungible | AssetKind::ChainWrapped)
                    && a.decimals.is_none()
                {
                    issues.push(format!("asset_{}_missing_decimals", a.id));
                }
            }
            for d in &u.distributions_v2 {
                if let Some(heir) = u.heirs_v2.iter().find(|h| h.id == d.heir_id) {
                    let needs_principal = matches!(
                        d.payout_preference,
                        PayoutPreference::ToPrincipal | PayoutPreference::CkWithdraw
                    );
                    if needs_principal && heir.principal.is_none() {
                        issues.push(format!("heir_{}_missing_principal", heir.id));
                    }
                } else {
                    issues.push(format!("distribution_missing_heir_{}", d.heir_id));
                }
            }
            let blocking = issues
                .iter()
                .filter(|i| i.contains("not_fully_allocated") || i.contains("missing_heir"))
                .count();
            let report = ReadinessReport {
                ready: blocking == 0,
                issues: issues.clone(),
            };
            u.readiness_cache = Some(ReadinessCache {
                computed_at: now_secs(),
                ready: report.ready,
                issues,
            });
            report
        } else {
            ReadinessReport {
                ready: false,
                issues: vec!["user_not_found".into()],
            }
        }
    })
}

// Filtered audit listing (by optional asset_id and/or heir_id) with pagination
pub fn list_audit_filtered(
    offset: u64,
    limit: u64,
    asset_id: Option<u64>,
    heir_id: Option<u64>,
) -> Vec<AuditEvent> {
    let caller = user_id();
    USERS.with(|users| {
        let users = users.borrow();
        if let Some(u) = users.get(&caller) {
            let mut events: Vec<AuditEvent> = u.audit_log.iter().cloned().collect();
            if let Some(aid) = asset_id {
                events.retain(|e| match &e.kind {
                    AuditEventKind::FungibleCustodyStaged { asset_id, .. } => *asset_id == aid,
                    AuditEventKind::NftCustodyStaged { asset_id, .. } => *asset_id == aid,
                    AuditEventKind::CkWithdrawStaged { asset_id, .. } => *asset_id == aid,
                    AuditEventKind::EscrowReconciliationDiscrepancy { asset_id, .. } => {
                        *asset_id == aid
                    }
                    AuditEventKind::CustodyReconciliationDiscrepancy { .. } => true,
                    _ => true,
                });
            }
            if let Some(hid) = heir_id {
                events.retain(|e| match &e.kind {
                    AuditEventKind::FungibleCustodyStaged { heir_id, .. } => *heir_id == hid,
                    AuditEventKind::NftCustodyStaged { heir_id, .. } => *heir_id == hid,
                    AuditEventKind::CkWithdrawStaged { heir_id, .. } => *heir_id == hid,
                    AuditEventKind::CustodyReconciliationDiscrepancy { heir_id, .. } => {
                        *heir_id == hid
                    }
                    _ => true,
                });
            }
            let total = events.len() as u64;
            let off = offset.min(total);
            let lim = limit.min(500).max(1);
            let end = (off + lim).min(total);
            events
                .into_iter()
                .skip(off as usize)
                .take((end - off) as usize)
                .collect()
        } else {
            Vec::new()
        }
    })
}

#[derive(Clone, candid::CandidType, serde::Deserialize)]
pub struct MetricsSnapshot {
    pub retry_counts: std::collections::HashMap<String, u64>,
    pub custody_totals: std::collections::HashMap<u64, u128>,
    pub escrow_totals: std::collections::HashMap<u64, u128>,
    pub custody_discrepancies: u64,
    pub escrow_discrepancies: u64,
}

pub fn metrics_snapshot() -> MetricsSnapshot {
    let caller = user_id(); // per-user metrics for now
    USERS.with(|users| {
        let users = users.borrow();
        if let Some(u) = users.get(&caller) {
            use std::collections::HashMap;
            let mut retry_counts: HashMap<String, u64> = HashMap::new();
            if let Some(q) = &u.retry_queue {
                for item in q {
                    let k = match &item.kind {
                        crate::api::retry::RetryKind::FungibleCustodyRelease { .. } => {
                            "FungibleCustodyRelease"
                        }
                        crate::api::retry::RetryKind::NftCustodyRelease { .. } => {
                            "NftCustodyRelease"
                        }
                        crate::api::retry::RetryKind::BridgeSubmit { .. } => "BridgeSubmit",
                        crate::api::retry::RetryKind::BridgePoll { .. } => "BridgePoll",
                        crate::api::retry::RetryKind::EscrowRelease { .. } => "EscrowRelease",
                    };
                    *retry_counts.entry(k.to_string()).or_insert(0) += 1;
                }
            }
            let mut custody_totals: HashMap<u64, u128> = HashMap::new();
            for c in &u.fungible_custody {
                if c.released_at.is_none() {
                    *custody_totals.entry(c.asset_id).or_insert(0) += c.amount;
                }
            }
            let mut escrow_totals: HashMap<u64, u128> = HashMap::new();
            for e in &u.escrow {
                *escrow_totals.entry(e.asset_id).or_insert(0) += e.amount.unwrap_or(0);
            }
            let mut custody_discrepancies = 0u64;
            let mut escrow_discrepancies = 0u64;
            for ev in &u.audit_log {
                match &ev.kind {
                    AuditEventKind::CustodyReconciliationDiscrepancy { .. } => {
                        custody_discrepancies += 1
                    }
                    AuditEventKind::EscrowReconciliationDiscrepancy { .. } => {
                        escrow_discrepancies += 1
                    }
                    _ => {}
                }
            }
            MetricsSnapshot {
                retry_counts,
                custody_totals,
                escrow_totals,
                custody_discrepancies,
                escrow_discrepancies,
            }
        } else {
            MetricsSnapshot {
                retry_counts: Default::default(),
                custody_totals: Default::default(),
                escrow_totals: Default::default(),
                custody_discrepancies: 0,
                escrow_discrepancies: 0,
            }
        }
    })
}

// Internal: build MetricsFrame for a specific principal (used by maintenance to append history)
pub fn metrics_snapshot_internal(principal: &str) -> Option<crate::models::MetricsFrame> {
    USERS.with(|users| {
        let users = users.borrow();
        users.get(principal).map(|u| {
            use std::collections::HashMap;
            let mut retry_counts: HashMap<String, u64> = HashMap::new();
            if let Some(q) = &u.retry_queue {
                for item in q {
                    let k = match &item.kind {
                        crate::api::retry::RetryKind::FungibleCustodyRelease { .. } => {
                            "FungibleCustodyRelease"
                        }
                        crate::api::retry::RetryKind::NftCustodyRelease { .. } => {
                            "NftCustodyRelease"
                        }
                        crate::api::retry::RetryKind::BridgeSubmit { .. } => "BridgeSubmit",
                        crate::api::retry::RetryKind::BridgePoll { .. } => "BridgePoll",
                        crate::api::retry::RetryKind::EscrowRelease { .. } => "EscrowRelease",
                    };
                    *retry_counts.entry(k.to_string()).or_insert(0) += 1;
                }
            }
            let mut custody_totals: HashMap<u64, u128> = HashMap::new();
            for c in &u.fungible_custody {
                if c.released_at.is_none() {
                    *custody_totals.entry(c.asset_id).or_insert(0) += c.amount;
                }
            }
            let mut escrow_totals: HashMap<u64, u128> = HashMap::new();
            for e in &u.escrow {
                *escrow_totals.entry(e.asset_id).or_insert(0) += e.amount.unwrap_or(0);
            }
            let mut custody_discrepancies = 0u64;
            let mut escrow_discrepancies = 0u64;
            for ev in &u.audit_log {
                match &ev.kind {
                    AuditEventKind::CustodyReconciliationDiscrepancy { .. } => {
                        custody_discrepancies += 1
                    }
                    AuditEventKind::EscrowReconciliationDiscrepancy { .. } => {
                        escrow_discrepancies += 1
                    }
                    _ => {}
                }
            }
            crate::models::MetricsFrame {
                ts: now_secs(),
                retry_counts,
                custody_totals,
                escrow_totals,
                custody_discrepancies,
                escrow_discrepancies,
            }
        })
    })
}

#[allow(dead_code)] // Admin dashboard (pending) – direct metrics frame query for arbitrary principal
pub fn metrics_snapshot_for(principal: &str) -> Option<crate::models::MetricsFrame> {
    metrics_snapshot_internal(principal)
}

pub fn set_distribution_v2(asset_id: u64, shares: Vec<DistributionShare>) -> Result<(), CivError> {
    let caller = user_id();
    USERS.with(|users| {
        let mut users = users.borrow_mut();
        if let Some(user) = users.get_mut(&caller) {
            assert_mutable(user)?;
            let mut total: u32 = 0;
            use crate::models::infer_asset_kind;
            let asset = user
                .assets
                .iter()
                .find(|a| a.id == asset_id)
                .ok_or(CivError::DistributionAssetNotFound)?;
            let kind = infer_asset_kind(&asset.asset_type);
            for s in &shares {
                total += s.percentage as u32;
                if !user.heirs_v2.iter().any(|h| h.id == s.heir_id) {
                    return Err(CivError::DistributionHeirNotFound);
                }
                let allowed = match kind {
                    AssetKind::Fungible => matches!(
                        s.payout_preference,
                        PayoutPreference::ToPrincipal | PayoutPreference::ToCustody
                    ),
                    AssetKind::ChainWrapped => matches!(
                        s.payout_preference,
                        PayoutPreference::ToPrincipal
                            | PayoutPreference::ToCustody
                            | PayoutPreference::CkWithdraw
                    ),
                    AssetKind::Nft => matches!(
                        s.payout_preference,
                        PayoutPreference::ToPrincipal | PayoutPreference::ToCustody
                    ),
                    AssetKind::Document => false,
                };
                if !allowed {
                    return Err(CivError::InvalidPayoutPreference);
                }
            }
            if total != 100 {
                return Err(CivError::InvalidHeirPercentage);
            }
            user.distributions_v2.retain(|d| d.asset_id != asset_id);
            user.distributions_v2.extend(shares);
            push_audit(user, AuditEventKind::DistributionSet { asset_id });
            if user.timer_expiry == 0 {
                user.timer_expiry = now_secs() + INACTIVITY_PERIOD_SECS;
            }
            Ok(())
        } else {
            Err(CivError::UserNotFound)
        }
    })
}

// Legacy / transitional distribution APIs
#[allow(dead_code)]
// Legacy bulk assignment API (kept for admin dashboard & backward compat); prefer set_distribution_v2 / set_asset_distributions
#[deprecated(note = "Use per-asset APIs instead.")]
pub fn assign_distributions(distributions: Vec<AssetDistribution>) -> Result<(), CivError> {
    let user = user_id();
    USERS.with(|users| {
        let mut users = users.borrow_mut();
        let now = now_secs();
        let user = users
            .entry(user.clone())
            .or_insert_with(|| User::new(&user, now + INACTIVITY_PERIOD_SECS));
        use std::collections::HashMap;
        let mut asset_map: HashMap<u64, u32> = HashMap::new();
        for dist in &distributions {
            if !user.assets.iter().any(|a| a.id == dist.asset_id) {
                return Err(CivError::DistributionAssetNotFound);
            }
            if !user.heirs.iter().any(|h| h.id == dist.heir_id) {
                return Err(CivError::DistributionHeirNotFound);
            }
            *asset_map.entry(dist.asset_id).or_insert(0) += dist.percentage as u32;
        }
        if asset_map.values().any(|&s| s != 100) {
            return Err(CivError::InvalidHeirPercentage);
        }
        user.distributions = distributions;
        Ok(())
    })
}

#[allow(dead_code)] // Legacy helper for older UI list mapping; slated for removal after admin dashboard cut
#[deprecated(note = "Use get_asset_distributions instead.")]
pub fn get_distribution() -> Vec<(String, u64)> {
    let user = user_id();
    USERS.with(|users| {
        let users = users.borrow();
        users
            .get(&user)
            .map(|u| {
                u.distributions
                    .iter()
                    .map(|d| (d.asset_id.to_string(), d.heir_id))
                    .collect()
            })
            .unwrap_or_default()
    })
}
#[allow(dead_code)] // Legacy full distribution list; superseded by per-asset queries
#[deprecated(note = "Use get_asset_distributions instead.")]
pub fn list_distributions() -> Vec<AssetDistribution> {
    let user = user_id();
    USERS.with(|users| {
        let users = users.borrow();
        users
            .get(&user)
            .map(|u| u.distributions.clone())
            .unwrap_or_default()
    })
}

pub fn get_asset_distributions(asset_id: u64) -> Vec<AssetDistribution> {
    let user = user_id();
    USERS.with(|users| {
        let users = users.borrow();
        users
            .get(&user)
            .map(|u| {
                u.distributions
                    .iter()
                    .filter(|d| d.asset_id == asset_id)
                    .cloned()
                    .collect()
            })
            .unwrap_or_else(|| Vec::new())
    })
}

pub fn set_asset_distributions(
    asset_id: u64,
    distributions: Vec<AssetDistribution>,
) -> Result<(), CivError> {
    let user = user_id();
    USERS.with(|users| {
        let mut users = users.borrow_mut();
        if let Some(user) = users.get_mut(&user) {
            if !user.assets.iter().any(|a| a.id == asset_id) {
                return Err(CivError::DistributionAssetNotFound);
            }
            let mut total: u32 = 0;
            for d in &distributions {
                if d.asset_id != asset_id {
                    return Err(CivError::DistributionAssetNotFound);
                }
                if !user.heirs.iter().any(|h| h.id == d.heir_id) {
                    return Err(CivError::DistributionHeirNotFound);
                }
                total += d.percentage as u32;
            }
            if total > 100 {
                return Err(CivError::InvalidHeirPercentage);
            }
            user.distributions.retain(|d| d.asset_id != asset_id);
            user.distributions.extend(distributions);
            let now = now_secs();
            if user.distributions.len() > 0 && user.timer_expiry == 0 {
                user.timer_expiry = now + INACTIVITY_PERIOD_SECS;
                user.distributed = false;
            }
            Ok(())
        } else {
            Err(CivError::UserNotFound)
        }
    })
}

pub fn delete_distribution(asset_id: u64, heir_id: u64) -> Result<(), CivError> {
    let user = user_id();
    USERS.with(|users| {
        let mut users = users.borrow_mut();
        if let Some(user) = users.get_mut(&user) {
            let before = user.distributions.len();
            user.distributions
                .retain(|d| !(d.asset_id == asset_id && d.heir_id == heir_id));
            if before == user.distributions.len() {
                return Err(CivError::DistributionHeirNotFound);
            }
            Ok(())
        } else {
            Err(CivError::UserNotFound)
        }
    })
}

pub fn get_asset_distributions_v2(asset_id: u64) -> Vec<DistributionShare> {
    let caller = user_id();
    USERS.with(|users| {
        let users = users.borrow();
        users
            .get(&caller)
            .map(|u| {
                u.distributions_v2
                    .iter()
                    .filter(|d| d.asset_id == asset_id)
                    .cloned()
                    .collect()
            })
            .unwrap_or_else(|| Vec::new())
    })
}

pub fn check_integrity() -> IntegrityReport {
    let user = user_id();
    USERS.with(|users| {
        let users = users.borrow();
        if let Some(user) = users.get(&user) {
            use std::collections::HashMap;
            let mut map: HashMap<u64, u32> = HashMap::new();
            let mut issues: Vec<String> = Vec::new();
            for d in &user.distributions {
                *map.entry(d.asset_id).or_insert(0) += d.percentage as u32;
                if !user.assets.iter().any(|a| a.id == d.asset_id) {
                    issues.push(format!(
                        "Distribution references missing asset {}",
                        d.asset_id
                    ));
                }
                if !user.heirs.iter().any(|h| h.id == d.heir_id) {
                    issues.push(format!(
                        "Distribution references missing heir {}",
                        d.heir_id
                    ));
                }
                if d.percentage == 0 {
                    issues.push(format!(
                        "Zero percentage entry asset {} heir {}",
                        d.asset_id, d.heir_id
                    ));
                }
            }
            let mut over = Vec::new();
            let mut full = Vec::new();
            let mut partial = Vec::new();
            for asset in &user.assets {
                let sum = map.get(&asset.id).copied().unwrap_or(0);
                if sum > 100 {
                    over.push(asset.id);
                } else if sum == 100 {
                    full.push(asset.id);
                } else if sum > 0 {
                    partial.push(asset.id);
                }
            }
            let unallocated: Vec<u64> = user
                .assets
                .iter()
                .filter(|a| !map.contains_key(&a.id))
                .map(|a| a.id)
                .collect();
            IntegrityReport {
                asset_count: user.assets.len() as u64,
                distribution_count: user.distributions.len() as u64,
                over_allocated_assets: over,
                fully_allocated_assets: full,
                partially_allocated_assets: partial,
                unallocated_assets: unallocated,
                issues,
            }
        } else {
            IntegrityReport {
                asset_count: 0,
                distribution_count: 0,
                over_allocated_assets: vec![],
                fully_allocated_assets: vec![],
                partially_allocated_assets: vec![],
                unallocated_assets: vec![],
                issues: vec!["User not initialized".into()],
            }
        }
    })
}

pub fn get_timer() -> i64 {
    let user = user_id();
    USERS.with(|users| {
        let users = users.borrow();
        if let Some(user) = users.get(&user) {
            let now = now_secs();
            if user.distributions.is_empty() {
                return -1;
            }
            user.timer_expiry.saturating_sub(now) as i64
        } else {
            -1
        }
    })
}

pub fn get_user() -> Option<User> {
    let user = user_id();
    USERS.with(|users| {
        let users = users.borrow();
        users.get(&user).cloned()
    })
}

pub fn list_audit_log() -> Vec<AuditEvent> {
    let caller = user_id();
    USERS.with(|users| {
        let users = users.borrow();
        users
            .get(&caller)
            .map(|u| u.audit_log.clone())
            .unwrap_or_default()
    })
}

pub fn list_audit_paged(offset: usize, limit: usize) -> Vec<AuditEvent> {
    let caller = user_id();
    USERS.with(|users| {
        let users = users.borrow();
        if let Some(u) = users.get(&caller) {
            let len = u.audit_log.len();
            if offset >= len {
                return Vec::new();
            }
            let end = (offset + limit).min(len);
            u.audit_log[offset..end].to_vec()
        } else {
            Vec::new()
        }
    })
}

// Prune old audit events keeping most recent N (soft) and age-based cutoff.
// Called opportunistically (e.g. before listing) to bound memory.
pub fn prune_audit_log(max_events: usize, max_age_secs: u64) {
    let caller = user_id();
    let now = now_secs();
    USERS.with(|users| {
        let mut users = users.borrow_mut();
        if let Some(u) = users.get_mut(&caller) {
            if u.audit_prune_in_progress {
                // Already pruning – emit a lightweight skipped audit and return
                push_audit(
                    u,
                    AuditEventKind::AuditPruned {
                        removed: 0,
                        remaining: u.audit_log.len() as u64,
                    },
                );
                return;
            }
            u.audit_prune_in_progress = true;
            let before = u.audit_log.len();
            // Age prune
            u.audit_log
                .retain(|e| now.saturating_sub(e.timestamp) <= max_age_secs);
            // Size prune (keep newest)
            if u.audit_log.len() > max_events {
                let drop = u.audit_log.len() - max_events;
                u.audit_log.drain(0..drop);
            }
            let after = u.audit_log.len();
            if before != after {
                push_audit(
                    u,
                    AuditEventKind::AuditPruned {
                        removed: (before - after) as u64,
                        remaining: after as u64,
                    },
                );
            }
            u.audit_prune_in_progress = false;
        }
    });
}
