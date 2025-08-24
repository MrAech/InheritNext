use crate::api::common::{assert_mutable, user_id};
use crate::audit::push_audit;
use crate::crypto::{constant_time_eq, hash_secret_with_salt};
use crate::models::*;
use crate::storage::USERS;
use crate::time::now_secs;
use crate::rng; // secure RNG

// Heir secret throttling parameters (initial pragmatic defaults).
// A heir secret may only be attempted HEIR_SECRET_MAX_ATTEMPTS_WINDOW times per
// rolling window of HEIR_SECRET_ATTEMPT_WINDOW_SECS. After the window expires the
// attempt counter is reset. Successful verification resets the counter immediately.
const HEIR_SECRET_ATTEMPT_WINDOW_SECS: u64 = 60 * 60; // legacy window (used for attempt reset baseline)
const HEIR_SECRET_MAX_ATTEMPTS_WINDOW: u32 = 5; // base attempts before stronger backoff kicks
const HEIR_SECRET_BACKOFF_BASE_SECS: u64 = 30; // initial backoff start
const HEIR_SECRET_BACKOFF_MAX_SECS: u64 = 6 * 3600; // cap at 6h

fn compute_secret_backoff(attempts: u32) -> u64 {
    // attempts counted AFTER increment; first attempt attempts=1 => no delay
    if attempts <= 1 {
        return 0;
    }
    // Exponential: base * 2^(attempts-2) (so attempt2 -> base, attempt3 -> 2*base, ...)
    let exp = attempts.saturating_sub(2).min(12); // cap growth
    let delay = HEIR_SECRET_BACKOFF_BASE_SECS.saturating_mul(1u64 << exp);
    delay.min(HEIR_SECRET_BACKOFF_MAX_SECS)
}

// Basic heir operations (legacy + v2)
pub fn add_heir(new_heir: HeirInput) -> Result<(), CivError> {
    let user = user_id();
    USERS.with(|users| {
        let mut users = users.borrow_mut();
        let now = now_secs();
        let user = users.entry(user.clone()).or_insert_with(|| {
            let mut u = User::new(&user, now + INACTIVITY_PERIOD_SECS);
            push_audit(&mut u, AuditEventKind::UserCreated);
            u
        });
        assert_mutable(user)?;
        let next_id = user.heirs.iter().map(|h| h.id).max().unwrap_or(0) + 1;
        let HeirInput {
            name,
            relationship,
            email,
            phone,
            address,
        } = new_heir;
        user.heirs.push(Heir {
            id: next_id,
            name: name.clone(),
            relationship: relationship.clone(),
            email: email.clone(),
            phone: phone.clone(),
            address: address.clone(),
            created_at: now,
            updated_at: now,
        });
        // placeholder v2 extended record
    let mut salt = vec![0u8; 16];
    rng::fill(&mut salt);
        let placeholder_hash = hash_secret_with_salt("PENDING", &salt);
        user.heirs_v2.push(HeirEx {
            id: next_id,
            name,
            relationship,
            email,
            phone,
            address,
            principal: None,
            identity_secret: HeirIdentitySecret {
                kind: "PENDING".into(),
                hash: placeholder_hash.to_vec(),
                salt,
                status: HeirSecretStatus::Pending,
                updated_at: now,
                attempts: 0,
                last_attempt_at: None,
                next_allowed_attempt_at: None,
            },
            identity_hash: None,
            identity_salt: None,
            notes: None,
            created_at: now,
            updated_at: now,
        });
        push_audit(user, AuditEventKind::HeirAdded { heir_id: next_id });
        Ok(())
    })
}

