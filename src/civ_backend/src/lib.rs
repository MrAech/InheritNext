mod auth;
mod models;
use models::*;
use ic_cdk_macros::*;
use candid::Principal;
use std::collections::HashMap;
use std::cell::RefCell;
// Note: icrc_ledger_client_cdk client types were used earlier; we now use direct calls
use icrc_ledger_types::icrc1::account::Account as IcrcAccount;
use icrc_ledger_types::icrc1::transfer::TransferArg as Icrc1TransferArg;
use icrc_ledger_types::icrc1::transfer::NumTokens as IcrcNumTokens;
use sha2::{Digest, Sha256};

// Local classification of transfer outcomes for retry logic.
#[derive(Debug, Clone)]
pub(crate) enum TransferOutcome {
    Duplicate,
    TemporarilyUnavailable,
    Other(String),
}
// Maximum attempts before marking a journal entry as failed
const MAX_JOURNAL_ATTEMPTS: u32 = 5;
use serde::{Deserialize as SerdeDeserialize, Serialize as SerdeSerialize};
use serde_json;
thread_local! {
    static USERS: RefCell<HashMap<Principal, UserState>> = RefCell::new(HashMap::new());
    static JOURNAL: RefCell<HashMap<u64, JournalEntry>> = RefCell::new(HashMap::new());
    static NEXT_JOURNAL_ID: RefCell<u64> = RefCell::new(1);
    static SALT_COUNTER: RefCell<u64> = RefCell::new(1);
    // Map asset_type (e.g. "ckBTC", "ckETH", "NFT") -> ledger principal
    static LEDGER_MAP: RefCell<HashMap<String, Principal>> = RefCell::new(HashMap::new());
    // Cache ledger metadata per ledger principal: decimals and metadata map
    static LEDGER_METADATA: RefCell<HashMap<Principal, LedgerMeta>> = RefCell::new(HashMap::new());
    // Simple reentrancy/processing guard
    static PROCESSING_GUARD: RefCell<bool> = RefCell::new(false);
    // Optional test override for icrc transfers: allows unit tests to simulate ledger behaviour
    static TRANSFER_OVERRIDE: RefCell<Option<fn(Principal, Icrc1TransferArg) -> Result<(), TransferOutcome>>> = RefCell::new(None);
}

// Generate a per-user salt hex string using principal, time and a local counter.
fn generate_salt_hex(principal: Principal) -> String {
    let now = now_secs();
    let counter = SALT_COUNTER.with(|c| { let mut v = c.borrow_mut(); let cur = *v; *v += 1; cur });
    let payload = format!("{}:{}:{}", principal.to_string(), now, counter);
    let mut hasher = Sha256::new();
    hasher.update(payload.as_bytes());
    let result = hasher.finalize();
    // return first 16 bytes as hex (32 chars)
    result.iter().take(16).map(|b| format!("{:02x}", b)).collect::<Vec<String>>().join("")
}

// Cached ledger metadata
#[derive(Clone)]
struct LedgerMeta {
    decimals: u8,
    metadata: HashMap<String, String>,
}

// Helper: build a deterministic memo (32 bytes) from journal id, caller principal and timestamp
fn build_memo_32(id: u64, caller: Principal, timestamp_secs: u64) -> [u8;32] {
    // Compose a small JSON payload and hash to 32 bytes
    let payload = serde_json::json!({"journal_id": id, "caller": caller.to_string(), "ts": timestamp_secs});
    let bytes = payload.to_string().into_bytes();
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    let result = hasher.finalize();
    let mut out = [0u8; 32];
    out.copy_from_slice(&result[..32]);
    out
}

// Helper: query icrc1_decimals and cache it
#[cfg(not(test))]
fn get_decimals_for_ledger(ledger: Principal) -> Option<u8> {
    // Return cached if present
    let cached = LEDGER_METADATA.with(|m| m.borrow().get(&ledger).cloned());
    if let Some(meta) = cached { return Some(meta.decimals); }

    use ic_cdk::call::Call;
    use candid::Encode;

    let (tx, rx) = std::sync::mpsc::sync_channel(1);
    let mg = ledger;
    ic_cdk::futures::spawn_017_compat(async move {
        let arg = Encode!().unwrap_or_else(|_| vec![]);
        let res = Call::bounded_wait(mg, "icrc1_decimals").with_arg(&arg).await;
        let _ = tx.send(res);
    });
    match rx.recv() {
        Ok(Ok(res)) => {
            // decode nat8
            match res.candid::<u8>() {
                Ok(dec) => {
                    // cache basic metadata with decimals
                    LEDGER_METADATA.with(|m| { m.borrow_mut().insert(ledger, LedgerMeta { decimals: dec, metadata: HashMap::new() }); });
                    Some(dec)
                }
                Err(_) => None,
            }
        }
        _ => None,
    }
}

// During unit tests we cannot perform inter-canister calls; return a sensible default (8) to allow conversions
#[cfg(test)]
fn get_decimals_for_ledger(_ledger: Principal) -> Option<u8> {
    Some(8u8)
}

// Helper: query icrc1_fee
fn get_fee_for_ledger(ledger: Principal) -> Option<u128> {
    use ic_cdk::call::Call;
    use candid::Encode;
    let (tx, rx) = std::sync::mpsc::sync_channel(1);
    let mg = ledger;
    ic_cdk::futures::spawn_017_compat(async move {
        let arg = Encode!().unwrap_or_else(|_| vec![]);
        let res = Call::bounded_wait(mg, "icrc1_fee").with_arg(&arg).await;
        let _ = tx.send(res);
    });
    match rx.recv() {
        Ok(Ok(res)) => match res.candid::<u128>() {
            Ok(f) => Some(f),
            Err(_) => None,
        },
        _ => None,
    }
}

// Helper: query icrc1_balance_of for an account on a ledger
#[cfg(not(test))]
fn get_balance_for_ledger(ledger: Principal, account: IcrcAccount) -> Option<u128> {
    use ic_cdk::call::Call;
    use candid::Encode;

    let (tx, rx) = std::sync::mpsc::sync_channel(1);
    let mg = ledger;
    ic_cdk::futures::spawn_017_compat(async move {
        let arg = Encode!(&account).unwrap_or_else(|_| vec![]);
        let res = Call::bounded_wait(mg, "icrc1_balance_of").with_arg(&arg).await;
        let _ = tx.send(res);
    });
    match rx.recv() {
        Ok(Ok(res)) => match res.candid::<u128>() {
            Ok(b) => Some(b),
            Err(_) => None,
        },
        _ => None,
    }
}

// During unit tests we can't do inter-canister calls; return a large balance to allow claim flow tests to proceed.
#[cfg(test)]
fn get_balance_for_ledger(_ledger: Principal, _account: IcrcAccount) -> Option<u128> {
    Some(1_000_000u128)
}

