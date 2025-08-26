//! Work item snapshotting & transfer processing helpers (internal to executor).

use crate::audit::push_audit;
use crate::models::*;
use crate::storage::USERS;
use crate::time::now_secs;
use candid::Principal;

// Map internal string error markers to structured CivError variants (serialized via string for now).
fn map_error_code(code: &str) -> String {
    match code {
        "missing_approval" => "ERR_MISSING_APPROVAL".into(),
        "allowance_not_found_on_chain" => "ERR_ALLOWANCE_CHAIN_MISSING".into(),
        "invalid_owner_principal" => "ERR_INVALID_OWNER_PRINCIPAL".into(),
        "missing_destination_principal" => "ERR_MISSING_DESTINATION".into(),
        c if c.starts_with("dip721_err:") => format!("NFT_DIP721:{}", &c[11..]),
        c if c.starts_with("ext_err:") => format!("NFT_EXT:{}", &c[8..]),
        c if c.starts_with("nft_standard_unsupported:") => format!("NFT_UNSUPPORTED:{}", &c[24..]),
        other => other.to_string(),
    }
}

// -------- Internal ICRC helpers (u128 everywhere) --------
#[allow(dead_code)]
pub async fn icrc1_transfer(
    canister: Principal,
    to_principal: Principal,
    to_subaccount: Option<Vec<u8>>,
    amount: u128,
) -> (Option<u128>, Option<String>) {
    // NOTE: Phase 1 groundwork: this helper currently only supports destination subaccount.
    // Future enhancement: introduce source subaccount parameter for custody-held balances
    // once custody reconciliation determines available funds in derived subaccounts.
    #[derive(candid::CandidType, serde::Deserialize)]
    struct Icrc1Args {
        to: Icrc1Account,
        amount: u128,
    }
    #[derive(candid::CandidType, serde::Deserialize)]
    struct Icrc1Account {
        owner: Principal,
        subaccount: Option<Vec<u8>>,
    }
    let args = Icrc1Args {
        to: Icrc1Account {
            owner: to_principal,
            subaccount: to_subaccount,
        },
        amount,
    };
    use ic_cdk::call::Call;
    let call = Call::unbounded_wait(canister, "icrc1_transfer").with_arg(args);
    let res = call
        .await
        .map_err(|e| format!("call failed: {:?}", e))
        .and_then(|resp| {
            resp.candid_tuple::<(Result<u128, String>,)>()
                .map_err(|e| format!("decode err: {:?}", e))
        });
    match res {
        Ok((Ok(idx),)) => (Some(idx), None),
        Ok((Err(e),)) => (None, Some(e)),
        Err(e) => (None, Some(e)),
    }
}

// Source-subaccount (custody) variant: performs transfer from a specified subaccount owned by this canister to external principal (no destination subaccount currently).
pub async fn icrc1_transfer_from_sub(
    canister: Principal,
    from_subaccount: Vec<u8>,
    to_principal: Principal,
    to_subaccount: Option<Vec<u8>>,
    amount: u128,
) -> (Option<u128>, Option<String>) {
    #[derive(candid::CandidType, serde::Deserialize)]
    struct Icrc1Account {
        owner: Principal,
        subaccount: Option<Vec<u8>>,
    }
    #[derive(candid::CandidType, serde::Deserialize)]
    struct Icrc1TransferArgs {
        from_subaccount: Option<Vec<u8>>,
        to: Icrc1Account,
        amount: u128,
    }
    use ic_cdk::call::Call;
    let args = Icrc1TransferArgs {
        from_subaccount: Some(from_subaccount),
        to: Icrc1Account {
            owner: to_principal,
            subaccount: to_subaccount,
        },
        amount,
    };
    let call = Call::unbounded_wait(canister, "icrc1_transfer").with_arg(args);
    let res = call
        .await
        .map_err(|e| format!("call failed: {:?}", e))
        .and_then(|resp| {
            resp.candid_tuple::<(Result<u128, String>,)>()
                .map_err(|e| format!("decode err: {:?}", e))
        });
    match res {
        Ok((Ok(idx),)) => (Some(idx), None),
        Ok((Err(e),)) => (None, Some(e)),
        Err(e) => (None, Some(e)),
    }
}

