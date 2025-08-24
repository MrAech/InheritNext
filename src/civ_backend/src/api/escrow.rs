// Escrow & Approval domain logic extracted from legacy api.rs
use crate::api::common::{assert_mutable, user_id};
use crate::audit::push_audit;
use crate::models::*;
use crate::storage::USERS;
use crate::time::now_secs;
use candid::Principal;
use ic_cdk::call::Call;

#[derive(Clone, candid::CandidType, serde::Deserialize)]
pub struct EscrowDepositInput {
    pub asset_id: u64,
    pub amount: Option<u128>,
    pub token_id: Option<u64>,
}

pub async fn deposit_escrow(input: EscrowDepositInput) -> Result<(), CivError> {
    let caller = user_id();
    // Phase 1: snapshot token canister & validate
    let (token_can_txt_opt, amount_opt) = USERS.with(|users| {
        let users = users.borrow();
        if let Some(u) = users.get(&caller) {
            let asset = u.assets.iter().find(|a| a.id == input.asset_id);
            let can = asset.and_then(|a| a.token_canister.clone());
            (can, input.amount)
        } else {
            (None, input.amount)
        }
    });
    // Phase 2: if fungible with amount and token canister, perform icrc2_transfer_from from user principal into escrow subaccount.
    if let (Some(can_txt), Some(amount)) = (token_can_txt_opt.clone(), amount_opt) {
        if amount > 0 {
            if let Ok(can_principal) = Principal::from_text(&can_txt) {
                // Build transfer_from args: from = user principal (owner), to = canister escrow subaccount
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
                let owner_p =
                    Principal::from_text(&caller).map_err(|_| CivError::InvalidOwnerPrincipal)?;
                let escrow_sub = crate::crypto::derive_escrow_subaccount(&caller, input.asset_id);
                let to_acct = Icrc2Account {
                    owner: ic_cdk::api::canister_self(),
                    subaccount: Some(escrow_sub.to_vec()),
                };
                let from_acct = Icrc2Account {
                    owner: owner_p,
                    subaccount: None,
                };
                let args = Icrc2Args {
                    from: from_acct,
                    to: to_acct,
                    amount,
                };
                let call =
                    Call::unbounded_wait(can_principal, "icrc2_transfer_from").with_arg(args);
                let res = call
                    .await
                    .map_err(|e| {
                        CivError::TransferCallFailed(format!("escrow_transfer_call_failed:{:?}", e))
                    })
                    .and_then(|reply| {
                        reply
                            .candid_tuple::<(Result<u128, String>,)>()
                            .map_err(|e| {
                                CivError::TransferCallFailed(format!(
                                    "escrow_transfer_decode_failed:{:?}",
                                    e
                                ))
                            })
                    });
                match res {
                    Ok((Ok(_idx),)) => { /* success */ }
                    Ok((Err(e),)) => {
                        return Err(CivError::Other(format!("escrow_transfer_failed:{}", e)));
                    }
                    Err(e) => {
                        return Err(e);
                    }
                }
            }
        }
    }
    // Phase 3: record logical escrow entry
    USERS.with(|users| {
        let mut users = users.borrow_mut();
        if let Some(u) = users.get_mut(&caller) {
            assert_mutable(u)?;
            if !u.assets.iter().any(|a| a.id == input.asset_id) {
                return Err(CivError::AssetNotFound);
            }
            u.escrow.retain(|e| e.asset_id != input.asset_id);
            u.escrow.push(EscrowRecord {
                asset_id: input.asset_id,
                amount: input.amount,
                token_id: input.token_id,
                deposited_at: now_secs(),
            });
            push_audit(
                u,
                AuditEventKind::EscrowDeposited {
                    asset_id: input.asset_id,
                    amount: input.amount,
                },
            );
            Ok(())
        } else {
            Err(CivError::UserNotFound)
        }
    })
}

pub fn list_escrow() -> Vec<EscrowRecord> {
    let caller = user_id();
    USERS.with(|users| {
        let users = users.borrow();
        users
            .get(&caller)
            .map(|u| u.escrow.clone())
            .unwrap_or_default()
    })
}

pub fn withdraw_escrow(asset_id: u64) -> Result<(), CivError> {
    let caller = user_id();
    USERS.with(|users| {
        let mut users = users.borrow_mut();
        if let Some(u) = users.get_mut(&caller) {
            assert_mutable(u)?;
            let before = u.escrow.len();
            u.escrow.retain(|e| e.asset_id != asset_id);
            if before == u.escrow.len() {
                return Err(CivError::AssetNotFound);
            }
            push_audit(
                u,
                AuditEventKind::EscrowWithdrawn {
                    asset_id,
                    amount: None,
                },
            );
            Ok(())
        } else {
            Err(CivError::UserNotFound)
        }
    })
}