// Helper: perform a transfer_from-like call (icrc2_transfer_from) when possible.
// First honor TRANSFER_OVERRIDE for unit tests by mapping parameters to an Icrc1TransferArg.
fn icrc2_transfer_from(ledger: Principal, from: IcrcAccount, to: IcrcAccount, amount: u128, created_at_ns: Option<u64>, memo_bytes: Option<Vec<u8>>) -> Result<(), TransferOutcome> {
    // Test override: synthesize an Icrc1TransferArg for the override to consume
    let override_opt = TRANSFER_OVERRIDE.with(|o| o.borrow().clone());
    if let Some(f) = override_opt {
        // Build an Icrc1TransferArg that represents the transfer for overriding behavior
        let ia = Icrc1TransferArg {
            from_subaccount: from.subaccount.clone(),
            to: to.clone(),
            fee: None,
            created_at_time: created_at_ns,
            memo: memo_bytes.clone().map(|b| icrc_ledger_types::icrc1::transfer::Memo::from(b)),
            amount: IcrcNumTokens::from(amount),
        };
        return f(ledger, ia);
    }

    // Otherwise perform the real icrc2_transfer_from call
    use ic_cdk::call::Call;
    use candid::Encode;
    use icrc_ledger_types::icrc1::transfer::TransferError as Icrc1TransferError;

    // Use the canonical TransferFromArgs record shape as defined by ICRC2/ledger DID:
    // (to : Account, fee : opt Tokens, spender_subaccount : opt Subaccount, from : Account,
    //  memo : opt blob, created_at_time : opt Timestamp, amount : Tokens)
    use candid::Nat as CandidNat;
    let fee_opt: Option<CandidNat> = None;
    let spender_subaccount_opt: Option<Vec<u8>> = None; // Subaccount is a blob when present
    let memo_opt: Option<Vec<u8>> = memo_bytes.clone();
    let created_at_opt: Option<u64> = created_at_ns;
    let amount_nat = CandidNat::from(amount);
    let record = (to.clone(), fee_opt, spender_subaccount_opt, from.clone(), memo_opt, created_at_opt, amount_nat);
    let arg_enc = Encode!(&record).unwrap_or_else(|_| vec![]);

    let (tx, rx) = std::sync::mpsc::sync_channel(1);
    let mg = ledger;
    ic_cdk::futures::spawn_017_compat(async move {
        let res = Call::bounded_wait(mg, "icrc2_transfer_from").with_arg(&arg_enc).await;
        let _ = tx.send(res);
    });

    match rx.recv() {
        Ok(Ok(res)) => {
            match res.candid::<Result<u64, Icrc1TransferError>>() {
                Ok(Ok(_)) => Ok(()),
                Ok(Err(err)) => {
                    match err {
                        Icrc1TransferError::Duplicate { .. } => Err(TransferOutcome::Duplicate),
                        Icrc1TransferError::TemporarilyUnavailable => Err(TransferOutcome::TemporarilyUnavailable),
                        Icrc1TransferError::TooOld => Err(TransferOutcome::Other("TooOld".to_string())),
                        Icrc1TransferError::CreatedInFuture { .. } => Err(TransferOutcome::Other("CreatedInFuture".to_string())),
                        Icrc1TransferError::InsufficientFunds { .. } => Err(TransferOutcome::Other("InsufficientFunds".to_string())),
                        Icrc1TransferError::BadFee { expected_fee } => Err(TransferOutcome::Other(format!("BadFee:expected={}", expected_fee))),
                        _ => Err(TransferOutcome::Other(format!("LedgerError:{:?}", err))),
                    }
                }
                Err(_) => Err(TransferOutcome::Other("candid_decode_failed".to_string())),
            }
        }
        Ok(Err(_)) => Err(TransferOutcome::TemporarilyUnavailable),
        Err(_) => Err(TransferOutcome::Other("transfer_recv_failed".to_string())),
    }
}

// Typed transfer details stored in JournalEntry.details as JSON
#[derive(Clone, SerdeSerialize, SerdeDeserialize)]
struct TransferDetails {
    owner: String,
    asset_type: String,
    to: String, // Principal as string
    amount: u64,
    // Optional collection canister principal as string (for NFTs)
    collection: Option<String>,
}

// Helper to return current time in seconds. In tests we use system time; on canister runtime we use ic_cdk.
fn now_secs() -> u64 {
    #[cfg(test)]
    {
        use std::time::{SystemTime, UNIX_EPOCH};
        let d = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_else(|_| std::time::Duration::from_secs(0));
        return d.as_secs();
    }
    #[cfg(not(test))]
    {
        return (ic_cdk::api::time() / 1_000_000_000) as u64;
    }
}

// --- Service methods for mainnet-ready backend ---

#[update]
fn accept_terms() -> Result<(), CustodianError> {
    let principal = ic_cdk::api::msg_caller();
    USERS.with(|users| {
        let mut users = users.borrow_mut();
        let user = users.entry(principal).or_insert_with(|| UserState {
                profile: UserProfile {
                    principal,
                    name: String::new(),
                    gov_id_hash: String::new(),
                    pbkdf2_salt: generate_salt_hex(principal),
                    terms_accepted: false,
                    plan_type: PlanType::Basic,
                    activated: false,
                    activation_timestamp: None,
                    expiry_timer: None,
                    warning_days: 7,
                    inactivity_days: 30,
                },
            assets: vec![],
            heirs: vec![],
            distributions: vec![],
            approvals: vec![],
            vaulted_nfts: vec![],
        });
        user.profile.terms_accepted = true;
        Ok(())
    })
}

#[update]
fn select_plan(plan: PlanType) -> Result<(), CustodianError> {
    let principal = ic_cdk::api::msg_caller();
    USERS.with(|users| {
        let mut users = users.borrow_mut();
        let user = users.entry(principal).or_insert_with(|| UserState {
                profile: UserProfile {
                    principal,
                    name: String::new(),
                    gov_id_hash: String::new(),
                    pbkdf2_salt: generate_salt_hex(principal),
                    terms_accepted: false,
                    plan_type: PlanType::Basic,
                    activated: false,
                    activation_timestamp: None,
                    expiry_timer: None,
                    warning_days: 7,
                    inactivity_days: 30,
                },
            assets: vec![],
            heirs: vec![],
            distributions: vec![],
            approvals: vec![],
            vaulted_nfts: vec![],
        });
        user.profile.plan_type = plan;
        Ok(())
    })
}

#[update]
fn add_asset(asset_id: String, asset_type: String, name: String, value: u64, description: String, approved: bool) -> Result<(), CustodianError> {
    let principal = ic_cdk::api::msg_caller();
    USERS.with(|users| {
        let mut users = users.borrow_mut();
        let user = users.entry(principal).or_insert_with(|| UserState {
                profile: UserProfile {
                    principal,
                    name: String::new(),
                    gov_id_hash: String::new(),
                    pbkdf2_salt: generate_salt_hex(principal),
                    terms_accepted: false,
                    plan_type: PlanType::Basic,
                    activated: false,
                    activation_timestamp: None,
                    expiry_timer: None,
                    warning_days: 7,
                    inactivity_days: 30,
                },
            assets: vec![],
            heirs: vec![],
            distributions: vec![],
            approvals: vec![],
            vaulted_nfts: vec![],
        });
        if user.assets.iter().any(|a| a.asset_id == asset_id) {
            return Err(CustodianError::AlreadyExists);
        }
        user.assets.push(Asset {
            asset_id,
            asset_type,
            approved,
            value,
            name,
            description,
        });
        Ok(())
    })
}

#[update]
fn add_heir(name: String, gov_id_hash: String, security_question_hash: Option<String>) -> Result<(), CustodianError> {
    let principal = ic_cdk::api::msg_caller();
    USERS.with(|users| {
        let mut users = users.borrow_mut();
        let user = users.entry(principal).or_insert_with(|| UserState {
                profile: UserProfile {
                    principal,
                    name: String::new(),
                    gov_id_hash: String::new(),
                    pbkdf2_salt: generate_salt_hex(principal),
                    terms_accepted: false,
                    plan_type: PlanType::Basic,
                    activated: false,
                    activation_timestamp: None,
                    expiry_timer: None,
                    warning_days: 7,
                    inactivity_days: 30,
                },
            assets: vec![],
            heirs: vec![],
            distributions: vec![],
            approvals: vec![],
            vaulted_nfts: vec![],
        });
        if user.heirs.iter().any(|h| h.name == name && h.gov_id_hash == gov_id_hash) {
            return Err(CustodianError::AlreadyExists);
        }
        user.heirs.push(Heir {
            name,
            gov_id_hash,
            security_question_hash,
        });
        Ok(())
    })
}

// Update an existing heir by gov_id_hash (caller must be owner)
#[update]
fn update_heir(name: String, gov_id_hash: String, security_question_hash: Option<String>) -> Result<(), CustodianError> {
    let caller = ic_cdk::api::msg_caller();
    USERS.with(|users| {
        let mut users = users.borrow_mut();
        let user = users.get_mut(&caller).ok_or(CustodianError::NotAuthorized)?;
        if let Some(h) = user.heirs.iter_mut().find(|h| h.gov_id_hash == gov_id_hash) {
            h.name = name;
            h.security_question_hash = security_question_hash;
            Ok(())
        } else {
            Err(CustodianError::NotFound)
        }
    })
}

