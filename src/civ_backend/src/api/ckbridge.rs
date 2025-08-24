use crate::api::bridge::submit_bridge_withdraw;
use crate::api::common::user_id;
use crate::audit::push_audit;
use crate::models::*;
use crate::storage::USERS;
use crate::time::now_secs;
use candid::Principal;
use num_traits::ToPrimitive; // for BigUint.to_u128()

// ---------------------------
// Chain-wrapped (ck) asset withdrawal lifecycle
// ---------------------------

pub fn list_ck_withdraws() -> Vec<CkWithdrawRecord> {
    let caller = user_id();
    USERS.with(|users| {
        let users = users.borrow();
        users
            .get(&caller)
            .map(|u| u.ck_withdraws.clone())
            .unwrap_or_default()
    })
}

pub fn request_ck_withdraw(session_id: u64, asset_id: u64, heir_id: u64) -> Result<(), CivError> {
    let caller = user_id();
    USERS.with(|users| {
        let mut users = users.borrow_mut();
        if let Some(u) = users.get_mut(&caller) {
            if !matches!(u.phase, EstatePhase::Locked | EstatePhase::Executed) {
                return Err(CivError::EstateNotReady);
            }
            let sess = u
                .sessions
                .iter()
                .find(|s| s.id == session_id)
                .ok_or(CivError::Other("session_not_found".into()))?;
            if now_secs() > sess.expires_at {
                return Err(CivError::SessionExpired);
            }
            if sess.heir_id != heir_id {
                return Err(CivError::Unauthorized);
            }

            // Mutate the record inside a tight scope, build the audit event, then drop the borrow.
            let event_opt = {
                let rec = u
                    .ck_withdraws
                    .iter_mut()
                    .find(|r| r.asset_id == asset_id && r.heir_id == heir_id)
                    .ok_or(CivError::AssetNotFound)?;
                if rec.requested_at.is_none() {
                    rec.requested_at = Some(now_secs());
                    rec.bridge_status = Some(BridgeStatus::Requested);
                    Some(AuditEventKind::CkWithdrawRequested { asset_id, heir_id })
                } else {
                    return Err(CivError::Other("already_requested".into()));
                }
            };

            if let Some(ev) = event_opt {
                push_audit(u, ev);
            }
            Ok(())
        } else {
            Err(CivError::UserNotFound)
        }
    })
}