pub async fn icrc1_balance_of(
    canister: Principal,
    owner: Principal,
    sub: Option<Vec<u8>>,
) -> Result<u128, String> {
    #[derive(candid::CandidType, serde::Deserialize)]
    struct Icrc1Account {
        owner: Principal,
        subaccount: Option<Vec<u8>>,
    }
    #[derive(candid::CandidType, serde::Deserialize)]
    struct Args {
        account: Icrc1Account,
    }
    use ic_cdk::call::Call;
    let args = Args {
        account: Icrc1Account {
            owner,
            subaccount: sub,
        },
    };
    let call = Call::unbounded_wait(canister, "icrc1_balance_of").with_arg(args);
    let res = call
        .await
        .map_err(|e| format!("call failed: {:?}", e))
        .and_then(|resp| {
            resp.candid_tuple::<(Result<u128, String>,)>()
                .map_err(|e| format!("decode err: {:?}", e))
        });
    match res {
        Ok((Ok(v),)) => Ok(v),
        Ok((Err(e),)) => Err(e),
        Err(e) => Err(e),
    }
}

#[allow(dead_code)]
async fn icrc1_transfer_from_subaccount(
    canister: Principal,
    _from_sub: Vec<u8>,
    to_principal: Principal,
    amount: u128,
) -> (Option<u128>, Option<String>) {
    icrc1_transfer(canister, to_principal, None, amount).await
}

#[allow(dead_code)]
pub(crate) async fn icrc2_transfer_from(
    canister: Principal,
    from_principal: Principal,
    from_sub: Option<Vec<u8>>,
    to_principal: Principal,
    to_sub: Option<Vec<u8>>,
    amount: u128,
) -> (Option<u128>, Option<String>) {
    #[derive(candid::CandidType, serde::Deserialize)]
    struct Icrc2Account {
        owner: Principal,
        subaccount: Option<Vec<u8>>,
    }
    #[derive(candid::CandidType, serde::Deserialize)]
    struct Icrc2Args {
        from: Icrc2Account,
        to: Icrc2Account,
        amount: u128,
    }
    let args = Icrc2Args {
        from: Icrc2Account {
            owner: from_principal,
            subaccount: from_sub,
        },
        to: Icrc2Account {
            owner: to_principal,
            subaccount: to_sub,
        },
        amount,
    };
    use ic_cdk::call::Call;
    let call = Call::unbounded_wait(canister, "icrc2_transfer_from").with_arg(args);
    let res = call
        .await
        .map_err(|e| format!("call failed: {:?}", e))
        .and_then(|resp| {
            resp.candid_tuple::<(Result<u128, String>,)>()
                .map_err(|e| format!("decode err: {:?}", e))
        });
    match res {
        Ok((Ok(idx),)) => (Some(idx), None),
        Ok((Err(e),)) => (None, Some(e)),
        Err(e) => (None, Some(e)),
    }
}

#[allow(dead_code)]
async fn icrc2_allowance(
    canister: Principal,
    owner: Principal,
    spender: Principal,
) -> Result<u128, String> {
    #[derive(candid::CandidType, serde::Deserialize)]
    struct AllowanceArgs {
        owner: Icrc2Account,
        spender: Icrc2Account,
    }
    #[derive(candid::CandidType, serde::Deserialize)]
    struct Icrc2Account {
        owner: Principal,
        subaccount: Option<Vec<u8>>,
    }
    use ic_cdk::call::Call;
    let args = AllowanceArgs {
        owner: Icrc2Account {
            owner,
            subaccount: None,
        },
        spender: Icrc2Account {
            owner: spender,
            subaccount: None,
        },
    };
    let call = Call::unbounded_wait(canister, "icrc2_allowance").with_arg(args);
    let res = call
        .await
        .map_err(|e| format!("call failed: {:?}", e))
        .and_then(|resp| {
            resp.candid_tuple::<(Result<u128, String>,)>()
                .map_err(|e| format!("decode err: {:?}", e))
        });
    match res {
        Ok((Ok(v),)) => Ok(v),
        Ok((Err(e),)) => Err(e),
        Err(e) => Err(e),
    }
}

// -------- Work item assembly --------
#[allow(clippy::large_enum_variant)]
#[derive(Clone)]
pub(crate) struct WorkItem {
    pub(crate) asset_id: u64,
    pub(crate) token_canister: Option<String>,
    pub(crate) token_id: Option<u64>,
    pub(crate) holding_mode: Option<HoldingMode>,
    pub(crate) heir_id: u64,
    pub(crate) payout_pref: PayoutPreference,
}