pub fn add_heir_v2(input: HeirAddInputV2) -> Result<u64, CivError> {
    let caller = user_id();
    USERS.with(|users| {
        let mut users = users.borrow_mut();
        let now = now_secs();
        let user = users.entry(caller.clone()).or_insert_with(|| {
            let mut u = User::new(&caller, 0);
            push_audit(&mut u, AuditEventKind::UserCreated);
            u
        });
        assert_mutable(user)?;
        if input.secret_plain.trim().is_empty() {
            return Err(CivError::SecretInvalid);
        }
        let next_id = user.heirs_v2.iter().map(|h| h.id).max().unwrap_or(0) + 1;
    let mut salt = vec![0u8; 16];
    rng::fill(&mut salt);
        let hash = hash_secret_with_salt(&input.secret_plain, &salt);
        // Derive optional identity composite hash
        let (identity_hash, identity_salt) = if let Some(ref claim) = input.identity_claim {
            if !claim.trim().is_empty() {
                let mut salt_id = vec![0u8; 16];
                rng::fill(&mut salt_id);
                let hash_id = hash_secret_with_salt(claim, &salt_id);
                (Some(hash_id.to_vec()), Some(salt_id))
            } else {
                (None, None)
            }
        } else {
            (None, None)
        };
        let heir = HeirEx {
            id: next_id,
            name: input.name,
            relationship: input.relationship,
            email: input.email,
            phone: input.phone,
            address: input.address,
            principal: input.heir_principal,
            identity_secret: HeirIdentitySecret {
                kind: input.secret_kind,
                hash: hash.to_vec(),
                salt,
                status: HeirSecretStatus::Pending,
                updated_at: now,
                attempts: 0,
                last_attempt_at: None,
                next_allowed_attempt_at: None,
            },
            identity_hash,
            identity_salt,
            notes: input.notes,
            created_at: now,
            updated_at: now,
        };
        user.heirs_v2.push(heir);
        push_audit(user, AuditEventKind::HeirAdded { heir_id: next_id });
        Ok(next_id)
    })
}

pub fn verify_heir_secret(heir_id: u64, secret_plain: String) -> Result<bool, CivError> {
    let caller = user_id();
    let (result, should_audit) = USERS.with(|users| {
        let mut users = users.borrow_mut();
        if let Some(u) = users.get_mut(&caller) {
            if let Some(h) = u.heirs_v2.iter_mut().find(|h| h.id == heir_id) {
                // Exponential throttling logic
                let now = now_secs();
                if let Some(na) = h.identity_secret.next_allowed_attempt_at {
                    if now < na {
                        return (Err(CivError::RateLimited), None);
                    }
                }
                if let Some(last) = h.identity_secret.last_attempt_at {
                    if now.saturating_sub(last) > HEIR_SECRET_ATTEMPT_WINDOW_SECS {
                        h.identity_secret.attempts = 0;
                    }
                }
                h.identity_secret.attempts = h.identity_secret.attempts.saturating_add(1);
                h.identity_secret.last_attempt_at = Some(now);
                let delay = compute_secret_backoff(h.identity_secret.attempts);
                if delay > 0 {
                    h.identity_secret.next_allowed_attempt_at = Some(now.saturating_add(delay));
                }
                let recomputed = hash_secret_with_salt(&secret_plain, &h.identity_secret.salt);
                if recomputed.as_slice() == h.identity_secret.hash.as_slice() {
                    if h.identity_secret.status != HeirSecretStatus::Verified {
                        h.identity_secret.status = HeirSecretStatus::Verified;
                        h.identity_secret.updated_at = now_secs();
                        // Reset counters on success
                        h.identity_secret.attempts = 0;
                        h.identity_secret.last_attempt_at = Some(now);
                        h.identity_secret.next_allowed_attempt_at = None;
                        return (Ok(true), Some(heir_id));
                    }
                    (Ok(true), None)
                } else {
                    (Ok(false), None)
                }
            } else {
                (Err(CivError::HeirNotFound), None)
            }
        } else {
            (Err(CivError::UserNotFound), None)
        }
    });
    if let Some(hid) = should_audit {
        USERS.with(|users| {
            let mut users = users.borrow_mut();
            if let Some(u) = users.get_mut(&caller) {
                push_audit(u, AuditEventKind::HeirSecretVerified { heir_id: hid });
            }
        });
    } else if matches!(result, Err(CivError::RateLimited)) {
        USERS.with(|users| {
            let mut users = users.borrow_mut();
            if let Some(u) = users.get_mut(&caller) {
                if let Some(h) = u.heirs_v2.iter().find(|h| h.id == heir_id) {
                    let wait = h
                        .identity_secret
                        .next_allowed_attempt_at
                        .map(|na| na.saturating_sub(now_secs()))
                        .unwrap_or(0);
                    push_audit(
                        u,
                        AuditEventKind::HeirSecretBackoffRateLimited {
                            heir_id: h.id,
                            attempts: h.identity_secret.attempts,
                            wait_secs: wait,
                        },
                    );
                }
            }
        });
    }
    result
}