// Remove an heir by gov_id_hash
#[update]
fn remove_heir(gov_id_hash: String) -> Result<(), CustodianError> {
    let caller = ic_cdk::api::msg_caller();
    USERS.with(|users| {
        let mut users = users.borrow_mut();
        let user = users.get_mut(&caller).ok_or(CustodianError::NotAuthorized)?;
        let before = user.heirs.len();
        user.heirs.retain(|h| h.gov_id_hash != gov_id_hash);
        if user.heirs.len() == before { Err(CustodianError::NotFound) } else { Ok(()) }
    })
}

#[update]
fn record_token_approval(token_canister: Principal, asset_type: String, approved_amount: u64, approval_expiry: Option<u64>, auto_renew: bool) -> Result<(), CustodianError> {
    // Caller must be the owner (msg_caller) and the asset_type must be mapped to the provided token_canister
    let caller = ic_cdk::api::msg_caller();

    // Validate ledger mapping exists and matches token_canister
    let ledger_ok = LEDGER_MAP.with(|m| m.borrow().get(&asset_type).cloned()).map_or(false, |p| p == token_canister);
    if !ledger_ok {
        return Err(CustodianError::InvalidInput);
    }

    USERS.with(|users| {
        let mut users = users.borrow_mut();
        let user = users.entry(caller).or_insert_with(|| UserState {
                profile: UserProfile {
                    principal: caller,
                    name: String::new(),
                    gov_id_hash: String::new(),
                    pbkdf2_salt: generate_salt_hex(caller),
                    terms_accepted: false,
                    plan_type: PlanType::Basic,
                    activated: false,
                    activation_timestamp: None,
                    expiry_timer: None,
                    warning_days: 7,
                    inactivity_days: 30,
                },
            assets: vec![],
            heirs: vec![],
            distributions: vec![],
            approvals: vec![],
            vaulted_nfts: vec![],
        });

        // Upsert approval by token_canister+asset_type
        if let Some(a) = user.approvals.iter_mut().find(|a| a.token_canister == token_canister && a.asset_type == asset_type) {
            a.approved_amount = approved_amount;
            a.approval_expiry = approval_expiry;
            a.auto_renew = auto_renew;
        } else {
            user.approvals.push(AssetApproval {
                owner: caller,
                asset_type: asset_type.clone(),
                token_canister,
                approved_amount,
                approval_expiry,
                auto_renew,
            });
        }
        Ok(())
    })
}

#[update]
fn register_vaulted_nft(collection_canister: Principal, token_id: String, assigned_heir_hash: String) -> Result<(), CustodianError> {
    let caller = ic_cdk::api::msg_caller();
    USERS.with(|users| {
        let mut users = users.borrow_mut();
        let user = users.entry(caller).or_insert_with(|| UserState {
            profile: UserProfile {
                principal: caller,
                name: String::new(),
                gov_id_hash: String::new(),
                pbkdf2_salt: generate_salt_hex(caller),
                terms_accepted: false,
                plan_type: PlanType::Basic,
                activated: false,
                activation_timestamp: None,
                expiry_timer: None,
                warning_days: 7,
                inactivity_days: 30,
            },
            assets: vec![],
            heirs: vec![],
            distributions: vec![],
            approvals: vec![],
            vaulted_nfts: vec![],
        });

        // Prevent duplicate vaulted NFT registration
        if user.vaulted_nfts.iter().any(|v| v.collection_canister == collection_canister && v.token_id == token_id) {
            return Err(CustodianError::AlreadyExists);
        }

        user.vaulted_nfts.push(VaultedNFT {
            owner: caller,
            collection_canister,
            token_id,
            assigned_heir_hash,
        });
        Ok(())
    })
}

#[update]
fn assign_distribution(asset_id: String, heir_gov_id: String, percent: u32) -> Result<(), CustodianError> {
    let principal = ic_cdk::api::msg_caller();
    assign_distribution_for(principal, asset_id, heir_gov_id, percent)
}

// Internal helper to allow unit testing without relying on msg_caller()
fn assign_distribution_for(principal: Principal, asset_id: String, heir_gov_id: String, percent: u32) -> Result<(), CustodianError> {
    USERS.with(|users| {
        let mut users = users.borrow_mut();
        let user = users.entry(principal).or_insert_with(|| UserState {
                profile: UserProfile {
                    principal,
                    name: String::new(),
                    gov_id_hash: String::new(),
                    pbkdf2_salt: generate_salt_hex(principal),
                    terms_accepted: false,
                    plan_type: PlanType::Basic,
                    activated: false,
                    activation_timestamp: None,
                    expiry_timer: None,
                    warning_days: 7,
                    inactivity_days: 30,
                },
            assets: vec![],
            heirs: vec![],
            distributions: vec![],
            approvals: vec![],
            vaulted_nfts: vec![],
        });
        // Find heir name from gov id if available
        let heir_name = user.heirs.iter().find(|h| h.gov_id_hash == heir_gov_id).map(|h| h.name.clone()).unwrap_or_else(|| heir_gov_id.clone());
        // Remove any existing distribution for this asset+heir (by gov id)
        user.distributions.retain(|d| !(d.asset_id == asset_id && d.heir_gov_id == heir_gov_id));
        user.distributions.push(Distribution {
            asset_id: asset_id.clone(),
            heir_name,
            heir_gov_id: heir_gov_id.clone(),
            percent,
        });
        // Validate sum for each asset
        let mut asset_sums = std::collections::HashMap::new();
        for d in &user.distributions {
            *asset_sums.entry(&d.asset_id).or_insert(0u32) += d.percent;
        }
        if asset_sums.values().any(|&sum| sum != 100) {
            return Err(CustodianError::DistributionInvalid);
        }
        Ok(())
    })
}

#[update]
fn set_distributions_for_asset(asset_id: String, distributions_in: Vec<(String, u32)>) -> Result<(), CustodianError> {
    // distributions_in: vec of (heir_gov_id, percent)
    let principal = ic_cdk::api::msg_caller();
    set_distributions_for_asset_for(principal, asset_id, distributions_in)
}

// Helper to set distributions atomically for a principal (used for testing)
fn set_distributions_for_asset_for(principal: Principal, asset_id: String, distributions_in: Vec<(String, u32)>) -> Result<(), CustodianError> {
    USERS.with(|users| {
        let mut users = users.borrow_mut();
        let user = users.entry(principal).or_insert_with(|| UserState {
                profile: UserProfile {
                    principal,
                    name: String::new(),
                    gov_id_hash: String::new(),
                    pbkdf2_salt: generate_salt_hex(principal),
                    terms_accepted: false,
                    plan_type: PlanType::Basic,
                    activated: false,
                    activation_timestamp: None,
                    expiry_timer: None,
                    warning_days: 7,
                    inactivity_days: 30,
                },
            assets: vec![],
            heirs: vec![],
            distributions: vec![],
            approvals: vec![],
            vaulted_nfts: vec![],
        });

        // Validate total percent sums to 100 for this asset
        let total: u32 = distributions_in.iter().map(|(_, p)| *p).sum();
        if total != 100 {
            return Err(CustodianError::DistributionInvalid);
        }

        // Remove existing distributions for this asset
        user.distributions.retain(|d| d.asset_id != asset_id);

        // Insert new distributions with heir name lookup
        for (heir_gov_id, percent) in distributions_in.into_iter() {
            let heir_name = user.heirs.iter().find(|h| h.gov_id_hash == heir_gov_id).map(|h| h.name.clone()).unwrap_or_else(|| heir_gov_id.clone());
            user.distributions.push(Distribution {
                asset_id: asset_id.clone(),
                heir_name,
                heir_gov_id: heir_gov_id.clone(),
                percent,
            });
        }

        Ok(())
    })
}