pub(crate) fn snapshot_workitems(caller: &str) -> Result<Vec<WorkItem>, CivError> {
    USERS.with(|users| {
        let users = users.borrow();
        if let Some(u) = users.get(caller) {
            if u.phase != EstatePhase::Locked {
                return Ok(Vec::new());
            }
            // Enforcement: block work snapshot if any fungible or chain-wrapped asset lacks decimals
            let mut missing_decimals = false;
            for a in u.assets.iter() {
                let kind = crate::models::infer_asset_kind(&a.asset_type);
                if matches!(kind, AssetKind::Fungible | AssetKind::ChainWrapped)
                    && a.decimals == 0
                {
                    missing_decimals = true;
                    break;
                }
            }
            if missing_decimals {
                return Err(CivError::Other("decimals_missing_gate".into()));
            }
            // Escrow sufficiency preflight (refined): for each escrow-mode fungible asset ensure remaining escrow >= required payout sum.
            // Required payout sum = sum( asset.value * pct / 100 ) across all distributions for that asset.
            // NOTE: asset.value is assumed already in smallest token units (or consistent with escrow amount units).
            for a in u
                .assets
                .iter()
                .filter(|a| matches!(a.holding_mode, Some(HoldingMode::Escrow)))
            {
                let escrow_rem = u
                    .escrow
                    .iter()
                    .find(|e| e.asset_id == a.id)
                    .and_then(|e| e.amount)
                    .unwrap_or(0u128);
                // accumulate required distribution total in u128
                let mut required: u128 = 0;
                for d in u.distributions_v2.iter().filter(|d| d.asset_id == a.id) {
                    let pct = d.percentage as u128;
                    let part = (a.value as u128)
                        .saturating_mul(pct)
                        .checked_div(100)
                        .unwrap_or(0);
                    required = required.saturating_add(part);
                }
                if escrow_rem < required {
                    return Err(CivError::Other(format!(
                        "escrow_insufficient_required:{}:{}:{}",
                        a.id, escrow_rem, required
                    )));
                }
            }
            // Approvals sufficiency preflight: ensure that for every fungible distribution NOT using escrow we have a local approval record.
            // (Deep chain allowance validation deferred to per-transfer path; here we only check presence.)
            for d in u.distributions_v2.iter() {
                let asset = u.assets.iter().find(|a| a.id == d.asset_id);
                if let Some(a) = asset {
                    if !matches!(a.holding_mode, Some(HoldingMode::Escrow)) {
                        let has_appr = u.approvals.iter().any(|ap| ap.asset_id == a.id);
                        if !has_appr {
                            return Err(CivError::Other(format!(
                                "approval_missing_asset:{}",
                                a.id
                            )));
                        }
                    }
                }
            }
            // Allowance projection: ensure each non-escrow fungible asset has aggregate local allowance (if tracked) >= total distribution requirement.
            for a in u.assets.iter().filter(|a| {
                let kind = crate::models::infer_asset_kind(&a.asset_type);
                matches!(kind, AssetKind::Fungible | AssetKind::ChainWrapped)
                    && !matches!(a.holding_mode, Some(HoldingMode::Escrow))
            }) {
                let mut required: u128 = 0;
                for d in u.distributions_v2.iter().filter(|d| d.asset_id == a.id) {
                    let pct = d.percentage as u128;
                    let part = (a.value as u128)
                        .saturating_mul(pct)
                        .checked_div(100)
                        .unwrap_or(0);
                    required = required.saturating_add(part);
                }
                if let Some(appr) = u.approvals.iter().find(|ap| ap.asset_id == a.id) {
                    if let Some(local_allow) = appr.allowance {
                        if local_allow < required {
                            return Err(CivError::AllowanceInsufficient {
                                needed: required,
                                found: local_allow,
                            });
                        }
                    }
                }
            }
            let items = u
                .distributions_v2
                .iter()
                .filter_map(|d| {
                    let asset = u.assets.iter().find(|a| a.id == d.asset_id)?;
                    let effective_pref = u
                        .payout_overrides
                        .iter()
                        .find(|o| o.asset_id == d.asset_id && o.heir_id == d.heir_id)
                        .map(|o| o.payout_preference.clone())
                        .unwrap_or_else(|| d.payout_preference.clone());
                    Some(WorkItem {
                        asset_id: asset.id,
                        token_canister: asset.token_canister.clone(),
                        token_id: asset.token_id,
                        holding_mode: asset.holding_mode.clone(),
                        heir_id: d.heir_id,
                        payout_pref: effective_pref,
                    })
                })
                .collect();
            Ok(items)
        } else {
            Ok(Vec::new())
        }
    })
}