pub fn bind_heir_principal(heir_id: u64, principal: String) -> Result<(), CivError> {
    let caller = user_id();
    let (res, do_audit) = USERS.with(|users| {
        let mut users = users.borrow_mut();
        if let Some(u) = users.get_mut(&caller) {
            if let Err(e) = assert_mutable(u) {
                return (Err(e), false);
            }
            if let Some(h) = u.heirs_v2.iter_mut().find(|h| h.id == heir_id) {
                if h.identity_secret.status != HeirSecretStatus::Verified {
                    return (Err(CivError::SecretInvalid), false);
                }
                h.principal = Some(principal.clone());
                h.updated_at = now_secs();
                (Ok(()), true)
            } else {
                (Err(CivError::HeirNotFound), false)
            }
        } else {
            (Err(CivError::UserNotFound), false)
        }
    });
    if do_audit {
        USERS.with(|users| {
            let mut users = users.borrow_mut();
            if let Some(u) = users.get_mut(&caller) {
                push_audit(u, AuditEventKind::HeirPrincipalBound { heir_id });
            }
        });
        // Post-binding sweep: enqueue retries for any custody releases or NFT custody entries for this heir
        USERS.with(|users| {
            let users = users.borrow();
            if let Some(u) = users.get(&caller) {
                // Fungible custody
                for fc in u
                    .fungible_custody
                    .iter()
                    .filter(|c| c.heir_id == heir_id && c.released_at.is_none())
                {
                    crate::api::retry::enqueue_retry(
                        crate::api::retry::RetryKind::FungibleCustodyRelease {
                            asset_id: fc.asset_id,
                            heir_id,
                        },
                        1,
                    );
                }
                // NFT custody
                for nc in u
                    .nft_custody
                    .iter()
                    .filter(|c| c.heir_id == heir_id && c.released_at.is_none())
                {
                    crate::api::retry::enqueue_retry(
                        crate::api::retry::RetryKind::NftCustodyRelease {
                            asset_id: nc.asset_id,
                            heir_id,
                            token_id: nc.token_id,
                        },
                        1,
                    );
                }
            }
        });
    }
    res
}

pub fn list_heirs() -> Vec<Heir> {
    let user = user_id();
    USERS.with(|users| {
        let users = users.borrow();
        users
            .get(&user)
            .map(|u| u.heirs.clone())
            .unwrap_or_default()
    })
}

pub fn list_heirs_v2() -> Vec<HeirEx> {
    let user = user_id();
    USERS.with(|users| {
        let users = users.borrow();
        users
            .get(&user)
            .map(|u| u.heirs_v2.clone())
            .unwrap_or_default()
    })
}

pub fn remove_heir(heir_id: u64) -> Result<(), CivError> {
    let user = user_id();
    USERS.with(|users| {
        let mut users = users.borrow_mut();
        if let Some(u) = users.get_mut(&user) {
            assert_mutable(u)?;
            let existed = u.heirs.iter().any(|h| h.id == heir_id)
                || u.heirs_v2.iter().any(|h| h.id == heir_id);
            if !existed {
                return Err(CivError::HeirNotFound);
            }
            u.heirs.retain(|h| h.id != heir_id);
            u.heirs_v2.retain(|h| h.id != heir_id);
            u.distributions.retain(|d| d.heir_id != heir_id);
            u.distributions_v2.retain(|d| d.heir_id != heir_id);
            push_audit(u, AuditEventKind::HeirRemoved { heir_id });
            Ok(())
        } else {
            Err(CivError::UserNotFound)
        }
    })
}

