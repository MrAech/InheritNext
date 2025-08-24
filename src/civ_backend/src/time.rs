// Time utilities (seconds precision)
pub fn now_secs() -> u64 {
    ic_cdk::api::time() / 1_000_000_000
}