pub(crate) fn next_transfer_id(caller: &str) -> u64 {
    USERS.with(|users| {
        let users = users.borrow();
        users
            .get(caller)
            .map(|u| u.transfers.iter().map(|t| t.id).max().unwrap_or(0) + 1)
            .unwrap_or(1)
    })
}

pub(crate) async fn process_workitem(
    caller: &str,
    w: &WorkItem,
    ts: u64,
    id: u64,
) -> (TransferRecord, Option<CkWithdrawRecord>) {
    let kind = if w.token_id.is_some() {
        TransferKind::Nft
    } else {
        TransferKind::Fungible
    };
    let (amount, tx_index, note, error) =
        match w.holding_mode.clone().unwrap_or(HoldingMode::Approval) {
            HoldingMode::Escrow => {
                let (amt, _tx_unused, n, err) = handle_escrow(caller, w, kind.clone());
                (amt, _tx_unused, n, err)
            }
            HoldingMode::Approval => match kind {
                TransferKind::Fungible => handle_fungible_transfer(caller, w).await,
                TransferKind::Nft => handle_nft_transfer(caller, w).await,
                TransferKind::Document => (None, None, Some("document_unlocked".into()), None),
            },
        };
    let ck_stage = if matches!(w.payout_pref, PayoutPreference::CkWithdraw) {
        amount.map(|amt| CkWithdrawRecord {
            asset_id: w.asset_id,
            heir_id: w.heir_id,
            amount: amt,
            staged_at: ts,
            requested_at: None,
            completed_at: None,
            tx_index: None,
            bridge_status: Some(BridgeStatus::Staged),
            bridge_tx_id: None,
            bridge_error: None,
            tx_hash: None,
            effective_fee: None,
            quoted_fee: None,
        })
    } else {
        None
    };
    // Map raw error strings to normalized codes (kept as String in TransferRecord.error for now).
    let normalized_error = error.as_ref().map(|e| map_error_code(e));
    let (error_kind, error) = if let Some(ref code) = normalized_error {
        let (k, _d) = crate::models::payout::TransferErrorKind::from_legacy(code);
        (Some(k), normalized_error)
    } else {
        (None, None)
    };
    let record = TransferRecord {
        id,
        timestamp: ts,
        asset_id: Some(w.asset_id),
        heir_id: Some(w.heir_id),
        kind,
        amount,
        payout_preference: Some(w.payout_pref.clone()),
        note,
        tx_index,
        error,
        error_kind,
    };
    (record, ck_stage)
}

fn handle_escrow(
    caller: &str,
    w: &WorkItem,
    kind: TransferKind,
) -> (Option<u128>, Option<u128>, Option<String>, Option<String>) {
    match kind {
        TransferKind::Fungible => {
            // Gather snapshot: escrow amount, percentage, token canister & decimals
            let snapshot = USERS.with(|users| {
                let users = users.borrow();
                if let Some(u) = users.get(caller) {
                    let asset = u.assets.iter().find(|a| a.id == w.asset_id);
                    let esc_amount = u
                        .escrow
                        .iter()
                        .find(|e| e.asset_id == w.asset_id)
                        .and_then(|e| e.amount);
                    let share = u
                        .distributions_v2
                        .iter()
                        .find(|d| d.asset_id == w.asset_id && d.heir_id == w.heir_id);
                    let pct = share.map(|s| s.percentage as u128).unwrap_or(0);
                    if let Some(a) = asset {
                        (esc_amount, pct, a.token_canister.clone(), a.decimals)
                    } else {
                        (esc_amount, pct, None, 0u8)
                    }
                } else {
                    (None, 0, None, 0u8)
                }
            });
            let (esc_amount_opt, pct, token_can_txt, _decimals_opt) = snapshot;
            let escrow_total = esc_amount_opt.unwrap_or(0);
            if escrow_total == 0 || pct == 0 {
                return (Some(0), None, Some("escrow_release_zero".into()), None);
            }
            let heir_amount_base = escrow_total
                .saturating_mul(pct)
                .checked_div(100)
                .unwrap_or(0);
            // Adjust for decimals if we stored human value (assume escrow amounts already in smallest units, so no scaling here)
            let heir_amount = heir_amount_base; // already token units
            let mut error: Option<String> = None;
            if let Some(can_txt) = token_can_txt {
                if let Ok(can_principal) = Principal::from_text(&can_txt) {
                    // For on-chain escrow subaccount transfer, derive subaccount & call icrc1_transfer from that subaccount to heir principal
                    let (heir_principal_opt,) = USERS.with(|users| {
                        let users = users.borrow();
                        if let Some(u) = users.get(caller) {
                            (u.heirs_v2
                                .iter()
                                .find(|h| h.id == w.heir_id)
                                .and_then(|h| h.principal.clone()),)
                        } else {
                            (None,)
                        }
                    });
                    if let Some(hp_txt) = heir_principal_opt {
                        if let Ok(_hp) = Principal::from_text(&hp_txt) {
                            let _sub = crate::crypto::derive_escrow_subaccount(caller, w.asset_id);
                            // Enqueue EscrowRelease retry (async processing will perform transfer)
                            crate::api::retry::enqueue_retry(
                                crate::api::retry::RetryKind::EscrowRelease {
                                    asset_id: w.asset_id,
                                    heir_id: w.heir_id,
                                },
                                5,
                            );
                            return (
                                Some(heir_amount),
                                None,
                                Some("escrow_release_enqueued".into()),
                                None,
                            );
                        } else {
                            error = Some("invalid_heir_principal".into());
                        }
                    } else {
                        error = Some("missing_heir_principal".into());
                    }
                }
            }
            (
                Some(heir_amount),
                None,
                Some("escrow_release".into()),
                error,
            )
        }
        TransferKind::Nft => (None, None, Some("escrow_release_nft".into()), None),
        TransferKind::Document => (None, None, Some("document_unlocked".into()), None),
    }
}