pub fn update_heir(heir_id: u64, new_heir: HeirInput) -> Result<(), CivError> {
    let caller = user_id();
    let (res, audit) = USERS.with(|users| {
        let mut users = users.borrow_mut();
        if let Some(u) = users.get_mut(&caller) {
            if let Err(e) = assert_mutable(u) {
                return (Err(e), false);
            }
            if let Some(existing) = u.heirs.iter_mut().find(|h| h.id == heir_id) {
                existing.name = new_heir.name;
                existing.relationship = new_heir.relationship;
                existing.email = new_heir.email;
                existing.phone = new_heir.phone;
                existing.address = new_heir.address;
                existing.updated_at = now_secs();
                (Ok(()), true)
            } else {
                (Err(CivError::HeirNotFound), false)
            }
        } else {
            (Err(CivError::UserNotFound), false)
        }
    });
    if audit {
        USERS.with(|users| {
            let mut users = users.borrow_mut();
            if let Some(u) = users.get_mut(&caller) {
                push_audit(u, AuditEventKind::HeirUpdated { heir_id });
            }
        });
    }
    res
}

// Legacy heir_claim input. Renamed field `principal` -> `heirPrincipal` for Candid to
// avoid collision/confusion with primitive `principal` keyword in some generators.
// We keep backward compatibility by accepting the legacy incoming field name via alias.
#[derive(candid::CandidType, serde::Deserialize)]
pub struct HeirClaimInput {
    pub heir_id: u64,
    pub secret_plain: Option<String>,
    #[serde(alias = "principal", rename = "heirPrincipal")]
    pub heir_principal: Option<String>,
}
#[derive(candid::CandidType, serde::Deserialize)]
pub struct HeirClaimResult {
    pub verified: bool,
    pub principal_bound: bool,
}

// Legacy combined claim flow: optionally verify secret and/or bind principal in one call.
// Returns whether secret ended up verified and whether principal was newly bound this call.
pub fn heir_claim(input: HeirClaimInput) -> Result<HeirClaimResult, CivError> {
    let caller = user_id();
    USERS.with(|users| {
        let mut users = users.borrow_mut();
        if let Some(u) = users.get_mut(&caller) {
            // Find heir in v2 list (preferred) falling back to legacy if needed.
            if let Some(idx) = u.heirs_v2.iter().position(|h| h.id == input.heir_id) {
                let (verified, principal_bound, ver_evt, princ_evt, rate_limited) = {
                    let h = &mut u.heirs_v2[idx];
                    let mut verified = h.identity_secret.status == HeirSecretStatus::Verified;
                    let mut ver_event = false;
                    let mut rate_limited = false;
                    if !verified {
                        if let Some(sec) = &input.secret_plain {
                            // Throttling block for legacy combined claim path
                            let now = now_secs();
                            if let Some(na) = h.identity_secret.next_allowed_attempt_at {
                                if now < na {
                                    rate_limited = true;
                                }
                            }
                            if !rate_limited {
                                if let Some(last) = h.identity_secret.last_attempt_at {
                                    if now.saturating_sub(last) > HEIR_SECRET_ATTEMPT_WINDOW_SECS {
                                        h.identity_secret.attempts = 0;
                                    }
                                }
                            }
                            if !rate_limited {
                                h.identity_secret.attempts =
                                    h.identity_secret.attempts.saturating_add(1);
                                h.identity_secret.last_attempt_at = Some(now);
                                let delay = compute_secret_backoff(h.identity_secret.attempts);
                                if delay > 0 {
                                    h.identity_secret.next_allowed_attempt_at =
                                        Some(now.saturating_add(delay));
                                }
                                let recomputed =
                                    hash_secret_with_salt(sec, &h.identity_secret.salt);
                                if recomputed.as_slice() == h.identity_secret.hash.as_slice() {
                                    h.identity_secret.status = HeirSecretStatus::Verified;
                                    h.identity_secret.updated_at = now_secs();
                                    h.identity_secret.attempts = 0;
                                    h.identity_secret.last_attempt_at = Some(now);
                                    h.identity_secret.next_allowed_attempt_at = None; // reset on success
                                    verified = true;
                                    ver_event = true;
                                }
                            }
                        }
                    }
                    let mut principal_bound = false;
                    let mut princ_event = false;
                    if let Some(p) = &input.heir_principal {
                        if h.principal.as_ref() != Some(p) {
                            h.principal = Some(p.clone());
                            principal_bound = true;
                            princ_event = true;
                        }
                    }
                    (
                        verified,
                        principal_bound,
                        ver_event,
                        princ_event,
                        rate_limited,
                    )
                };
                if rate_limited {
                    // Emit audit and return rate limited error
                    if let Some(h) = u.heirs_v2.iter().find(|h| h.id == input.heir_id) {
                        let wait = h
                            .identity_secret
                            .next_allowed_attempt_at
                            .map(|na| na.saturating_sub(now_secs()))
                            .unwrap_or(0);
                        push_audit(
                            u,
                            AuditEventKind::HeirSecretBackoffRateLimited {
                                heir_id: input.heir_id,
                                attempts: h.identity_secret.attempts,
                                wait_secs: wait,
                            },
                        );
                    }
                    return Err(CivError::RateLimited);
                }
                // audit after releasing &mut h borrow
                if ver_evt {
                    push_audit(
                        u,
                        AuditEventKind::HeirSecretVerified {
                            heir_id: input.heir_id,
                        },
                    );
                }
                if princ_evt {
                    push_audit(
                        u,
                        AuditEventKind::HeirPrincipalBound {
                            heir_id: input.heir_id,
                        },
                    );
                }
                Ok(HeirClaimResult {
                    verified,
                    principal_bound,
                })
            } else {
                Err(CivError::HeirNotFound)
            }
        } else {
            Err(CivError::UserNotFound)
        }
    })
}