pub async fn submit_ck_withdraw(
    session_id: u64,
    asset_id: u64,
    heir_id: u64,
    l1_address: String,
) -> Result<BridgeTxInfo, CivError> {
    let caller = user_id();
    // Phase 1 capture essentials
    let (token_canister_txt, amount, chain_kind_opt, need_fee_quote) = USERS.with(|users| {
        let mut users = users.borrow_mut();
        let u = users.get_mut(&caller).ok_or(CivError::UserNotFound)?;
        let sess = u
            .sessions
            .iter()
            .find(|s| s.id == session_id)
            .ok_or(CivError::Other("session_not_found".into()))?;
        if now_secs() > sess.expires_at {
            return Err(CivError::SessionExpired);
        }
        if sess.heir_id != heir_id {
            return Err(CivError::Unauthorized);
        }
        let rec = u
            .ck_withdraws
            .iter_mut()
            .find(|r| r.asset_id == asset_id && r.heir_id == heir_id)
            .ok_or(CivError::AssetNotFound)?;
        if rec.requested_at.is_none() {
            return Err(CivError::EstateNotReady);
        }
        if rec.completed_at.is_some() {
            return Err(CivError::Other("bridge_already_completed".into()));
        }
        let asset_ref = u
            .assets
            .iter()
            .find(|a| a.id == asset_id)
            .ok_or(CivError::AssetNotFound)?;
        let can_txt = asset_ref
            .token_canister
            .clone()
            .ok_or(CivError::AssetNotFound)?;
        let chain_kind_opt = asset_ref.chain_wrapped.clone();
        let need_fee = rec.quoted_fee.is_none();
        Ok((can_txt, rec.amount, chain_kind_opt, need_fee))
    })?;

    // Phase 2 fee preflight
    let mut fee_quote: Option<u128> = None;
    if need_fee_quote {
        if let Ok(p) = Principal::from_text(&token_canister_txt) {
            if let Ok(call) = std::panic::catch_unwind(|| {
                ic_cdk::call::Call::unbounded_wait(p, "icrc1_fee").with_arg(())
            }) {
                if let Ok(reply) = call.await {
                    if let Ok(tuple) = reply.candid_tuple::<(candid::Nat,)>() {
                        fee_quote = tuple.0 .0.to_u128();
                    }
                }
            }
        }
    }

    // Phase 3 submit
    let can_principal = Principal::from_text(&token_canister_txt).map_err(|_| {
        CivError::bridge_err(BridgeErrorKind::InvalidCanister, "invalid_token_canister")
    })?;
    let chain_kind = chain_kind_opt.ok_or(CivError::bridge_err(
        BridgeErrorKind::Other,
        "missing_chain_kind",
    ))?;
    let tx_res = submit_bridge_withdraw(can_principal, amount, &l1_address, chain_kind).await;

    // Phase 4 persist (two-phase to avoid aliasing mutable borrows)
    let (info, fee_audit_opt, fail_audit_opt) = USERS.with(|users| {
        let mut users = users.borrow_mut();
        let u = users.get_mut(&caller).ok_or(CivError::UserNotFound)?;

        // Inner block mutates the record only; collect the events to emit after the borrow ends.
        let (info, fee_audit_opt, fail_audit_opt) = {
            let rec = u
                .ck_withdraws
                .iter_mut()
                .find(|r| r.asset_id == asset_id && r.heir_id == heir_id)
                .ok_or(CivError::AssetNotFound)?;
            let now = now_secs();

            let mut fee_audit: Option<AuditEventKind> = None;
            if let Some(fv) = fee_quote {
                if rec.quoted_fee.is_none() {
                    rec.quoted_fee = Some(fv);
                    rec.bridge_status = Some(BridgeStatus::FeeQuoted);
                    fee_audit = Some(AuditEventKind::CkWithdrawFeeQuoted {
                        asset_id,
                        heir_id,
                        fee: fv,
                    });
                }
            }

            let mut fail_audit: Option<AuditEventKind> = None;
            match &tx_res {
                Ok(tx_id_opt) => {
                    rec.bridge_status = Some(BridgeStatus::Submitted);
                    rec.bridge_tx_id = tx_id_opt.clone();
                }
                Err(err) => {
                    let (msg, info_opt) = match err {
                        CivError::Bridge(info) => (info.message.clone(), Some(info.clone())),
                        CivError::Other(s) => (s.clone(), None),
                        _ => (format!("{:?}", err), None),
                    };
                    rec.bridge_status = Some(BridgeStatus::Failed(msg.clone()));
                    rec.bridge_error = info_opt;
                    fail_audit = Some(AuditEventKind::CkWithdrawFailed {
                        asset_id,
                        heir_id,
                        error: msg,
                    });
                }
            }

            let info = BridgeTxInfo {
                asset_id,
                heir_id,
                l1_address: l1_address.clone(),
                submitted_at: now,
                tx_id: tx_res.as_ref().ok().and_then(|x| x.clone()),
                consecutive_misses: Some(0),
                notfound_terminal: Some(false),
            };
            (info, fee_audit, fail_audit)
        };

        // Now safe to push audits & modify vectors
        if let Some(ev) = fee_audit_opt.clone() {
            push_audit(u, ev);
        }
        u.bridge_txs.push(info.clone());
        push_audit(u, AuditEventKind::CkWithdrawSubmitted { asset_id, heir_id });
        if let Some(ev) = fail_audit_opt.clone() {
            push_audit(u, ev);
        }
        Ok((info, fee_audit_opt, fail_audit_opt))
    })?;

    tx_res.map(|_| info.clone())
}

// ---------------------------
// poll_ck_withdraw
// ---------------------------