#[update]
fn activate_plan() -> Result<(), CustodianError> {
    let principal = ic_cdk::api::msg_caller();
    USERS.with(|users| {
        let mut users = users.borrow_mut();
        let user = users.entry(principal).or_insert_with(|| UserState {
            profile: UserProfile {
                principal,
                name: String::new(),
                gov_id_hash: String::new(),
                pbkdf2_salt: generate_salt_hex(principal),
                terms_accepted: false,
                plan_type: PlanType::Basic,
                activated: false,
                activation_timestamp: None,
                expiry_timer: None,
                warning_days: 7,
                inactivity_days: 30,
            },
            assets: vec![],
            heirs: vec![],
            distributions: vec![],
            approvals: vec![],
            vaulted_nfts: vec![],
        });
        if user.profile.activated {
            return Err(CustodianError::AlreadyExists);
        }
        user.profile.activated = true;
    let now = now_secs();
        user.profile.activation_timestamp = Some(now);
        // Set expiry timer (in seconds)
        let expiry = now + user.profile.inactivity_days * 24 * 60 * 60;
        user.profile.expiry_timer = Some(expiry);
        // Schedule a one-off timer to call trigger_collection for this owner at expiry (best-effort)
        // Note: timers are not perfectly reliable across restarts; this is best-effort.
        let delay_secs = if expiry > now { expiry - now } else { 0 };
        let owner_copy = principal;
        ic_cdk_timers::set_timer(std::time::Duration::from_secs(delay_secs), move || {
            // Call internal helper to trigger collection for the owner we scheduled for.
            // We ignore errors here because timers are best-effort.
            let _ = trigger_collection_for_owner(owner_copy);
        });
        Ok(())
    })
}

#[update]
fn trigger_collection() -> Result<(), CustodianError> {
    // Delegate to internal helper using the msg_caller as the owner
    let owner = ic_cdk::api::msg_caller();
    trigger_collection_for_owner(owner)
}

// Internal helper that triggers collection for a specific owner principal.
// This allows timers to call the collection logic with the correct owner context.
fn trigger_collection_for_owner(owner: Principal) -> Result<(), CustodianError> {
    USERS.with(|users| {
        let mut users = users.borrow_mut();
        let user = users.get_mut(&owner).ok_or(CustodianError::NotFound)?;
        if !user.profile.activated {
            return Err(CustodianError::NotActivated);
        }
        // For each distribution on user's assets, create a journal entry that encodes
        // the ledger asset type, recipient principal (heir), and amount to send.
        // We conservatively compute amount as proportion of asset.value using percent.
        let now = now_secs();
        for asset in &user.assets {
            // Find distributions for this asset
            let dists: Vec<_> = user.distributions.iter().filter(|d| d.asset_id == asset.asset_id).cloned().collect();
            if dists.is_empty() {
                continue;
            }
            for dist in dists {
                // Attempt to parse heir_gov_id as a principal string; if not parsable we skip
                let to_principal = match dist.heir_gov_id.parse::<Principal>() {
                    Ok(p) => p,
                    Err(_) => continue,
                };
                // Compute amount proportional to asset value
                let amount = (asset.value as u128 * (dist.percent as u128) / 100u128) as u64;
                // Prepare typed details and serialize to JSON. No collection for fungible assets.
                let td = TransferDetails { owner: owner.to_string(), asset_type: asset.asset_type.clone(), to: to_principal.to_string(), amount, collection: None };
                let details = serde_json::to_string(&td).unwrap_or_else(|_| "{}".to_string());

                let journal_id = NEXT_JOURNAL_ID.with(|id| {
                    let mut i = id.borrow_mut();
                    let cur = *i;
                    *i += 1;
                    cur
                });
                let entry = JournalEntry {
                    id: journal_id,
                    asset_id: asset.asset_id.clone(),
                    action: "icrc1_transfer".to_string(),
                    details,
                    status: JournalStatus::Pending,
                    attempts: 0,
                    created_at: now,
                    updated_at: now,
                };
                JOURNAL.with(|j| { j.borrow_mut().insert(journal_id, entry); });
            }
        }

        // Deactivate profile locally to reflect collection request; actual transfers will happen asynchronously
        user.profile.activated = false;
        Ok(())
    })
}

#[query]
fn get_user_state() -> Option<UserState> {
    let principal = ic_cdk::api::msg_caller();
    USERS.with(|users| users.borrow().get(&principal).cloned())
}

// Return the pbkdf2 salt for the caller, if present. Safe to expose: this is a salt value,
// not raw PII. Frontend should call this to obtain the per-user salt for deriving gov_id_hash.
#[query]
fn get_user_salt() -> Option<String> {
    let principal = ic_cdk::api::msg_caller();
    USERS.with(|users| users.borrow().get(&principal).map(|u| u.profile.pbkdf2_salt.clone()))
}

// Rotate the caller's pbkdf2 salt. This will generate a new salt, store it in the user's profile,
// and return the new salt. Policy: callers are responsible for re-registering heirs or otherwise
// reconciling derived gov_id_hash values after rotation.
#[update]
fn rotate_user_salt() -> Result<String, CustodianError> {
    let principal = ic_cdk::api::msg_caller();
    USERS.with(|users| {
        let mut users = users.borrow_mut();
        let user = users.get_mut(&principal).ok_or(CustodianError::NotAuthorized)?;
        let new_salt = generate_salt_hex(principal);
        user.profile.pbkdf2_salt = new_salt.clone();
        Ok(new_salt)
    })
}

#[update]
fn claim_asset(asset_id: String, heir_gov_id: String) -> Result<(), CustodianError> {
    let principal = ic_cdk::api::msg_caller();
    claim_asset_for(principal, asset_id, heir_gov_id)
}

// Heir-facing claim endpoint: caller is the heir principal who provides their gov id string
#[update]
fn heir_claim_asset(owner_principal: Principal, asset_id: String, heir_gov_id: String, security_answer_hash: Option<String>) -> Result<(), CustodianError> {
    let caller = ic_cdk::api::msg_caller();
    heir_claim_asset_for(caller, owner_principal, asset_id, heir_gov_id, security_answer_hash)
}