pub fn create_claim_link(heir_id: u64) -> Result<ClaimLinkUnsealed, CivError> {
    let caller = user_id();
    USERS.with(|users| {
        let mut users = users.borrow_mut();
        if let Some(u) = users.get_mut(&caller) {
            if !u.heirs_v2.iter().any(|h| h.id == heir_id) {
                return Err(CivError::HeirNotFound);
            }
            let mut salt = vec![0u8; 16];
            rng::fill(&mut salt);
            // Generate 6-digit code using unbiased numeric code generator (async prefilled buffer)
            // For now we attempt sync path; if buffer underflow occurs it traps instructing to init RNG.
            // TODO: consider making create_claim_link async to use rng::numeric_code.
            let mut tmp = [0u8; 4];
            rng::fill(&mut tmp);
            let raw = u32::from_le_bytes(tmp) % 1_000_000; // acceptable minimal bias for 2^32 % 1e6 (~ <1e-6)
            let code_plain = format!("{:06}", raw);
            let hash = hash_secret_with_salt(&code_plain, &salt);
            let next_id = u.claim_links.iter().map(|c| c.id).max().unwrap_or(0) + 1;
            u.claim_links.push(ClaimLink {
                id: next_id,
                heir_id,
                code_hash: hash.to_vec(),
                salt,
                created_at: now_secs(),
                used: false,
            });
            push_audit(
                u,
                AuditEventKind::ClaimLinkCreated {
                    heir_id,
                    link_id: next_id,
                },
            );
            Ok(ClaimLinkUnsealed {
                link_id: next_id,
                code_plain,
            })
        } else {
            Err(CivError::UserNotFound)
        }
    })
}

pub fn heir_begin_claim(link_id: u64, code_plain: String) -> Result<u64, CivError> {
    let caller = user_id();
    USERS.with(|users| {
        let mut users = users.borrow_mut();
        if let Some(u) = users.get_mut(&caller) {
            // Find index first to avoid holding sub-borrow during audit logging
            let link_index = match u.claim_links.iter().position(|l| l.id == link_id) {
                Some(idx) => idx,
                None => return Err(CivError::Other("claim_link_not_found".into())),
            };
            if u.claim_links[link_index].used {
                return Err(CivError::Other("link_used".into()));
            }
            let recomputed = hash_secret_with_salt(&code_plain, &u.claim_links[link_index].salt);
            if recomputed.as_slice() != u.claim_links[link_index].code_hash.as_slice() {
                return Err(CivError::SecretInvalid);
            }
            u.claim_links[link_index].used = true;
            let heir_id = u.claim_links[link_index].heir_id;
            let session_id = u.sessions.iter().map(|s| s.id).max().unwrap_or(0) + 1;
            u.sessions.push(HeirSession {
                id: session_id,
                heir_id,
                started_at: now_secs(),
                verified_secret: false,
                bound_principal: false,
                expires_at: now_secs() + 24 * 60 * 60, // 24h default
            });
            push_audit(
                u,
                AuditEventKind::HeirSessionStarted {
                    heir_id,
                    session_id,
                },
            );
            Ok(session_id)
        } else {
            Err(CivError::UserNotFound)
        }
    })
}