pub(crate) fn stage_ck_withdraw(caller: &str, record: CkWithdrawRecord) {
    USERS.with(|users| {
        let mut users = users.borrow_mut();
        if let Some(u) = users.get_mut(caller) {
            if u.phase == EstatePhase::Locked {
                let asset_id = record.asset_id;
                let heir_id = record.heir_id;
                let amount = record.amount;
                u.ck_withdraws.push(record);
                push_audit(
                    u,
                    AuditEventKind::CkWithdrawStaged {
                        asset_id,
                        heir_id,
                        amount,
                    },
                );
            }
        }
    });
}

async fn handle_fungible_transfer(
    caller: &str,
    w: &WorkItem,
) -> (Option<u128>, Option<u128>, Option<String>, Option<String>) {
    let mut amount: Option<u128> = None;
    let mut tx_index: Option<u128> = None;
    let mut note: Option<String> = None;
    let mut error: Option<String> = None;
    if let Some(can_txt) = &w.token_canister {
        if let Ok(principal) = Principal::from_text(can_txt) {
            let (asset_value, pct, decimals) = USERS.with(|users| {
                let users = users.borrow();
                if let Some(u) = users.get(caller) {
                    let asset = u.assets.iter().find(|a| a.id == w.asset_id);
                    let share = u
                        .distributions_v2
                        .iter()
                        .find(|d| d.asset_id == w.asset_id && d.heir_id == w.heir_id);
                    if let (Some(a), Some(s)) = (asset, share) {
                        (a.value as u128, s.percentage as u128, a.decimals)
                    } else {
                        (0u128, 0u128, 0u8)
                    }
                } else {
                    (0u128, 0u128, 0u8)
                }
            });
            let raw_amount = asset_value
                .saturating_mul(pct)
                .checked_div(100)
                .unwrap_or(0);
            let transfer_amount = if decimals != 0 {
                raw_amount.saturating_mul(10u128.saturating_pow(decimals as u32))
            } else {
                raw_amount
            };
            amount = Some(transfer_amount);
            if transfer_amount > 0 {
                // Special custody funding path: move funds from user principal into canister custody subaccount immediately.
                if matches!(w.payout_pref, PayoutPreference::ToCustody) {
                    // We require an approval (ICRC2) so we can pull funds from the user's account.
                    let (has_appr, allowance_opt) = USERS.with(|users| {
                        let users = users.borrow();
                        if let Some(u) = users.get(caller) {
                            if let Some(appr) =
                                u.approvals.iter().find(|a| a.asset_id == w.asset_id)
                            {
                                (true, appr.allowance)
                            } else {
                                (false, None)
                            }
                        } else {
                            (false, None)
                        }
                    });
                    if !has_appr {
                        error = Some("missing_approval".into());
                    } else if let Some(local_allowance) = allowance_opt {
                        // Ensure on-chain allowance >= amount; refresh if possible.
                        let mut effective_allowance = local_allowance;
                        if let Ok(owner_p) = Principal::from_text(caller) {
                            if let Ok(chain_allow) =
                                icrc2_allowance(principal, owner_p, ic_cdk::api::canister_self())
                                    .await
                            {
                                effective_allowance = chain_allow;
                            }
                        }
                        if effective_allowance < transfer_amount {
                            error = Some("allowance_not_found_on_chain".into());
                        } else {
                            // Ensure custody subaccount exists & obtain it.
                            let custody_sub = USERS.with(|users| {
                                let mut users = users.borrow_mut();
                                if let Some(u) = users.get_mut(caller) {
                                    if let Some(c) =
                                        u.custody.iter().find(|c| c.heir_id == w.heir_id)
                                    {
                                        c.subaccount.clone()
                                    } else {
                                        let sub = crate::crypto::derive_custody_subaccount(
                                            caller, w.heir_id,
                                        );
                                        let v = sub.to_vec();
                                        u.custody.push(CustodyRecord {
                                            heir_id: w.heir_id,
                                            subaccount: v.clone(),
                                        });
                                        v
                                    }
                                } else {
                                    Vec::new()
                                }
                            });
                            // Execute icrc2_transfer_from from user -> canister custody subaccount.
                            match Principal::from_text(caller) {
                                Ok(owner_p) => {
                                    let (tx, err2) = icrc2_transfer_from(
                                        principal,
                                        owner_p,
                                        None,
                                        ic_cdk::api::canister_self(),
                                        Some(custody_sub),
                                        transfer_amount,
                                    )
                                    .await;
                                    tx_index = tx;
                                    error = err2;
                                    note = Some(
                                        if error.is_none() {
                                            "custody_funded"
                                        } else {
                                            "custody_funding_failed"
                                        }
                                        .into(),
                                    );
                                    // On success, decrement local allowance and record fungible custody record on-chain snapshot.
                                    if error.is_none() {
                                        USERS.with(|users| {
                                            let mut users = users.borrow_mut();
                                            if let Some(u) = users.get_mut(caller) {
                                                if let Some(appr) = u
                                                    .approvals
                                                    .iter_mut()
                                                    .find(|a| a.asset_id == w.asset_id)
                                                {
                                                    if let Some(alw) = &mut appr.allowance {
                                                        *alw = alw.saturating_sub(transfer_amount);
                                                    }
                                                }
                                                // Add a fungible custody record (if not already exists accumulate amount)
                                                let now = now_secs();
                                                if let Some(existing) =
                                                    u.fungible_custody.iter_mut().find(|c| {
                                                        c.asset_id == w.asset_id
                                                            && c.heir_id == w.heir_id
                                                            && c.released_at.is_none()
                                                    })
                                                {
                                                    existing.amount = existing
                                                        .amount
                                                        .saturating_add(transfer_amount);
                                                } else {
                                                    u.fungible_custody.push(
                                                        FungibleCustodyRecord {
                                                            asset_id: w.asset_id,
                                                            heir_id: w.heir_id,
                                                            amount: transfer_amount,
                                                            staged_at: now,
                                                            released_at: None,
                                                            attempts: 0,
                                                            last_error: None,
                                                            releasing: false,
                                                            next_attempt_after: None,
                                                        },
                                                    );
                                                    push_audit(
                                                        u,
                                                        AuditEventKind::FungibleCustodyStaged {
                                                            asset_id: w.asset_id,
                                                            heir_id: w.heir_id,
                                                            amount: transfer_amount,
                                                        },
                                                    );
                                                }
                                            }
                                        });
                                    }
                                }
                                Err(_) => {
                                    error = Some("invalid_owner_principal".into());
                                }
                            }
                        }
                    } else {
                        error = Some("allowance_not_found_on_chain".into());
                    }
                    return (amount, tx_index, note, error); // custody funding path complete
                }
                let (to_principal, to_sub) = USERS.with(|users| {
                    let users = users.borrow();
                    let u = users.get(caller).unwrap();
                    let heir = u.heirs_v2.iter().find(|h| h.id == w.heir_id);
                    let to_p = heir.and_then(|h| h.principal.clone());
                    if to_p.is_none() {
                        (None, None)
                    } else {
                        let ptxt = to_p.unwrap();
                        let pr = Principal::from_text(ptxt).ok();
                        let sub = if matches!(w.payout_pref, PayoutPreference::ToCustody) {
                            u.custody
                                .iter()
                                .find(|c| c.heir_id == w.heir_id)
                                .map(|c| c.subaccount.clone())
                        } else {
                            None
                        };
                        (pr, sub)
                    }
                });
                if matches!(w.payout_pref, PayoutPreference::ToCustody) && to_principal.is_none() {
                    if let Some(amt) = amount {
                        USERS.with(|users| {
                            let mut users = users.borrow_mut();
                            if let Some(u) = users.get_mut(caller) {
                                let has_custody = u.custody.iter().any(|c| c.heir_id == w.heir_id);
                                if !has_custody {
                                    let sub =
                                        crate::crypto::derive_custody_subaccount(caller, w.heir_id);
                                    u.custody.push(CustodyRecord {
                                        heir_id: w.heir_id,
                                        subaccount: sub.to_vec(),
                                    });
                                }
                                let exists = u
                                    .fungible_custody
                                    .iter()
                                    .any(|c| c.asset_id == w.asset_id && c.heir_id == w.heir_id);
                                if !exists {
                                    u.fungible_custody.push(FungibleCustodyRecord {
                                        asset_id: w.asset_id,
                                        heir_id: w.heir_id,
                                        amount: amt,
                                        staged_at: now_secs(),
                                        released_at: None,
                                        attempts: 0,
                                        last_error: None,
                                        releasing: false,
                                        next_attempt_after: None,
                                    });
                                    push_audit(
                                        u,
                                        AuditEventKind::FungibleCustodyStaged {
                                            asset_id: w.asset_id,
                                            heir_id: w.heir_id,
                                            amount: amt,
                                        },
                                    );
                                }
                            }
                        });
                        note = Some("fungible_custody_staged".into());
                        tx_index = None;
                        error = None;
                        return (amount, tx_index, note, error);
                    }
                }
                if let (Some(tp), sub) = (to_principal, to_sub) {
                    let (has_appr, allowance_opt) = USERS.with(|users| {
                        let users = users.borrow();
                        if let Some(u) = users.get(caller) {
                            if let Some(appr) =
                                u.approvals.iter().find(|a| a.asset_id == w.asset_id)
                            {
                                (true, appr.allowance)
                            } else {
                                (false, None)
                            }
                        } else {
                            (false, None)
                        }
                    });
                    if !has_appr {
                        error = Some("missing_approval".into()); // CivError::MissingApproval
                    } else if let Some(local_allowance) = allowance_opt {
                        let mut effective_allowance = local_allowance;
                        if let Ok(owner_p) = Principal::from_text(caller) {
                            if let Ok(chain_allow) =
                                icrc2_allowance(principal, owner_p, ic_cdk::api::canister_self())
                                    .await
                            {
                                effective_allowance = chain_allow;
                            }
                        }
                        if effective_allowance >= transfer_amount {
                            match Principal::from_text(caller) {
                                Ok(owner_p) => {
                                    let (tx, err) = icrc2_transfer_from(
                                        principal,
                                        owner_p,
                                        None,
                                        tp,
                                        sub,
                                        transfer_amount,
                                    )
                                    .await;
                                    tx_index = tx;
                                    error = err;
                                    note = Some(
                                        if error.is_none() {
                                            "icrc2_transfer_from"
                                        } else {
                                            "icrc2_transfer_from_failed"
                                        }
                                        .into(),
                                    );
                                }
                                Err(_) => {
                                    error = Some("invalid_owner_principal".into());
                                    // CivError::InvalidOwnerPrincipal
                                }
                            }
                        } else {
                            let (tx, err) =
                                icrc1_transfer(principal, tp, sub, transfer_amount).await;
                            tx_index = tx;
                            error = err;
                            note = Some(
                                if error.is_none() {
                                    "icrc1_transfer"
                                } else {
                                    "icrc1_transfer_failed"
                                }
                                .into(),
                            );
                        }
                    } else {
                        // approval present but no local allowance value stored
                        error = Some("allowance_not_found_on_chain".into()); // CivError::AllowanceNotFoundOnChain
                    }
                } else if matches!(w.payout_pref, PayoutPreference::ToCustody) {
                    note = Some("fungible_custody_staged".into()); // unreachable with early custody funding path, retained for safety
                } else {
                    error = Some("missing_destination_principal".into());
                }
            } else {
                note = Some("zero_amount_skip".into());
            }
        } else {
            note = Some("invalid token canister principal".into());
        }
    } else {
        note = Some("no_token_metadata".into());
    }
    (amount, tx_index, note, error)
}