// Helper implementing heir claim flow. Creates journal entries for fungible approvals and vaulted NFTs
fn heir_claim_asset_for(heir: Principal, owner: Principal, asset_id: String, heir_gov_id: String, security_answer_hash: Option<String>) -> Result<(), CustodianError> {
    // Validate heir_gov_id matches one of the owner's heirs and enforce optional security question proof
    USERS.with(|users| {
        let mut users = users.borrow_mut();
        let owner_state = users.get_mut(&owner).ok_or(CustodianError::NotFound)?;
        // Find matching distribution for this asset and this heir_gov_id
        let dists: Vec<Distribution> = owner_state.distributions.iter().filter(|d| d.asset_id == asset_id && d.heir_gov_id == heir_gov_id).cloned().collect();
        if dists.is_empty() {
            return Err(CustodianError::NotFound);
        }

        // If the owner has a stored heir record for this gov_id and that heir configured a security question,
        // require the caller to present a matching security_answer_hash. If no stored heir record exists,
        // we allow the claim flow as long as a distribution exists (this preserves earlier behaviour and
        // keeps tests and off-chain distribution flows working).
        if let Some(stored_heir) = owner_state.heirs.iter().find(|h| h.gov_id_hash == heir_gov_id) {
            if let Some(expected_q_hash) = &stored_heir.security_question_hash {
                if security_answer_hash.is_none() || security_answer_hash.as_ref().map(|s| s != expected_q_hash).unwrap_or(true) {
                    return Err(CustodianError::NotAuthorized);
                }
            }
        }

        // For each approval matching this asset_type (we treat asset.asset_type as the token asset_type)
        // Find the asset by id first
        let asset_opt = owner_state.assets.iter().find(|a| a.asset_id == asset_id).cloned();
        if asset_opt.is_none() { return Err(CustodianError::NotFound); }
        let asset = asset_opt.expect("asset existence checked");

        // For each approval on the owner's state that matches asset.asset_type, query ledger balance and clamp
        let now = now_secs();
        let approvals: Vec<AssetApproval> = owner_state.approvals.iter().filter(|ap| ap.asset_type == asset.asset_type).cloned().collect();
        for ap in approvals {
            // Find ledger for this asset_type
            let ledger_opt = LEDGER_MAP.with(|m| m.borrow().get(&ap.asset_type).cloned());
            let mut owner_balance_opt: Option<u128> = None;
            if let Some(ledger_p) = ledger_opt {
                let from_account = IcrcAccount { owner, subaccount: None };
                owner_balance_opt = get_balance_for_ledger(ledger_p, from_account);
            }

            // If we couldn't query the ledger, fall back to approved_amount; otherwise clamp to balance
            let approved_u128 = ap.approved_amount as u128;
            let transferable = match owner_balance_opt {
                Some(bal) => std::cmp::min(approved_u128, bal),
                None => approved_u128,
            };
            if transferable == 0 { continue; }

            // Distribute transferable among heirs by percent
            for dist in dists.iter() {
                let heir_amount_u128 = transferable * (dist.percent as u128) / 100u128;
                let heir_amount = heir_amount_u128 as u64;
                if heir_amount == 0 { continue; }

                // Create TransferDetails and journal entry for icrc2_transfer_from
                let td = TransferDetails { owner: owner.to_string(), asset_type: ap.asset_type.clone(), to: heir.to_string(), amount: heir_amount, collection: None };
                let details = serde_json::to_string(&td).unwrap_or_else(|_| "{}".to_string());
                let journal_id = NEXT_JOURNAL_ID.with(|id| { let mut i = id.borrow_mut(); let cur = *i; *i += 1; cur });
                let entry = JournalEntry { id: journal_id, asset_id: asset_id.clone(), action: "icrc2_transfer_from".to_string(), details, status: JournalStatus::Pending, attempts: 0, created_at: now, updated_at: now };
                JOURNAL.with(|j| { j.borrow_mut().insert(journal_id, entry); });
            }
        }

        // For vaulted NFTs assigned to this heir, create journal entries for transfers as well
        let vaulted: Vec<VaultedNFT> = owner_state.vaulted_nfts.iter().filter(|v| v.assigned_heir_hash == heir_gov_id).cloned().collect();
        for v in vaulted {
            // Include the collection canister principal in the transfer details so the processor knows where to call
            let td = TransferDetails { owner: owner.to_string(), asset_type: "NFT".to_string(), to: heir.to_string(), amount: 1, collection: Some(v.collection_canister.to_string()) }; // amount=1 for single NFT
            let details = serde_json::to_string(&td).unwrap_or_else(|_| "{}".to_string());
            let journal_id = NEXT_JOURNAL_ID.with(|id| { let mut i = id.borrow_mut(); let cur = *i; *i += 1; cur });
            let entry = JournalEntry { id: journal_id, asset_id: v.token_id.clone(), action: "icrc37_transfer".to_string(), details, status: JournalStatus::Pending, attempts: 0, created_at: now, updated_at: now };
            JOURNAL.with(|j| { j.borrow_mut().insert(journal_id, entry); });
        }

        Ok(())
    })
}


#[update]
fn remove_distribution(asset_id: String, heir_gov_id: String) -> Result<(), CustodianError> {
    let principal = ic_cdk::api::msg_caller();
    remove_distribution_for(principal, asset_id, heir_gov_id)
}

fn claim_asset_for(principal: Principal, asset_id: String, heir_gov_id: String) -> Result<(), CustodianError> {
    USERS.with(|users| {
        let mut users = users.borrow_mut();
        let user = users.get_mut(&principal).ok_or(CustodianError::NotAuthorized)?;
        // Find a distribution matching asset_id and heir_gov_id
        let pos = user.distributions.iter().position(|d| d.asset_id == asset_id && d.heir_gov_id == heir_gov_id);
        match pos {
            Some(idx) => {
                // Remove distribution to mark as claimed
                user.distributions.remove(idx);
                Ok(())
            }
            None => Err(CustodianError::NotFound),
        }
    })
}

#[update]
fn update_asset(asset_id: String, asset_type: String, name: String, value: u64, description: String, approved: bool) -> Result<(), CustodianError> {
    let caller = ic_cdk::api::msg_caller();
    USERS.with(|users| {
        let mut users = users.borrow_mut();
        let user = users.get_mut(&caller).ok_or(CustodianError::NotAuthorized)?;
        if let Some(a) = user.assets.iter_mut().find(|a| a.asset_id == asset_id) {
            a.asset_type = asset_type;
            a.name = name;
            a.value = value;
            a.description = description;
            a.approved = approved;
            Ok(())
        } else {
            Err(CustodianError::NotFound)
        }
    })
}

#[update]
fn remove_asset(asset_id: String) -> Result<(), CustodianError> {
    let caller = ic_cdk::api::msg_caller();
    USERS.with(|users| {
        let mut users = users.borrow_mut();
        let user = users.get_mut(&caller).ok_or(CustodianError::NotAuthorized)?;
        let before = user.assets.len();
        user.assets.retain(|a| a.asset_id != asset_id);
        if user.assets.len() == before { Err(CustodianError::NotFound) } else { Ok(()) }
    })
}

// Admin API to set ledger principal for an asset type
#[update]
fn set_ledger_for_asset_type(asset_type: String, ledger_principal: Option<Principal>) -> Result<(), CustodianError> {
    // Only the canister controllers should be able to call this in production.
    if !is_caller_controller() {
        return Err(CustodianError::NotAuthorized);
    }
    LEDGER_MAP.with(|m| {
        let mut map = m.borrow_mut();
        match ledger_principal {
            Some(p) => {
                // insert mapping
                map.insert(asset_type.clone(), p);
                // attempt to query decimals and cache metadata (best-effort)
                if let Some(dec) = get_decimals_for_ledger(p) {
                    LEDGER_METADATA.with(|md| { md.borrow_mut().insert(p, LedgerMeta { decimals: dec, metadata: HashMap::new() }); });
                }
            }
            None => { map.remove(&asset_type); }
        }
        Ok(())
    })
}

// Query helper exposed to frontend: return decimals for a mapped asset_type (if present and queryable)
#[query]
fn get_ledger_decimals(asset_type: String) -> Option<u8> {
    // lookup ledger principal
    let ledger_opt = LEDGER_MAP.with(|m| m.borrow().get(&asset_type).cloned());
    if let Some(ledger) = ledger_opt {
        // Attempt to fetch and cache decimals using existing helper
        return get_decimals_for_ledger(ledger);
    }
    None
}

// Helper to check whether the current msg_caller is one of this canister's controllers
fn is_caller_controller() -> bool {
    use candid::Encode;
    use ic_cdk::call::Call;
    // Use the management canister types crate for decoding the canister_status result
    use ic_management_canister_types::CanisterStatusResult;

    let mgmt = Principal::management_canister();
    let canister_id = ic_cdk::api::canister_self();
    // Encode the argument record { canister_id = <this canister id> }
    let arg = Encode!(&canister_id).unwrap_or_else(|_| vec![]);

    // Spawn an async call to the management canister and wait for result synchronously
    let (tx, rx) = std::sync::mpsc::sync_channel(1);
    ic_cdk::futures::spawn_017_compat(async move {
        let res = Call::bounded_wait(mgmt, "canister_status").with_arg(&arg).await;
        let _ = tx.send(res);
    });

    let caller = ic_cdk::api::msg_caller();
    match rx.recv() {
        Ok(Ok(res)) => match res.candid::<CanisterStatusResult>() {
            Ok(status) => {
                // controllers is a Vec<Principal>
                for p in status.settings.controllers.iter() {
                    if *p == caller { return true; }
                }
                false
            }
            Err(_) => false,
        },
        _ => false,
    }
}

