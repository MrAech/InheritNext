use crate::audit::push_audit;
use crate::models::*;
use crate::storage::USERS;
use crate::time::now_secs;
use ic_cdk::api::msg_caller;

// Caller id as text key
pub fn user_id() -> String {
    msg_caller().to_text()
}

// Guard that estate is still mutable (Draft or Warning)
pub fn assert_mutable(u: &User) -> Result<(), CivError> {
    match u.phase {
        EstatePhase::Draft | EstatePhase::Warning => Ok(()),
        _ => Err(CivError::EstateLocked),
    }
}

// Phase advancement helper reused by maintenance/executor paths
pub fn maybe_advance_phase(u: &mut User) {
    let now = now_secs();
    match u.phase {
        EstatePhase::Draft => {
            if u.timer_expiry > 0 && u.timer_expiry.saturating_sub(now) <= WARNING_WINDOW_SECS {
                let from = u.phase.clone();
                u.phase = EstatePhase::Warning;
                u.warning_started_at = Some(now);
                push_audit(
                    u,
                    AuditEventKind::PhaseChanged {
                        from,
                        to: u.phase.clone(),
                    },
                );
            }
        }
        EstatePhase::Warning => {
            if u.timer_expiry > 0 && now >= u.timer_expiry {
                let from = u.phase.clone();
                u.phase = EstatePhase::Locked;
                u.locked_at = Some(now);
                push_audit(
                    u,
                    AuditEventKind::PhaseChanged {
                        from,
                        to: u.phase.clone(),
                    },
                );
            }
        }
        _ => {}
    }
}

// Role/authorization helpers
// Determine if caller is the estate owner (principal matches stored user id)
pub fn is_owner(u: &User) -> bool {
    u.user == user_id()
}

// Validate a session for current caller; returns (heir_id, session_index)
pub fn validate_heir_session(u: &mut User, session_id: u64) -> Result<(u64, usize), CivError> {
    // Only usable after estate locked (heirs claim), but allow earlier read if needed.
    if u.sessions.is_empty() {
        return Err(CivError::HeirSessionUnauthorized);
    }
    let idx = u
        .sessions
        .iter()
        .position(|s| s.id == session_id)
        .ok_or(CivError::HeirSessionUnauthorized)?;
    let sess = &u.sessions[idx];
    if now_secs() > sess.expires_at {
        return Err(CivError::SessionExpired);
    }
    if !sess.verified_secret {
        return Err(CivError::SecretInvalid);
    }
    Ok((sess.heir_id, idx))
}

// Convenience: require owner else error Unauthorized
pub fn require_owner(u: &User) -> Result<(), CivError> {
    if is_owner(u) {
        Ok(())
    } else {
        Err(CivError::Unauthorized)
    }
}