async fn handle_nft_transfer(
    caller: &str,
    w: &WorkItem,
) -> (Option<u128>, Option<u128>, Option<String>, Option<String>) {
    let mut error: Option<String> = None;
    let mut note: Option<String> = None;
    if let Some(token_id) = w.token_id {
        let to_principal_opt = USERS.with(|users| {
            let users = users.borrow();
            users.get(caller).and_then(|u| {
                u.heirs_v2
                    .iter()
                    .find(|h| h.id == w.heir_id)
                    .and_then(|h| h.principal.clone())
            })
        });
        if let Some(to_p_txt) = to_principal_opt {
            if let Ok(to_p) = Principal::from_text(&to_p_txt) {
                let has_appr = USERS.with(|users| {
                    let users = users.borrow();
                    users
                        .get(caller)
                        .map(|u| {
                            u.approvals
                                .iter()
                                .any(|a| a.asset_id == w.asset_id && a.token_id == Some(token_id))
                        })
                        .unwrap_or(false)
                });
                if !has_appr {
                    error = Some("missing_approval".into()); // CivError::MissingApproval
                } else if let Some(can_txt) = &w.token_canister {
                    if let Ok(principal) = Principal::from_text(can_txt) {
                        // Determine NFT standard from asset metadata
                        let nft_standard_opt = USERS.with(|users| {
                            let users = users.borrow();
                            users.get(caller).and_then(|u| {
                                u.assets
                                    .iter()
                                    .find(|a| a.id == w.asset_id)
                                    .and_then(|a| a.nft_standard.clone())
                            })
                        });
                        match nft_standard_opt.clone() {
                            Some(crate::models::NftStandard::Other(s)) => {
                                error = Some(format!("nft_standard_unsupported:{}", s));
                                note = Some("nft_transfer_unsupported".into());
                            }
                            other => {
                                let adapter = crate::api::nft_adapter::adapter_for(other);
                                let outcome = adapter.transfer(principal, to_p, token_id).await;
                                match outcome {
                                    crate::api::nft_adapter::NftTransferOutcome::Success {
                                        note: n,
                                    } => {
                                        note = Some(n);
                                    }
                                    crate::api::nft_adapter::NftTransferOutcome::Failure {
                                        code,
                                        note: n,
                                    } => {
                                        error = Some(code);
                                        note = Some(n);
                                    }
                                }
                            }
                        }
                    } else {
                        error = Some("invalid_token_canister_principal".into());
                    }
                } else {
                    note = Some("nft_no_token_canister".into());
                }
            } else {
                error = Some("invalid_heir_principal".into());
            }
        } else {
            USERS.with(|users| {
                let mut users = users.borrow_mut();
                if let Some(u) = users.get_mut(caller) {
                    if let Some(token_id) = w.token_id {
                        let exists = u.nft_custody.iter().any(|c| {
                            c.asset_id == w.asset_id
                                && c.heir_id == w.heir_id
                                && c.token_id == token_id
                        });
                        if !exists {
                            let now = now_secs();
                            let asset_id = w.asset_id;
                            let heir_id = w.heir_id;
                            u.nft_custody.push(NftCustodyRecord {
                                asset_id,
                                heir_id,
                                token_id,
                                staged_at: now,
                                released_at: None,
                                attempts: 0,
                                last_error: None,
                                releasing: false,
                                next_attempt_after: None,
                            });
                            push_audit(
                                u,
                                AuditEventKind::NftCustodyStaged {
                                    asset_id,
                                    heir_id,
                                    token_id,
                                },
                            );
                        }
                        note = Some("nft_custody_staged".into());
                    } else {
                        note = Some("nft_custody_missing_token".into());
                    }
                }
            });
        }
    } else {
        note = Some("nft_no_token_id".into()); // Not an error; indicates asset metadata incomplete
    }
    (None, None, note, error)
}

pub(crate) fn finalize_execution(
    caller: &str,
    records: Vec<TransferRecord>,
    ts: u64,
    summary: ExecutionSummary,
) {
    USERS.with(|users| {
        let mut users = users.borrow_mut();
        if let Some(u) = users.get_mut(caller) {
            if u.phase != EstatePhase::Locked {
                return;
            }
            u.transfers.extend(records);
            let from = u.phase.clone();
            u.phase = EstatePhase::Executed;
            u.executed_at = Some(ts);
            u.last_execution_summary = Some(summary);
            // Clear execution nonce now that execution completed
            u.execution_nonce = None;
            push_audit(
                u,
                AuditEventKind::PhaseChanged {
                    from,
                    to: u.phase.clone(),
                },
            );
            push_audit(u, AuditEventKind::TriggerExecuted);
        }
    });
}