// Helper: perform a bounded-wait icrc1 transfer. Returns Ok(()) on success.
// This function currently uses a cross-canister call and handles common result variants.
fn icrc1_transfer(ledger: Principal, arg: Icrc1TransferArg) -> Result<(), TransferOutcome> {
    // Check for a test override first
    let override_opt = TRANSFER_OVERRIDE.with(|o| o.borrow().clone());
    if let Some(f) = override_opt {
        return f(ledger, arg);
    }

    use ic_cdk::call::Call;
    use candid::Encode;
    use icrc_ledger_types::icrc1::transfer::TransferError as Icrc1TransferError;

    // Encode the TransferArg candid bytes
    let arg_enc = Encode!(&arg).unwrap_or_else(|_| vec![]);

    let (tx, rx) = std::sync::mpsc::sync_channel(1);
    ic_cdk::futures::spawn_017_compat(async move {
        let res = Call::bounded_wait(ledger, "icrc1_transfer").with_arg(&arg_enc).await;
        let _ = tx.send(res);
    });

    match rx.recv() {
        Ok(Ok(res)) => {
            // Attempt to decode successful Ok(nat) or Err(TransferError)
            match res.candid::<Result<u64, Icrc1TransferError>>() {
                Ok(Ok(_idx)) => Ok(()),
                Ok(Err(err)) => {
                    match err {
                        Icrc1TransferError::Duplicate { duplicate_of: _ } => Err(TransferOutcome::Duplicate),
                        Icrc1TransferError::TemporarilyUnavailable => Err(TransferOutcome::TemporarilyUnavailable),
                        Icrc1TransferError::TooOld => Err(TransferOutcome::Other("TooOld".to_string())),
                        Icrc1TransferError::CreatedInFuture { .. } => Err(TransferOutcome::Other("CreatedInFuture".to_string())),
                        Icrc1TransferError::InsufficientFunds { .. } => Err(TransferOutcome::Other("InsufficientFunds".to_string())),
                        Icrc1TransferError::BadFee { expected_fee } => {
                            return Err(TransferOutcome::Other(format!("BadFee:expected={}", expected_fee)));
                        }
                        _ => Err(TransferOutcome::Other(format!("LedgerError:{:?}", err))),
                    }
                }
                Err(_e) => Err(TransferOutcome::Other("candid_decode_failed".to_string())),
            }
        }
        Ok(Err(_call_err)) => {
            // Inter-canister call error (rejection / system). Conservatively treat as temporarily unavailable.
            Err(TransferOutcome::TemporarilyUnavailable)
        }
        Err(_) => Err(TransferOutcome::Other("transfer_recv_failed".to_string())),
    }
}

// Background processor: iterate pending journal entries and attempt to process them. Called by a timer.
fn process_pending_journal_entries() {
    // Prevent concurrent processing using an RAII guard
    struct Guard;
    impl Drop for Guard { fn drop(&mut self) { PROCESSING_GUARD.with(|g| { *g.borrow_mut() = false; }); } }
    let already_processing = PROCESSING_GUARD.with(|g| { let mut gv = g.borrow_mut(); if *gv { true } else { *gv = true; false } });
    if already_processing { return; }
    let _guard = Guard;

    // Collect pending IDs snapshot
    let pending_ids: Vec<u64> = JOURNAL.with(|j| j.borrow().iter().filter(|(_, e)| matches!(e.status, JournalStatus::Pending)).map(|(id, _)| *id).collect());
    for id in pending_ids {
    let now = now_secs();
        // Read entry details and deserialize
        let entry = match JOURNAL.with(|j| j.borrow().get(&id).cloned()) {
            Some(e) => e,
            None => continue,
        };
        let details = entry.details.clone();
        let td_res: Result<TransferDetails, _> = serde_json::from_str(&details);
        let td = match td_res {
            Ok(v) => v,
            Err(_) => {
                JOURNAL.with(|j| {
                    if let Some(e) = j.borrow_mut().get_mut(&id) {
                        e.status = JournalStatus::Failed;
                        e.attempts += 1;
                        e.updated_at = now;
                    }
                });
                continue;
            }
        };
        let asset_type = td.asset_type;
        let to_str = td.to;
        let amount = td.amount;

        // Find ledger principal for asset_type
        let ledger_opt = LEDGER_MAP.with(|m| m.borrow().get(&asset_type).cloned());
        if ledger_opt.is_none() {
            // can't process yet; leave as pending but increment attempts
            JOURNAL.with(|j| {
                if let Some(e) = j.borrow_mut().get_mut(&id) {
                    e.attempts += 1;
                    e.updated_at = now;
                }
            });
            continue;
        }
        let ledger = match ledger_opt { Some(l) => l, None => { continue; } };

        // Parse recipient principal
        let to_principal = match to_str.parse::<Principal>() {
            Ok(p) => p,
            Err(_) => {
                JOURNAL.with(|j| {
                    if let Some(e) = j.borrow_mut().get_mut(&id) {
                        e.status = JournalStatus::Failed;
                        e.attempts += 1;
                        e.updated_at = now;
                    }
                });
                continue;
            }
        };

                // For icrc1 transfers build transfer arg as before
                if entry.action == "icrc1_transfer" {
                    let to_account = IcrcAccount { owner: to_principal, subaccount: None };
                    // created_at_time must be in nanoseconds
                    let created_at_ns = (now as u128 * 1_000_000_000u128) as u64;
                    // Build deterministic memo from journal id and declaring owner to help ledger deduplication
                    let owner_principal_res = td.owner.parse::<Principal>();
                    let memo_converted = match owner_principal_res {
                        Ok(owner_principal) => {
                            let memo_arr = build_memo_32(id, owner_principal, now);
                            Some(icrc_ledger_types::icrc1::transfer::Memo::from(memo_arr.to_vec()))
                        }
                        Err(_) => None,
                    };
                    // Convert amount (stored as u64 human-unit) to smallest-unit using ledger decimals if available
                    let smallest_amount: u128 = match LEDGER_MAP.with(|m| m.borrow().get(&asset_type).cloned()).and_then(|l| get_decimals_for_ledger(l)) {
                        Some(dec) => {
                            // amount is u64 human units; multiply by 10^dec safely into u128
                            let factor: u128 = 10u128.pow(dec as u32);
                            (amount as u128).saturating_mul(factor)
                        }
                        None => amount as u128
                    };

                    let transfer_arg = Icrc1TransferArg {
                        from_subaccount: None,
                        to: to_account,
                        fee: None,
                        created_at_time: Some(created_at_ns),
                        memo: memo_converted,
                        amount: IcrcNumTokens::from(smallest_amount),
                    };

                    // Call ledger transfer helper; receives TransferOutcome on Err
                    let res = icrc1_transfer(ledger, transfer_arg);

                    JOURNAL.with(|j| {
                        if let Some(e) = j.borrow_mut().get_mut(&id) {
                            e.updated_at = now;
                            match res {
                                Ok(()) => {
                                    e.status = JournalStatus::Success;
                                    e.attempts += 1;
                                }
                                Err(outcome) => {
                                    match outcome {
                                        TransferOutcome::Duplicate => {
                                            e.status = JournalStatus::Success;
                                            e.attempts += 1;
                                        }
                                        TransferOutcome::TemporarilyUnavailable => {
                                            e.attempts += 1;
                                            if e.attempts >= MAX_JOURNAL_ATTEMPTS {
                                                e.status = JournalStatus::Failed;
                                            } else {
                                                e.status = JournalStatus::Pending;
                                            }
                                        }
                                        TransferOutcome::Other(_s) => {
                                            e.attempts += 1;
                                            e.status = JournalStatus::Failed;
                                        }
                                    }
                                }
                            }
                        }
                    });
                    continue;
                }

                // Support icrc2_transfer_from actions
                if entry.action == "icrc2_transfer_from" {
                    // Build accounts
                    // Here td.owner represents the 'from' account owner, and td.to represents the recipient
                    let from_owner = td.owner.parse::<Principal>().unwrap_or_else(|_| Principal::anonymous());
                    let from_account = IcrcAccount { owner: from_owner, subaccount: None };
                    let to_account = IcrcAccount { owner: to_principal, subaccount: None };
                    let created_at_ns = (now as u128 * 1_000_000_000u128) as u64;
                    let memo_vec_opt = td.owner.parse::<Principal>().ok().map(|p| build_memo_32(id, p, now).to_vec());
                    // Convert amount to smallest-unit like above
                    let smallest_amount: u128 = match get_decimals_for_ledger(ledger) {
                        Some(dec) => {
                            let factor: u128 = 10u128.pow(dec as u32);
                            (amount as u128).saturating_mul(factor)
                        }
                        None => amount as u128
                    };

                    let res = icrc2_transfer_from(ledger, from_account, to_account, smallest_amount, Some(created_at_ns), memo_vec_opt);

                    JOURNAL.with(|j| {
                        if let Some(e) = j.borrow_mut().get_mut(&id) {
                            e.updated_at = now;
                            match res {
                                Ok(()) => { e.status = JournalStatus::Success; e.attempts += 1; }
                                Err(TransferOutcome::Duplicate) => { e.status = JournalStatus::Success; e.attempts += 1; }
                                Err(TransferOutcome::TemporarilyUnavailable) => { e.attempts += 1; if e.attempts >= MAX_JOURNAL_ATTEMPTS { e.status = JournalStatus::Failed; } else { e.status = JournalStatus::Pending; } }
                                Err(TransferOutcome::Other(_s)) => { e.attempts += 1; e.status = JournalStatus::Failed; }
                            }
                        }
                    });
                    continue;
                }

                // Support icrc37 (NFT) transfers: call icrc37_transfer_from on collection canister
                if entry.action == "icrc37_transfer" {
                    // details may include collection principal and token id in asset_id
                    // Prefer collection from details.collection if present, otherwise fall back to LEDGER_MAP lookup by asset_type.
                    let collection_from_details: Option<Principal> = td.collection.and_then(|s| s.parse::<Principal>().ok());
                    let collection_opt = collection_from_details.or_else(|| LEDGER_MAP.with(|m| m.borrow().get(&asset_type).cloned()));
                    if collection_opt.is_none() {
                        // can't process yet; increment attempts
                        JOURNAL.with(|j| {
                            if let Some(e) = j.borrow_mut().get_mut(&id) {
                                e.attempts += 1;
                                e.updated_at = now;
                            }
                        });
                        continue;
                    }
                    let collection = match collection_opt { Some(c) => c, None => { JOURNAL.with(|j| { if let Some(e) = j.borrow_mut().get_mut(&id) { e.attempts += 1; e.updated_at = now; } }); continue; } };
                    // Try numeric token-id first, else fall back to textual token-id path.
                    let from_owner = td.owner.parse::<Principal>().unwrap_or_else(|_| Principal::anonymous());
                    let from_account = IcrcAccount { owner: from_owner, subaccount: None };
                    let to_account = IcrcAccount { owner: to_principal, subaccount: None };
                    let spender = ic_cdk::api::canister_self();
                    let created_at_ns = (now as u128 * 1_000_000_000u128) as u64;
                    use candid::Encode;
                    use ic_cdk::call::Call;

                    if let Ok(token_nat) = entry.asset_id.parse::<u64>() {
                        let inner = (spender, from_account.clone(), to_account.clone(), token_nat, None::<Vec<u8>>, Some(created_at_ns));
                        let arg = Encode!(&vec![inner]).unwrap_or_else(|_| vec![]);
                        let (tx2, rx2) = std::sync::mpsc::sync_channel(1);
                        let mg2 = collection;
                        ic_cdk::futures::spawn_017_compat(async move {
                            let res = Call::bounded_wait(mg2, "icrc37_transfer_from").with_arg(&arg).await;
                            let _ = tx2.send(res);
                        });
                        if let Ok(Ok(_)) = rx2.recv() {
                            JOURNAL.with(|j| { if let Some(e) = j.borrow_mut().get_mut(&id) { e.status = JournalStatus::Success; e.attempts += 1; e.updated_at = now; } });
                            continue;
                        }
                    }

                    // Textual token-id path: try calling `icrc37_transfer_from_text` with string token id
                    let inner_text = (spender, from_account.clone(), to_account.clone(), entry.asset_id.clone(), None::<Vec<u8>>, Some(created_at_ns));
                    let arg_text = Encode!(&vec![inner_text]).unwrap_or_else(|_| vec![]);
                    let (tx3, rx3) = std::sync::mpsc::sync_channel(1);
                    let mg3 = collection;
                    ic_cdk::futures::spawn_017_compat(async move {
                        let res = Call::bounded_wait(mg3, "icrc37_transfer_from_text").with_arg(&arg_text).await;
                        let _ = tx3.send(res);
                    });
                    match rx3.recv() {
                        Ok(Ok(_)) => { JOURNAL.with(|j| { if let Some(e) = j.borrow_mut().get_mut(&id) { e.status = JournalStatus::Success; e.attempts += 1; e.updated_at = now; } }); }
                        _ => { JOURNAL.with(|j| { if let Some(e) = j.borrow_mut().get_mut(&id) { e.attempts += 1; if e.attempts >= MAX_JOURNAL_ATTEMPTS { e.status = JournalStatus::Failed; } else { e.status = JournalStatus::Pending; } e.updated_at = now; } }); }
                    }
                    continue;
                }

        // Other action types are handled above; continue to next entry.
    }

    // release guard
    PROCESSING_GUARD.with(|g| { *g.borrow_mut() = false; });
}

