use crate::models::NftStandard;
use candid::Principal;

#[derive(Clone, Debug)]
pub enum NftTransferOutcome {
    Success { note: String },
    Failure { code: String, note: String },
}

// Normalized error codes (string constants) to allow stable mapping while avoiding enlarging candid with new enums yet.
// These are prefixed with NFT_ for disambiguation at higher layers.
fn normalize_dip721_error(raw: &str) -> String {
    // Common DIP721 patterns (example placeholders; adapt when real patterns observed)
    if raw.contains("TokenNotFound") {
        return "NFT_DIP721_TOKEN_NOT_FOUND".into();
    }
    if raw.contains("Unauthorized") {
        return "NFT_DIP721_UNAUTHORIZED".into();
    }
    if raw.contains("InvalidTokenId") {
        return "NFT_DIP721_INVALID_TOKEN".into();
    }
    format!("NFT_DIP721_OTHER:{}", raw)
}

fn normalize_ext_error(raw: &str) -> String {
    if raw.contains("TokenNotFound") {
        return "NFT_EXT_TOKEN_NOT_FOUND".into();
    }
    if raw.contains("Unauthorized") {
        return "NFT_EXT_UNAUTHORIZED".into();
    }
    if raw.contains("AccountNotFound") {
        return "NFT_EXT_ACCOUNT_NOT_FOUND".into();
    }
    format!("NFT_EXT_OTHER:{}", raw)
}

#[async_trait::async_trait]
pub trait NftTransferAdapter {
    async fn transfer(
        &self,
        canister: Principal,
        to: Principal,
        token_id: u64,
    ) -> NftTransferOutcome;
}

pub struct Dip721Adapter;

#[async_trait::async_trait]
impl NftTransferAdapter for Dip721Adapter {
    async fn transfer(
        &self,
        canister: Principal,
        to: Principal,
        token_id: u64,
    ) -> NftTransferOutcome {
        use ic_cdk::call::Call;
        let call = Call::unbounded_wait(canister, "transfer").with_arg((to, token_id));
        let res = call
            .await
            .map_err(|e| format!("call failed: {:?}", e))
            .and_then(|resp| {
                resp.candid_tuple::<(Result<(), String>,)>()
                    .map_err(|e| format!("decode err: {:?}", e))
            });
        match res {
            Ok((Ok(()),)) => NftTransferOutcome::Success {
                note: "dip721_transfer".into(),
            },
            Ok((Err(e),)) => {
                let norm = normalize_dip721_error(&e);
                NftTransferOutcome::Failure {
                    code: norm,
                    note: "dip721_transfer_failed".into(),
                }
            }
            Err(e) => {
                let norm = normalize_dip721_error(&e);
                NftTransferOutcome::Failure {
                    code: norm,
                    note: "dip721_transfer_failed".into(),
                }
            }
        }
    }
}

pub struct ExtAdapter;

#[async_trait::async_trait]
impl NftTransferAdapter for ExtAdapter {
    async fn transfer(
        &self,
        canister: Principal,
        to: Principal,
        token_id: u64,
    ) -> NftTransferOutcome {
        use ic_cdk::call::Call;
        #[derive(candid::CandidType, serde::Deserialize)]
        struct ExtTransferArgs<'a>(
            Option<Principal>,
            Principal,
            u64,
            Option<&'a [u8]>,
            bool,
            Option<&'a [u8]>,
            Option<u128>,
        );
        let args = ExtTransferArgs(None, to, token_id, None, false, None, None);
        let call = Call::unbounded_wait(canister, "ext_transfer").with_arg(args);
        let res = call
            .await
            .map_err(|e| format!("call failed: {:?}", e))
            .and_then(|resp| {
                resp.candid_tuple::<(Result<u128, String>,)>()
                    .map_err(|e| format!("decode err: {:?}", e))
            });
        match res {
            Ok((Ok(_v),)) => NftTransferOutcome::Success {
                note: "ext_transfer".into(),
            },
            Ok((Err(e),)) => {
                let norm = normalize_ext_error(&e);
                NftTransferOutcome::Failure {
                    code: norm,
                    note: "ext_transfer_failed".into(),
                }
            }
            Err(e) => {
                let norm = normalize_ext_error(&e);
                NftTransferOutcome::Failure {
                    code: norm,
                    note: "ext_transfer_failed".into(),
                }
            }
        }
    }
}

pub fn adapter_for(standard: Option<NftStandard>) -> Box<dyn NftTransferAdapter + Send + Sync> {
    match standard.unwrap_or(NftStandard::Dip721) {
        // default to DIP721
        NftStandard::Dip721 => Box::new(Dip721Adapter),
        NftStandard::Ext => Box::new(ExtAdapter),
        NftStandard::Other(_) => Box::new(Dip721Adapter), // unsupported handled earlier
    }
}