// Optional identity claim verification separate from secret (if configured).
pub fn heir_verify_identity_session(
    session_id: u64,
    identity_claim: String,
) -> Result<bool, CivError> {
    let caller = user_id();
    USERS.with(|users| {
        let mut users = users.borrow_mut();
        if let Some(u) = users.get_mut(&caller) {
            let sess = u
                .sessions
                .iter()
                .find(|s| s.id == session_id)
                .ok_or(CivError::Other("session_not_found".into()))?;
            if now_secs() > sess.expires_at {
                return Err(CivError::SessionExpired);
            }
            let heir = u
                .heirs_v2
                .iter()
                .find(|h| h.id == sess.heir_id)
                .ok_or(CivError::HeirNotFound)?;
            match (&heir.identity_hash, &heir.identity_salt) {
                (Some(h), Some(s)) => {
                    let recomputed = hash_secret_with_salt(&identity_claim, s);
                    Ok(constant_time_eq(&recomputed, h))
                }
                _ => Ok(true), // no identity claim configured -> treat as pass
            }
        } else {
            Err(CivError::UserNotFound)
        }
    })
}

pub fn heir_verify_secret_session(session_id: u64, secret_plain: String) -> Result<bool, CivError> {
    let caller = user_id();
    USERS.with(|users| {
        let mut users = users.borrow_mut();
        let u = users.get_mut(&caller).ok_or(CivError::UserNotFound)?;

        // find session index
        let sess_idx = u
            .sessions
            .iter()
            .position(|s| s.id == session_id)
            .ok_or(CivError::Other("session_not_found".into()))?;
        if now_secs() > u.sessions[sess_idx].expires_at {
            return Err(CivError::SessionExpired);
        }
        if u.sessions[sess_idx].verified_secret {
            return Ok(true);
        }

        let heir_id = u.sessions[sess_idx].heir_id;
        let heir_idx = u
            .heirs_v2
            .iter()
            .position(|h| h.id == heir_id)
            .ok_or(CivError::HeirNotFound)?;

        // ---- Throttling & data capture block (no push_audit here) ----
        let (salt, expected_hash, now, rate_limited, attempts) = {
            let hsec = &mut u.heirs_v2[heir_idx].identity_secret;
            let now = now_secs();
            let mut rate_limited = false;

            if let Some(na) = hsec.next_allowed_attempt_at {
                if now < na {
                    hsec.last_attempt_at = Some(now);
                    rate_limited = true;
                }
            }

            if !rate_limited {
                if let Some(last) = hsec.last_attempt_at {
                    if now.saturating_sub(last) > HEIR_SECRET_ATTEMPT_WINDOW_SECS {
                        hsec.attempts = 0;
                    }
                }
                hsec.attempts = hsec.attempts.saturating_add(1);
                hsec.last_attempt_at = Some(now);
                let delay = compute_secret_backoff(hsec.attempts);
                if delay > 0 {
                    hsec.next_allowed_attempt_at = Some(now.saturating_add(delay));
                }
            }

            (
                hsec.salt.clone(),
                hsec.hash.clone(),
                now,
                rate_limited,
                hsec.attempts,
            )
        };

        // Now the field borrow is dropped; safe to mutate `u` elsewhere.
        if rate_limited {
            let wait = u.heirs_v2[heir_idx]
                .identity_secret
                .next_allowed_attempt_at
                .map(|na| na.saturating_sub(now))
                .unwrap_or(0);
            push_audit(
                u,
                AuditEventKind::HeirSecretBackoffRateLimited {
                    heir_id,
                    attempts,
                    wait_secs: wait,
                },
            );
            return Err(CivError::RateLimited);
        }

        // Recompute outside the previous borrow scope
        let recomputed = hash_secret_with_salt(&secret_plain, &salt);
        if recomputed.as_slice() == expected_hash.as_slice() {
            // Re-borrow mutably to update status fields after successful verification
            {
                let hsec2 = &mut u.heirs_v2[heir_idx].identity_secret;
                hsec2.status = HeirSecretStatus::Verified;
                hsec2.updated_at = now;
                hsec2.attempts = 0;
                hsec2.last_attempt_at = Some(now);
                hsec2.next_allowed_attempt_at = None;
            }
            u.sessions[sess_idx].verified_secret = true;
            push_audit(
                u,
                AuditEventKind::HeirSessionSecretVerified {
                    heir_id,
                    session_id,
                },
            );
            push_audit(u, AuditEventKind::HeirSecretVerified { heir_id });
            Ok(true)
        } else {
            Ok(false)
        }
    })
}