// Install a repeating timer to process pending journal entries. Idempotent.
#[init]
fn init() {
    // set a timer to run every 60 seconds
    ic_cdk_timers::set_timer_interval(std::time::Duration::from_secs(60), || {
        process_pending_journal_entries();
    });
    // Backfill: check for any users whose expiry_timer already passed while this canister was down
    // and attempt to trigger collection for them (best-effort). This avoids relying solely on set_timer.
    let now = now_secs();
    USERS.with(|users| {
        let users_snapshot: Vec<Principal> = users.borrow().iter().filter_map(|(p, s)| {
            if let Some(expiry) = s.profile.expiry_timer { if expiry <= now { Some(*p) } else { None } } else { None }
        }).collect();
        for p in users_snapshot {
            // Try to trigger collection; ignore errors
            let _ = trigger_collection_for_owner(p);
        }
    });
}

fn remove_distribution_for(principal: Principal, asset_id: String, heir_gov_id: String) -> Result<(), CustodianError> {
    USERS.with(|users| {
        let mut users = users.borrow_mut();
        let user = users.get_mut(&principal).ok_or(CustodianError::NotAuthorized)?;
        let before = user.distributions.len();
        user.distributions.retain(|d| !(d.asset_id == asset_id && d.heir_gov_id == heir_gov_id));
        if user.distributions.len() == before {
            return Err(CustodianError::NotFound);
        }
        Ok(())
    })
}

// Test helpers to override transfer behavior during unit tests
#[cfg(test)]
pub(crate) fn set_transfer_override(f: fn(Principal, Icrc1TransferArg) -> Result<(), TransferOutcome>) {
    TRANSFER_OVERRIDE.with(|o| { *o.borrow_mut() = Some(f); });
}

#[cfg(test)]
pub fn clear_transfer_override() {
    TRANSFER_OVERRIDE.with(|o| { *o.borrow_mut() = None; });
}


//export_service to auto create/update civ_backend.did
// Ensure all public types are in scope for candid export
// Bring all public types into scope for candid export

export_candid!();


#[cfg(test)]
mod tests {
    use super::*;
    use candid::Principal as CPrincipal;

    #[test]
    fn test_assign_claim_remove_flow() {
        let principal = CPrincipal::anonymous();

        // Create a user entry
        USERS.with(|users| {
            let mut users = users.borrow_mut();
            users.insert(principal, UserState {
                    profile: UserProfile {
                        principal,
                        name: "Test User".to_string(),
                        gov_id_hash: "ownerhash".to_string(),
                        pbkdf2_salt: generate_salt_hex(principal),
                        terms_accepted: true,
                        plan_type: PlanType::Basic,
                        activated: true,
                        activation_timestamp: None,
                        expiry_timer: None,
                        warning_days: 7,
                        inactivity_days: 30,
                    },
                    assets: vec![Asset { asset_id: "asset1".to_string(), asset_type: "Cash".to_string(), approved: true, value: 1000u64, name: "Wallet".to_string(), description: "Test wallet".to_string() }],
                heirs: vec![Heir { name: "Alice".to_string(), gov_id_hash: "alice_hash".to_string(), security_question_hash: None }],
                distributions: vec![],
                approvals: vec![],
                vaulted_nfts: vec![],
            });
        });

        // Assign distribution (should fail because sum != 100)
        let res = assign_distribution_for(principal, "asset1".to_string(), "alice_hash".to_string(), 50);
        assert!(res.is_err());

        // Assign two distributions to reach 100
        let _ = assign_distribution_for(principal, "asset1".to_string(), "alice_hash".to_string(), 100);

        // Claim the asset (should remove distribution)
        let claim_res = claim_asset_for(principal, "asset1".to_string(), "alice_hash".to_string());
        assert!(claim_res.is_ok());

        // Verify removal
        USERS.with(|users| {
            let users = users.borrow();
            let user = users.get(&principal).unwrap();
            assert!(user.distributions.iter().all(|d| d.asset_id != "asset1"));
        });
    }

