use crate::models::*;
use candid::Principal;
use ic_cdk::call::Call;

// Real bridge submission for ckBTC & ckETH tokens (additional chain-wrapped assets can extend this).
// We eliminated the earlier simulation/trait abstraction – this function now executes actual canister calls.

// ckBTC interface (retrieve_btc)
#[derive(candid::CandidType, serde::Deserialize)]
struct RetrieveBtcArgs {
    amount: u64,
    address: String,
}
#[derive(candid::CandidType, serde::Deserialize)]
struct RetrieveBtcOk {
    block_index: u64,
}
#[derive(candid::CandidType, serde::Deserialize)]
enum RetrieveBtcError {
    TemporarilyUnavailable,
    AlreadyProcessing,
    AmountTooLow,
    FeeNotFound,
    Reimbursed,
    InvalidAddress,
    Other,
}

// ckETH interface (aligned with ckETH minter DID)
// withdraw_eth : (WithdrawalArg) -> (variant { Ok : RetrieveEthRequest; Err : WithdrawalError });
#[derive(candid::CandidType, serde::Deserialize)]
struct WithdrawalArg {
    recipient: String,
    amount: candid::Nat,
    from_subaccount: Option<Vec<u8>>,
}
#[derive(candid::CandidType, serde::Deserialize)]
struct RetrieveEthRequest {
    block_index: candid::Nat,
}
#[derive(candid::CandidType, serde::Deserialize)]
enum WithdrawalError {
    AmountTooLow { min_withdrawal_amount: candid::Nat },
    InsufficientFunds { balance: candid::Nat },
    InsufficientAllowance { allowance: candid::Nat },
    RecipientAddressBlocked { address: String },
    TemporarilyUnavailable(String),
}

// Submit a withdrawal; returns tx_id (block_index) if accepted.
pub async fn submit_bridge_withdraw(
    token_canister: Principal,
    amount: u128,
    l1_address: &str,
    chain_kind: ChainWrappedKind,
) -> Result<Option<String>, CivError> {
    if amount == 0 {
        return Err(CivError::bridge_err(
            BridgeErrorKind::FeeShortfall,
            "amount_zero",
        ));
    }
    if amount > u64::MAX as u128 {
        return Err(CivError::bridge_err(
            BridgeErrorKind::Other,
            "amount_too_large",
        ));
    }
    match chain_kind {
        ChainWrappedKind::CkBtc => {
            let args = (RetrieveBtcArgs {
                amount: amount as u64,
                address: l1_address.to_string(),
            },);
            let call = Call::unbounded_wait(token_canister, "retrieve_btc").with_arg(args);
            let res = call
                .await
                .map_err(|e| {
                    CivError::bridge_err(BridgeErrorKind::Network, format!("call_failed:{:?}", e))
                })
                .and_then(|reply| {
                    reply
                        .candid_tuple::<(Result<RetrieveBtcOk, RetrieveBtcError>,)>()
                        .map_err(|e| {
                            CivError::bridge_err(
                                BridgeErrorKind::Other,
                                format!("decode_err:{:?}", e),
                            )
                        })
                })?;
            match res.0 {
                Ok(ok) => Ok(Some(ok.block_index.to_string())),
                Err(err) => {
                    use RetrieveBtcError::*;
                    let (k, msg) = match err {
                        AmountTooLow => (BridgeErrorKind::FeeShortfall, "amount_too_low"),
                        FeeNotFound => (BridgeErrorKind::FeeShortfall, "fee_not_found"),
                        InvalidAddress => (BridgeErrorKind::Rejected, "invalid_address"),
                        TemporarilyUnavailable => {
                            (BridgeErrorKind::Timeout, "temporarily_unavailable")
                        }
                        AlreadyProcessing => (BridgeErrorKind::Rejected, "already_processing"),
                        Reimbursed => (BridgeErrorKind::Reimbursed, "reimbursed"),
                        Other => (BridgeErrorKind::Other, "other"),
                    };
                    Err(CivError::bridge_err(k, msg))
                }
            }
        }
        ChainWrappedKind::CkEth => {
            let arg = WithdrawalArg {
                recipient: l1_address.to_string(),
                amount: candid::Nat::from(amount),
                from_subaccount: None,
            };
            let call = Call::unbounded_wait(token_canister, "withdraw_eth").with_arg((arg,));
            let res = call
                .await
                .map_err(|e| {
                    CivError::bridge_err(BridgeErrorKind::Network, format!("call_failed:{:?}", e))
                })
                .and_then(|reply| {
                    reply
                        .candid_tuple::<(Result<RetrieveEthRequest, WithdrawalError>,)>()
                        .map_err(|e| {
                            CivError::bridge_err(
                                BridgeErrorKind::Other,
                                format!("decode_err:{:?}", e),
                            )
                        })
                })?;
            match res.0 {
                Ok(req) => Ok(Some(req.block_index.to_string())),
                Err(err) => {
                    use WithdrawalError::*;
                    let (k, msg) = match err {
                        AmountTooLow { .. } => (BridgeErrorKind::FeeShortfall, "amount_too_low"),
                        InsufficientFunds { .. } => {
                            (BridgeErrorKind::Rejected, "insufficient_funds")
                        }
                        InsufficientAllowance { .. } => {
                            (BridgeErrorKind::Rejected, "insufficient_allowance")
                        }
                        RecipientAddressBlocked { .. } => {
                            (BridgeErrorKind::Rejected, "recipient_blocked")
                        }
                        TemporarilyUnavailable(text) => classify_eth_temp_unavail(&text),
                    };
                    Err(CivError::bridge_err(k, msg))
                }
            }
        }
    }
}

// Heuristic classification of ckETH TemporarilyUnavailable errors into finer-grained categories.
// Examples of raw messages we might observe:
//  - "rate limited: X" => classify as RateLimited
//  - "unauthorized chain id" => UnauthorizedChain
//  - generic fallback => Timeout
fn classify_eth_temp_unavail(s: &str) -> (BridgeErrorKind, &'static str) {
    let lower = s.to_ascii_lowercase();
    if lower.contains("rate limit") || lower.contains("rate_limit") {
        (BridgeErrorKind::RateLimited, "rate_limited")
    } else if lower.contains("unauthorized")
        || lower.contains("wrong chain")
        || lower.contains("chain id")
    {
        (BridgeErrorKind::UnauthorizedChain, "unauthorized_chain")
    } else {
        (BridgeErrorKind::Timeout, "temporarily_unavailable")
    }
}