// -------- On-chain escrow withdrawal (ICRC1) --------
#[derive(Clone, candid::CandidType, serde::Deserialize)]
pub struct EscrowWithdrawOnChainInput {
    pub asset_id: u64,
    pub amount: Option<u128>,
}

pub async fn withdraw_escrow_icrc1(input: EscrowWithdrawOnChainInput) -> Result<u128, CivError> {
    let caller = user_id();
    // Snapshot user phase, token canister, escrow record
    #[allow(clippy::type_complexity)]
    let (phase_opt, token_can_txt_opt, esc_amount_opt) = USERS.with(|users| {
        let users = users.borrow();
        if let Some(u) = users.get(&caller) {
            let asset = u.assets.iter().find(|a| a.id == input.asset_id);
            (
                Some(u.phase.clone()),
                asset.and_then(|a| a.token_canister.clone()),
                u.escrow
                    .iter()
                    .find(|e| e.asset_id == input.asset_id)
                    .and_then(|e| e.amount),
            )
        } else {
            (None, None, None)
        }
    });
    let phase = phase_opt.ok_or(CivError::UserNotFound)?;
    // Disallow withdrawal after estate locked/executed
    if matches!(phase, EstatePhase::Locked | EstatePhase::Executed) {
        return Err(CivError::Other("escrow_withdraw_locked".into()));
    }
    let token_can_txt = token_can_txt_opt.ok_or(CivError::AssetNotFound)?;
    let total = esc_amount_opt.unwrap_or(0);
    if total == 0 {
        return Err(CivError::Other("escrow_empty".into()));
    }
    let requested = input.amount.unwrap_or(total);
    let withdraw_amount = std::cmp::min(requested, total);
    if withdraw_amount == 0 {
        return Err(CivError::Other("escrow_zero_withdraw".into()));
    }
    let can_principal = Principal::from_text(&token_can_txt)
        .map_err(|_| CivError::Other("invalid_token_canister".into()))?;
    let owner_principal =
        Principal::from_text(&caller).map_err(|_| CivError::InvalidOwnerPrincipal)?;
    // Perform icrc1 transfer from escrow subaccount back to owner
    let sub = crate::crypto::derive_escrow_subaccount(&caller, input.asset_id);
    let (tx_idx_opt, err_opt) = crate::api::executor::work::icrc1_transfer_from_sub(
        can_principal,
        sub.to_vec(),
        owner_principal,
        None,
        withdraw_amount,
    )
    .await;
    if let Some(e) = err_opt {
        return Err(CivError::TransferCallFailed(format!(
            "escrow_withdraw_failed:{}",
            e
        )));
    }
    if tx_idx_opt.is_none() {
        return Err(CivError::TransferCallFailed(
            "escrow_withdraw_noindex".into(),
        ));
    }
    // Mutate escrow record
    USERS.with(|users| {
        let mut users = users.borrow_mut();
        if let Some(u) = users.get_mut(&caller) {
            if let Some(rec) = u.escrow.iter_mut().find(|e| e.asset_id == input.asset_id) {
                if let Some(prev) = rec.amount {
                    let rem = prev.saturating_sub(withdraw_amount);
                    rec.amount = Some(rem);
                }
            }
            u.escrow
                .retain(|e| !(e.asset_id == input.asset_id && e.amount == Some(0)));
            push_audit(
                u,
                AuditEventKind::EscrowWithdrawn {
                    asset_id: input.asset_id,
                    amount: Some(withdraw_amount),
                },
            );
        }
    });
    Ok(withdraw_amount)
}

#[derive(Clone, candid::CandidType, serde::Deserialize)]
pub struct ApprovalSetInput {
    pub asset_id: u64,
    pub allowance: Option<u128>,
    pub token_id: Option<u64>,
}

pub fn approval_set(input: ApprovalSetInput) -> Result<(), CivError> {
    let caller = user_id();
    USERS.with(|users| {
        let mut users = users.borrow_mut();
        if let Some(u) = users.get_mut(&caller) {
            assert_mutable(u)?;
            if !u.assets.iter().any(|a| a.id == input.asset_id) {
                return Err(CivError::AssetNotFound);
            }
            u.approvals.retain(|a| a.asset_id != input.asset_id);
            u.approvals.push(ApprovalRecord {
                asset_id: input.asset_id,
                allowance: input.allowance,
                token_id: input.token_id,
                granted_at: now_secs(),
            });
            push_audit(
                u,
                AuditEventKind::ApprovalSet {
                    asset_id: input.asset_id,
                },
            );
            Ok(())
        } else {
            Err(CivError::UserNotFound)
        }
    })
}