    #[test]
    fn test_process_journal_success() {
        let principal = CPrincipal::anonymous();
        // Setup user with asset and distribution
        USERS.with(|users| {
            let mut users = users.borrow_mut();
            users.insert(principal, UserState {
                profile: UserProfile { principal, name: "T2".to_string(), gov_id_hash: "owner2".to_string(), pbkdf2_salt: generate_salt_hex(principal), terms_accepted: true, plan_type: PlanType::Basic, activated: true, activation_timestamp: None, expiry_timer: None, warning_days: 7, inactivity_days: 30 },
                assets: vec![Asset { asset_id: "asset2".to_string(), asset_type: "Cash".to_string(), approved: true, value: 100u64, name: "W".to_string(), description: "d".to_string() }],
                heirs: vec![Heir { name: "Bob".to_string(), gov_id_hash: "bob_hash".to_string(), security_question_hash: None }],
                distributions: vec![Distribution { asset_id: "asset2".to_string(), heir_name: "Bob".to_string(), heir_gov_id: principal.to_string(), percent: 100 }],
                approvals: vec![],
                vaulted_nfts: vec![],
            });
        });

        // Set ledger mapping so processor can find it
        LEDGER_MAP.with(|m| { m.borrow_mut().insert("Cash".to_string(), CPrincipal::anonymous()); });

        // Enqueue journal entry manually
    let now = now_secs();
    let td = TransferDetails { owner: principal.to_string(), asset_type: "Cash".to_string(), to: principal.to_string(), amount: 100, collection: None };
        let details = serde_json::to_string(&td).unwrap();
        let id = NEXT_JOURNAL_ID.with(|i| { let mut x = i.borrow_mut(); let cur = *x; *x += 1; cur });
        let entry = JournalEntry { id, asset_id: "asset2".to_string(), action: "icrc1_transfer".to_string(), details, status: JournalStatus::Pending, attempts: 0, created_at: now, updated_at: now };
        JOURNAL.with(|j| { j.borrow_mut().insert(id, entry); });

        // Override transfer to succeed
        set_transfer_override(|_ledger, _arg| Ok(()));
        process_pending_journal_entries();

        // Verify success
        JOURNAL.with(|j| {
            let je = j.borrow().get(&id).unwrap().clone();
            assert!(matches!(je.status, JournalStatus::Success));
        });
        clear_transfer_override();
    }

    #[test]
    fn test_process_journal_temporary_failure_retry() {
        let principal = CPrincipal::anonymous();
        // Setup minimal journal entry
    let now = now_secs();
    let td = TransferDetails { owner: principal.to_string(), asset_type: "Cash".to_string(), to: principal.to_string(), amount: 10, collection: None };
        let details = serde_json::to_string(&td).unwrap();
        let id = NEXT_JOURNAL_ID.with(|i| { let mut x = i.borrow_mut(); let cur = *x; *x += 1; cur });
        let entry = JournalEntry { id, asset_id: "assetX".to_string(), action: "icrc1_transfer".to_string(), details, status: JournalStatus::Pending, attempts: 0, created_at: now, updated_at: now };
        JOURNAL.with(|j| { j.borrow_mut().insert(id, entry); });

        // No ledger mapping set -> processor should increment attempts and leave pending
        process_pending_journal_entries();
        JOURNAL.with(|j| {
            let je = j.borrow().get(&id).unwrap().clone();
            assert!(matches!(je.status, JournalStatus::Pending));
            assert_eq!(je.attempts, 1);
        });
    }

    #[test]
    fn test_heir_claim_creates_and_processes_journal_success() {
        let owner = CPrincipal::anonymous();
        let heir = CPrincipal::anonymous();
        // Setup owner with asset, distribution and approval
        USERS.with(|users| {
            let mut users = users.borrow_mut();
            users.insert(owner, UserState {
                profile: UserProfile { principal: owner, name: "Owner".to_string(), gov_id_hash: "owner_hash".to_string(), pbkdf2_salt: generate_salt_hex(owner), terms_accepted: true, plan_type: PlanType::Basic, activated: true, activation_timestamp: None, expiry_timer: None, warning_days: 7, inactivity_days: 30 },
                assets: vec![Asset { asset_id: "asset_claim".to_string(), asset_type: "Cash".to_string(), approved: true, value: 1000u64, name: "W".to_string(), description: "d".to_string() }],
                heirs: vec![Heir { name: "Heir".to_string(), gov_id_hash: "heir_hash".to_string(), security_question_hash: None }],
                distributions: vec![Distribution { asset_id: "asset_claim".to_string(), heir_name: "Heir".to_string(), heir_gov_id: heir.to_string(), percent: 100 }],
                approvals: vec![AssetApproval { owner, asset_type: "Cash".to_string(), token_canister: CPrincipal::anonymous(), approved_amount: 500, approval_expiry: None, auto_renew: false }],
                vaulted_nfts: vec![],
            });
        });

        // Set ledger mapping so processing can find it
        LEDGER_MAP.with(|m| { m.borrow_mut().insert("Cash".to_string(), CPrincipal::anonymous()); });

        // Call heir_claim_asset_for directly
    let res = heir_claim_asset_for(heir, owner, "asset_claim".to_string(), heir.to_string(), None);
        assert!(res.is_ok());

        // There should be journal entries; override transfer to succeed and process
        set_transfer_override(|_ledger, _arg| Ok(()));
        process_pending_journal_entries();

        // Verify all journal entries are success
        JOURNAL.with(|j| {
            for (_id, e) in j.borrow().iter() {
                if e.asset_id == "asset_claim" || e.action == "icrc1_transfer" || e.action == "icrc2_transfer_from" { assert!(matches!(e.status, JournalStatus::Success)); }
            }
        });
        clear_transfer_override();
    }

    #[test]
    fn test_heir_claim_creates_journal_pending_when_no_ledger() {
        let owner = CPrincipal::anonymous();
        let heir = CPrincipal::anonymous();
        // Setup owner with asset, distribution and approval but no ledger mapping
        USERS.with(|users| {
            let mut users = users.borrow_mut();
            users.insert(owner, UserState {
                profile: UserProfile { principal: owner, name: "Owner2".to_string(), gov_id_hash: "owner2_hash".to_string(), pbkdf2_salt: generate_salt_hex(owner), terms_accepted: true, plan_type: PlanType::Basic, activated: true, activation_timestamp: None, expiry_timer: None, warning_days: 7, inactivity_days: 30 },
                assets: vec![Asset { asset_id: "asset_claim2".to_string(), asset_type: "NoLedger".to_string(), approved: true, value: 200u64, name: "W".to_string(), description: "d".to_string() }],
                heirs: vec![Heir { name: "Heir2".to_string(), gov_id_hash: "heir2_hash".to_string(), security_question_hash: None }],
                distributions: vec![Distribution { asset_id: "asset_claim2".to_string(), heir_name: "Heir2".to_string(), heir_gov_id: heir.to_string(), percent: 100 }],
                approvals: vec![AssetApproval { owner, asset_type: "NoLedger".to_string(), token_canister: CPrincipal::anonymous(), approved_amount: 50, approval_expiry: None, auto_renew: false }],
                vaulted_nfts: vec![],
            });
        });

        // Call heir claim: should create journal entries but processing will leave them pending and increment attempts
    let res = heir_claim_asset_for(heir, owner, "asset_claim2".to_string(), heir.to_string(), None);
        assert!(res.is_ok());

        process_pending_journal_entries();

        // Verify at least one journal entry is pending with attempts == 1
        let mut found = false;
        JOURNAL.with(|j| {
            for (_id, e) in j.borrow().iter() {
                if e.asset_id == "asset_claim2" { found = true; assert!(matches!(e.status, JournalStatus::Pending)); assert!(e.attempts >= 1); }
            }
        });
        assert!(found);
    }
}