pub async fn poll_ck_withdraw(
    session_id: u64,
    asset_id: u64,
    heir_id: u64,
) -> Result<(), CivError> {
    let caller = user_id();

    // Gather immutable copies first (can't hold borrows across await)
    let (can_txt_opt, tx_id_opt) = USERS.with(|users| {
        let users = users.borrow();
        if let Some(u) = users.get(&caller) {
            let sess_ok = u
                .sessions
                .iter()
                .any(|s| s.id == session_id && s.heir_id == heir_id && now_secs() <= s.expires_at);
            if !sess_ok {
                return (None, None);
            }
            if let Some(rec) = u
                .ck_withdraws
                .iter()
                .find(|r| r.asset_id == asset_id && r.heir_id == heir_id)
            {
                let can_txt = u
                    .assets
                    .iter()
                    .find(|a| a.id == asset_id)
                    .and_then(|a| a.token_canister.clone());
                return (can_txt, rec.bridge_tx_id.clone());
            }
        }
        (None, None)
    });

    if can_txt_opt.is_none() {
        return Err(CivError::AssetNotFound);
    }
    if tx_id_opt.is_none() {
        return Ok(()); // nothing to poll yet
    }

    let can_principal = Principal::from_text(can_txt_opt.unwrap()).map_err(|_| {
        CivError::bridge_err(BridgeErrorKind::InvalidCanister, "invalid_token_canister")
    })?;
    let tx_id = tx_id_opt.unwrap();

    // Try ckBTC status first
    #[derive(candid::CandidType, serde::Deserialize)]
    struct BtcStatusOk {
        block_index: u64,
    }
    #[derive(candid::CandidType, serde::Deserialize)]
    enum BtcStatusErr {
        Pending,
        NotFound,
        Other,
    }

    let mut completed = false;
    let mut not_found_detected = false;
    if let Ok(call) = std::panic::catch_unwind(|| {
        ic_cdk::call::Call::unbounded_wait(can_principal, "retrieve_btc_status")
            .with_arg((tx_id.clone(),))
    }) {
        if let Ok(reply) = call.await {
            if let Ok(tuple) = reply.candid_tuple::<(Result<BtcStatusOk, BtcStatusErr>,)>() {
                match tuple.0 {
                    Ok(_ok) => {
                        completed = true;
                    }
                    Err(BtcStatusErr::Pending) => {
                        completed = false;
                    }
                    Err(BtcStatusErr::NotFound) => {
                        not_found_detected = true;
                    }
                    _ => {
                        completed = false;
                    }
                }
            }
        }
    }

    // ckETH status (retrieve_eth_status : (nat64) -> (RetrieveEthStatus))
    if !completed {
        #[derive(candid::CandidType, serde::Deserialize)]
        enum TxFinalizedStatus {
            Success {
                transaction_hash: String,
                effective_transaction_fee: Option<candid::Nat>,
            },
            Reimbursed {
                transaction_hash: String,
                reimbursed_amount: candid::Nat,
                reimbursed_in_block: candid::Nat,
            },
            PendingReimbursement {
                transaction_hash: String,
            },
        }
        #[derive(candid::CandidType, serde::Deserialize)]
        enum RetrieveEthStatus {
            NotFound,
            Pending,
            TxCreated,
            TxSent { transaction_hash: String },
            TxFinalized(TxFinalizedStatus),
        }

        let mut eth_tx_hash: Option<String> = None;
        let mut eth_effective_fee: Option<u128> = None;
        let mut eth_failed: Option<(String, BridgeErrorInfo)> = None;
        let mut eth_reimbursed: bool = false;

        if let Ok(call) = std::panic::catch_unwind(|| {
            ic_cdk::call::Call::unbounded_wait(can_principal, "retrieve_eth_status")
                .with_arg((tx_id.clone().parse::<u64>().unwrap_or_default(),))
        }) {
            if let Ok(reply) = call.await {
                if let Ok(tuple) = reply.candid_tuple::<(RetrieveEthStatus,)>() {
                    use RetrieveEthStatus::*;
                    match tuple.0 {
                        Pending | TxCreated | TxSent { .. } => { /* still pending */ }
                        TxFinalized(final_status) => {
                            use TxFinalizedStatus::*;
                            match final_status {
                                Success {
                                    transaction_hash,
                                    effective_transaction_fee,
                                } => {
                                    eth_tx_hash = Some(transaction_hash);
                                    eth_effective_fee =
                                        effective_transaction_fee.and_then(|n| n.0.to_u128());
                                    completed = true;
                                }
                                Reimbursed {
                                    transaction_hash,
                                    reimbursed_amount,
                                    reimbursed_in_block: _,
                                } => {
                                    let msg = format!("reimbursed:{}", reimbursed_amount);
                                    eth_failed = Some((
                                        msg.clone(),
                                        BridgeErrorInfo {
                                            kind: BridgeErrorKind::Reimbursed,
                                            message: msg.clone(),
                                        },
                                    ));
                                    eth_reimbursed = true;
                                    eth_tx_hash = Some(transaction_hash);
                                }
                                PendingReimbursement { transaction_hash } => {
                                    let msg = "pending_reimbursement".to_string();
                                    eth_failed = Some((
                                        msg.clone(),
                                        BridgeErrorInfo {
                                            kind: BridgeErrorKind::Timeout,
                                            message: msg,
                                        },
                                    ));
                                    eth_tx_hash = Some(transaction_hash);
                                }
                            }
                        }
                        NotFound => {
                            not_found_detected = true;
                        }
                    }
                }
            }
        }

        // persist eth specific details (build event, then push it after borrow ends)
        if eth_tx_hash.is_some() || eth_effective_fee.is_some() || eth_failed.is_some() {
            USERS.with(|users| {
                let mut users = users.borrow_mut();
                if let Some(u) = users.get_mut(&caller) {
                    let event_opt = {
                        if let Some(rec) = u
                            .ck_withdraws
                            .iter_mut()
                            .find(|r| r.asset_id == asset_id && r.heir_id == heir_id)
                        {
                            if let Some(h) = eth_tx_hash.clone() {
                                rec.tx_hash = Some(h);
                            }
                            if let Some(fee) = eth_effective_fee {
                                rec.effective_fee = Some(fee);
                            }
                            if let Some((msg, info)) = eth_failed.clone() {
                                if !completed {
                                    if eth_reimbursed {
                                        rec.bridge_status = Some(BridgeStatus::Reimbursed);
                                        rec.bridge_error = Some(info);
                                        Some(AuditEventKind::CkWithdrawReimbursed {
                                            asset_id,
                                            heir_id,
                                        })
                                    } else {
                                        rec.bridge_status = Some(BridgeStatus::Failed(msg.clone()));
                                        rec.bridge_error = Some(info);
                                        None
                                    }
                                } else {
                                    None
                                }
                            } else if !completed {
                                rec.bridge_status = Some(BridgeStatus::InProgress);
                                None
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    };
                    if let Some(ev) = event_opt {
                        push_audit(u, ev);
                    }
                }
            });
        }
    }

    // Final update block: collect audits and retry scheduling WITHOUT pushing while a field is borrowed.
    let mut audits: Vec<AuditEventKind> = Vec::new();
    let mut should_enqueue_retry = false;

    USERS.with(|users| {
        let mut users = users.borrow_mut();
        if let Some(u) = users.get_mut(&caller) {
            // Work inside a block to end all field-borrows before we emit audits.
            let (audit_events, enqueue_retry_needed) = {
                let mut local_audits: Vec<AuditEventKind> = Vec::new();
                let mut local_retry = false;

                if let Some(rec) = u
                    .ck_withdraws
                    .iter_mut()
                    .find(|r| r.asset_id == asset_id && r.heir_id == heir_id)
                {
                    // Update bridge_txs classification state (mut borrow limited to this scope)
                    if let Some(txinfo) = u
                        .bridge_txs
                        .iter_mut()
                        .find(|t| t.asset_id == asset_id && t.heir_id == heir_id)
                    {
                        if not_found_detected && !completed {
                            let miss = txinfo.consecutive_misses.unwrap_or(0).saturating_add(1);
                            txinfo.consecutive_misses = Some(miss);
                            if miss >= 5 && txinfo.notfound_terminal != Some(true) {
                                txinfo.notfound_terminal = Some(true);
                                local_audits.push(AuditEventKind::BridgePollNotFoundTerminal {
                                    asset_id,
                                    heir_id,
                                });
                                // mark record failed terminal if not already completed
                                rec.bridge_status =
                                    Some(BridgeStatus::Failed("not_found_terminal".into()));
                            }
                        } else if completed {
                            txinfo.consecutive_misses = Some(0);
                        }
                    }

                    if completed {
                        if rec.completed_at.is_none() {
                            rec.completed_at = Some(now_secs());
                            rec.bridge_status = Some(BridgeStatus::Completed);
                            local_audits
                                .push(AuditEventKind::CkWithdrawCompleted { asset_id, heir_id });
                        }
                    } else if rec
                        .bridge_status
                        .as_ref()
                        .map(|s| {
                            !matches!(
                                s,
                                BridgeStatus::Failed(_)
                                    | BridgeStatus::Reimbursed
                                    | BridgeStatus::Completed
                            )
                        })
                        .unwrap_or(true)
                    {
                        // Only enqueue retry if not terminal
                        let is_terminal_notfound = u
                            .bridge_txs
                            .iter()
                            .find(|t| t.asset_id == asset_id && t.heir_id == heir_id)
                            .and_then(|t| t.notfound_terminal)
                            .unwrap_or(false);
                        if !is_terminal_notfound {
                            local_retry = true;
                        }
                    }
                }
                (local_audits, local_retry)
            };

            // After inner scope ends, no field is mut-borrowed; now we can emit audits
            for ev in audit_events {
                push_audit(u, ev);
            }
            if enqueue_retry_needed {
                should_enqueue_retry = true;
            }
        }
    });

    // Perform retry scheduling outside the USERS.with borrow
    if should_enqueue_retry {
        crate::api::retry::enqueue_retry(
            crate::api::retry::RetryKind::BridgePoll { asset_id, heir_id },
            5,
        );
    }

    Ok(())
}