// -------- ICRC2 on-chain approval helper (real approve) --------
// Performs icrc2_approve call to ledger, records local ApprovalRecord with allowance returned (or requested allowance).
#[derive(Clone, candid::CandidType, serde::Deserialize)]
pub struct ApprovalSetOnChainInput {
    pub asset_id: u64,
    pub allowance: u128,
    pub expires_at: Option<u64>,
}

// Internal implementation; exported wrapper lives in lib.rs to maintain a single export surface.
pub async fn approval_set_icrc2(input: ApprovalSetOnChainInput) -> Result<(), CivError> {
    let caller = user_id();
    // Snapshot token canister
    let (token_can_txt,) = USERS.with(|users| {
        let users = users.borrow();
        if let Some(u) = users.get(&caller) {
            if let Some(a) = u.assets.iter().find(|a| a.id == input.asset_id) {
                (a.token_canister.clone(),)
            } else {
                (None,)
            }
        } else {
            (None,)
        }
    });
    let token_can_txt = token_can_txt.ok_or(CivError::AssetNotFound)?;
    let can_principal = Principal::from_text(&token_can_txt)
        .map_err(|_| CivError::Other("invalid_token_canister".into()))?;
    let owner_principal =
        Principal::from_text(&caller).map_err(|_| CivError::InvalidOwnerPrincipal)?;
    // Assemble icrc2_approve args (simplified subset).
    #[derive(candid::CandidType, serde::Deserialize)]
    struct Icrc2Account {
        owner: Principal,
        subaccount: Option<Vec<u8>>,
    }
    #[derive(candid::CandidType, serde::Deserialize)]
    struct ApproveArgs {
        from_subaccount: Option<Vec<u8>>,
        spender: Icrc2Account,
        allowance: u128,
        expires_at: Option<u64>,
    }
    let args = ApproveArgs {
        from_subaccount: None,
        spender: Icrc2Account {
            owner: ic_cdk::api::canister_self(),
            subaccount: None,
        },
        allowance: input.allowance,
        expires_at: input.expires_at,
    };
    let call = Call::unbounded_wait(can_principal, "icrc2_approve").with_arg(args);
    let res = call
        .await
        .map_err(|e| CivError::TransferCallFailed(format!("approve_call_failed:{:?}", e)))
        .and_then(|reply| {
            reply
                .candid_tuple::<(Result<u128, String>,)>()
                .map_err(|e| CivError::TransferCallFailed(format!("approve_decode_failed:{:?}", e)))
        });
    let (block_index_opt, err_opt) = match res {
        Ok((Ok(idx),)) => (Some(idx), None),
        Ok((Err(e),)) => (None, Some(e)),
        Err(e) => (None, Some(format!("{:?}", e))),
    };
    if let Some(err) = err_opt {
        return Err(CivError::Other(format!("approve_failed:{}", err)));
    }
    // Persist approval locally
    USERS.with(|users| { let mut users = users.borrow_mut(); if let Some(u)=users.get_mut(&caller) { u.approvals.retain(|a| a.asset_id != input.asset_id); u.approvals.push(ApprovalRecord { asset_id: input.asset_id, allowance: Some(input.allowance), token_id: None, granted_at: now_secs() }); push_audit(u, AuditEventKind::ApprovalSet { asset_id: input.asset_id }); if let Some(_idx)=block_index_opt { /* Could store block index later if ApprovalRecord extended */ } } });
    Ok(())
}

pub fn approval_revoke(asset_id: u64) -> Result<(), CivError> {
    let caller = user_id();
    USERS.with(|users| {
        let mut users = users.borrow_mut();
        if let Some(u) = users.get_mut(&caller) {
            assert_mutable(u)?;
            let before = u.approvals.len();
            u.approvals.retain(|a| a.asset_id != asset_id);
            if before == u.approvals.len() {
                return Err(CivError::AssetNotFound);
            }
            push_audit(u, AuditEventKind::ApprovalRevoked { asset_id });
            Ok(())
        } else {
            Err(CivError::UserNotFound)
        }
    })
}

pub fn list_approvals() -> Vec<ApprovalRecord> {
    let caller = user_id();
    USERS.with(|users| {
        let users = users.borrow();
        users
            .get(&caller)
            .map(|u| u.approvals.clone())
            .unwrap_or_default()
    })
}