pub fn heir_bind_principal_session(session_id: u64, principal: String) -> Result<(), CivError> {
    let caller = user_id();
    let (res, audit_heir) = USERS.with(|users| {
        let mut users = users.borrow_mut();
        if let Some(u) = users.get_mut(&caller) {
            // locate session
            let sess_idx = match u.sessions.iter().position(|s| s.id == session_id) {
                Some(i) => i,
                None => return (Err(CivError::Other("session_not_found".into())), None),
            };
            if now_secs() > u.sessions[sess_idx].expires_at {
                return (Err(CivError::SessionExpired), None);
            }
            if !u.sessions[sess_idx].verified_secret {
                return (Err(CivError::SecretInvalid), None);
            }
            let heir_id = u.sessions[sess_idx].heir_id;
            if let Some(hidx) = u.heirs_v2.iter().position(|h| h.id == heir_id) {
                u.heirs_v2[hidx].principal = Some(principal);
                u.sessions[sess_idx].bound_principal = true;
                (Ok(()), Some(heir_id))
            } else {
                (Err(CivError::HeirNotFound), None)
            }
        } else {
            (Err(CivError::UserNotFound), None)
        }
    });
    if let Some(hid) = audit_heir {
        USERS.with(|users| {
            let mut users = users.borrow_mut();
            if let Some(u) = users.get_mut(&caller) {
                push_audit(u, AuditEventKind::HeirPrincipalBound { heir_id: hid });
            }
        });
    }
    res
}

// Heir chooses payout preference for a specific asset distribution prior to execution.
// Rules:
// - Session must exist & secret verified.
// - Estate not executed and share not yet transferred.
// - Preference must be compatible with asset kind.
// - Principal bound when selecting ToPrincipal or CkWithdraw (and for NFT principal delivery).
pub fn heir_set_payout_preference_session(
    session_id: u64,
    asset_id: u64,
    preference: PayoutPreference,
) -> Result<(), CivError> {
    let caller = user_id();
    USERS.with(|users| {
        let mut users = users.borrow_mut();
        let u = users.get_mut(&caller).ok_or(CivError::UserNotFound)?;

        // Estate must not be fully executed
        if matches!(u.phase, EstatePhase::Executed) {
            return Err(CivError::EstateLocked);
        }

        // Locate session (index, not ref, so we can reborrow later)
        let sess_idx = u
            .sessions
            .iter()
            .position(|s| s.id == session_id)
            .ok_or_else(|| CivError::Other("session_not_found".into()))?;
        if now_secs() > u.sessions[sess_idx].expires_at {
            return Err(CivError::SessionExpired);
        }
        if !u.sessions[sess_idx].verified_secret {
            return Err(CivError::SecretInvalid);
        }
        let heir_id = u.sessions[sess_idx].heir_id;

        // Resolve indices (avoid long-lived & refs)
        let heir_idx = u
            .heirs_v2
            .iter()
            .position(|h| h.id == heir_id)
            .ok_or(CivError::HeirNotFound)?;
        let dist = u
            .distributions_v2
            .iter()
            .find(|d| d.asset_id == asset_id && d.heir_id == heir_id)
            .ok_or(CivError::DistributionAssetNotFound)?;
        let share_pref = dist.payout_preference.clone();
        let asset = u
            .assets
            .iter()
            .find(|a| a.id == asset_id)
            .ok_or(CivError::AssetNotFound)?;
        let kind = crate::models::infer_asset_kind(&asset.asset_type);

        // Copy minimal data we need to decide, then drop the short-lived borrows.
        let heir_has_principal = u.heirs_v2[heir_idx].principal.is_some();

        // Compatibility checks
        let allowed = match kind {
            AssetKind::Fungible => matches!(
                preference,
                PayoutPreference::ToPrincipal | PayoutPreference::ToCustody
            ),
            AssetKind::ChainWrapped => matches!(
                preference,
                PayoutPreference::ToPrincipal
                    | PayoutPreference::ToCustody
                    | PayoutPreference::CkWithdraw
            ),
            AssetKind::Nft => matches!(
                preference,
                PayoutPreference::ToPrincipal | PayoutPreference::ToCustody
            ),
            AssetKind::Document => false,
        };
        if !allowed {
            return Err(CivError::InvalidPayoutPreference);
        }
        // Principal requirement
        if matches!(
            preference,
            PayoutPreference::ToPrincipal | PayoutPreference::CkWithdraw
        ) && !heir_has_principal
        {
            return Err(CivError::InvalidPayoutPreference);
        }
        // If NFT and ToPrincipal ensure principal bound
        if matches!(kind, AssetKind::Nft)
            && matches!(preference, PayoutPreference::ToPrincipal)
            && !heir_has_principal
        {
            return Err(CivError::InvalidPayoutPreference);
        }
        // Prevent change if already transferred (check transfers ledger for matching asset+heir)
        if u.transfers
            .iter()
            .any(|t| t.asset_id == Some(asset_id) && t.heir_id == Some(heir_id))
        {
            return Err(CivError::EstateLocked);
        }

        // Rate limiting: per (heir,asset) max changes per day and cooldown between changes
        const PREF_OVERRIDE_DAILY_MAX: u32 = 3; // up to 3 changes per day
        const PREF_OVERRIDE_COOLDOWN_SECS: u64 = 2 * 60 * 60; // 2h cooldown between changes
        let now = now_secs(); // <-- moved before first use
        let day_epoch = now / 86_400; // simple day bucket
        let rates_vec = u.payout_override_rates.get_or_insert_with(|| Vec::new());
        let mut rate_entry_index: Option<usize> = None;
        for (idx, r) in rates_vec.iter().enumerate() {
            if r.heir_id == heir_id && r.asset_id == asset_id {
                rate_entry_index = Some(idx);
                break;
            }
        }
        let mut blocked = false;
        if let Some(idx) = rate_entry_index {
            let rec = &mut rates_vec[idx];
            if rec.day_epoch == day_epoch {
                if rec.count >= PREF_OVERRIDE_DAILY_MAX {
                    blocked = true;
                } else if now.saturating_sub(rec.last_set_at) < PREF_OVERRIDE_COOLDOWN_SECS {
                    blocked = true;
                }
            } else {
                // new day -> reset counters
                rec.day_epoch = day_epoch;
                rec.count = 0;
            }
            if !blocked {
                rec.count = rec.count.saturating_add(1);
                rec.last_set_at = now;
            }
        } else {
            rates_vec.push(crate::models::user::PayoutOverrideRate {
                heir_id,
                asset_id,
                day_epoch,
                count: 1,
                last_set_at: now,
            });
        }
        if blocked {
            push_audit(
                u,
                AuditEventKind::HeirPayoutPreferenceRateLimited { heir_id, asset_id },
            );
            return Err(CivError::RateLimited);
        }

        // Find existing override index (avoid holding &mut while also needing &mut u for audit)
        let existing_idx = u
            .payout_overrides
            .iter()
            .position(|o| o.asset_id == asset_id && o.heir_id == heir_id);

        let from_pref = existing_idx
            .and_then(|i| u.payout_overrides.get(i))
            .map(|p| p.payout_preference.clone())
            .unwrap_or(share_pref);

        // Update / insert override
        if let Some(i) = existing_idx {
            let rec = &mut u.payout_overrides[i];
            rec.payout_preference = preference.clone();
            rec.set_at = now;
        } else {
            u.payout_overrides.push(crate::models::HeirPayoutOverride {
                heir_id,
                asset_id,
                payout_preference: preference.clone(),
                set_at: now,
            });
        }

        // All borrows ended; safe to audit
        push_audit(
            u,
            AuditEventKind::HeirPayoutPreferenceSet {
                heir_id,
                asset_id,
                from: from_pref,
                to: preference,
            },
        );
        Ok(())
    })
}
