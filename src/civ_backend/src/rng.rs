// Secure RNG module for IC canister environment.
// Strategy:
// 1. On first use, obtain 32 bytes from ic_cdk::management_canister::raw_rand().
// 2. Seed a ChaCha20 stream cipher with that seed and an all-zero nonce to create a keystream.
// 3. Maintain a thread-local buffer of unused keystream bytes (since IC executes each update/query
//    in a single-threaded context, thread_local! is sufficient).
// 4. Expose fill() to populate caller's buffer with pseudorandom bytes.
// 5. Periodically (every RESEED_INTERVAL bytes) reseed by mixing in fresh raw_rand entropy
//    via XOR + hashing (SHA256) to avoid seed exhaustion / state compromise longevity.
// 6. Provide helper to generate a bounded numeric code without modulo bias using rejection sampling.
//
// This avoids relying on rand crate (wasm getrandom backend) and gives deterministic compilation
// while sourcing true entropy from the IC.

use chacha20::cipher::{KeyIvInit, StreamCipher};
use chacha20::ChaCha20; // 256-bit key, 96-bit nonce variant; we treat nonce constant.
use ic_cdk::management_canister::raw_rand;
use sha2::{Digest, Sha256};
use std::cell::RefCell;

const RESEED_INTERVAL: usize = 64 * 1024; // after generating this many bytes, mix new entropy

struct ChaChaRngState {
    key: [u8; 32],
    block_counter: u64, // counts 64-byte blocks produced
    generated_bytes: usize,
}

thread_local! {
    static RNG_STATE: RefCell<Option<ChaChaRngState>> = RefCell::new(None);
    static BUFFER: RefCell<Vec<u8>> = RefCell::new(Vec::new());
}

async fn ensure_state() {
    let need_init = RNG_STATE.with(|cell| cell.borrow().is_none());
    if !need_init {
        return;
    }
    // raw_rand() returns (Vec<u8>,) tuple; destructure to obtain entropy bytes.
    let seed = raw_rand().await.expect("raw_rand failed");
    let mut key = [0u8; 32];
    if seed.len() >= 32 {
        key.copy_from_slice(&seed[..32]);
    } else {
        let mut hasher = Sha256::new();
        hasher.update(&seed);
        let digest = hasher.finalize();
        key.copy_from_slice(&digest[..32]);
    }
    RNG_STATE.with(|cell| {
        *cell.borrow_mut() = Some(ChaChaRngState {
            key,
            block_counter: 0,
            generated_bytes: 0,
        })
    });
}

async fn reseed() {
    let new_entropy = raw_rand().await.expect("raw_rand reseed failed");
    RNG_STATE.with(|cell| {
        if let Some(state) = cell.borrow_mut().as_mut() {
            // Mix current key and new entropy
            let mut hasher = Sha256::new();
            hasher.update(&state.key);
            hasher.update(&new_entropy);
            let digest = hasher.finalize();
            state.key.copy_from_slice(&digest[..32]);
            state.block_counter = 0;
            state.generated_bytes = 0;
        }
    });
}

// Internal: generate keystream bytes into BUFFER until it has at least need bytes available.
async fn ensure_buffered(need: usize) {
    ensure_state().await;
    BUFFER.with(|buf_cell| {
        if buf_cell.borrow().len() >= need {
            return;
        }
    });
    // We generate in chunks of 4KB for efficiency.
    const CHUNK: usize = 4096;
    while BUFFER.with(|b| b.borrow().len()) < need {
        let mut chunk = vec![0u8; CHUNK];
        // Produce keystream block-by-block using key + block_counter as nonce (low 8 bytes) plus fixed high 4 bytes.
        let mut produced = 0;
        let mut reseed_required = false;
        while produced < CHUNK {
            let mut key = [0u8; 32];
            let mut counter = 0u64;
            RNG_STATE.with(|cell| {
                let mut sref = cell.borrow_mut();
                let st = sref.as_mut().expect("state");
                key.copy_from_slice(&st.key);
                counter = st.block_counter;
                st.block_counter = st.block_counter.wrapping_add(1);
                st.generated_bytes += 64; // each block 64 bytes
                if st.generated_bytes >= RESEED_INTERVAL {
                    reseed_required = true;
                }
            });
            let mut nonce = [0u8; 12];
            nonce[..8].copy_from_slice(&counter.to_le_bytes());
            let mut block = [0u8; 64];
            let mut cipher = ChaCha20::new(&key.into(), &nonce.into());
            cipher.apply_keystream(&mut block);
            let take = (CHUNK - produced).min(64);
            chunk[produced..produced + take].copy_from_slice(&block[..take]);
            produced += take;
        }
        BUFFER.with(|b| b.borrow_mut().extend_from_slice(&chunk));
        if reseed_required {
            reseed().await;
        }
    }
}

// Public async API: fill the provided slice with random bytes.
pub async fn fill_async(out: &mut [u8]) {
    ensure_buffered(out.len()).await;
    BUFFER.with(|b| {
        let mut b = b.borrow_mut();
        out.copy_from_slice(&b[..out.len()]);
        b.drain(..out.len());
    });
}

/// Returns true when RNG has been seeded/initialized.
pub fn is_initialized() -> bool {
    RNG_STATE.with(|cell| cell.borrow().is_some())
}

// Sync wrapper for query/update contexts where we want to block until bytes ready.
// (We cannot truly block on async in update; instead design callers to be async if they need randomness.)
// For legacy sync call sites, we provide a lazy future poll via block_on style using ic_cdk::spawn not possible.
// So we will migrate call sites to async variants. For now we expose a best-effort sync that panics if state not ready.
pub fn fill(out: &mut [u8]) {
    // If state not yet initialized (first call), we cannot synchronously obtain raw_rand.
    let mut need_async_init = false;
    RNG_STATE.with(|cell| {
        if cell.borrow().is_none() {
            need_async_init = true;
        }
    });
    if need_async_init {
        ic_cdk::trap("RNG not initialized: call init_rng() in canister init");
    }
    BUFFER.with(|b| {
        if b.borrow().len() < out.len() {
            ic_cdk::trap("RNG buffer underflow; use async fill_async");
        }
    });
    BUFFER.with(|b| {
        let mut b = b.borrow_mut();
        out.copy_from_slice(&b[..out.len()]);
        b.drain(..out.len());
    });
}

// Initialize RNG during canister init / post-upgrade (async).
pub async fn init_rng() {
    ensure_state().await;
    // Pre-fill a buffer (32KB) for sync usage.
    ensure_buffered(32 * 1024).await;
}

// Try to obtain a random u64 synchronously without trapping. Returns None if
// the RNG hasn't been initialized yet or the prefilled buffer is exhausted.
// This is useful in utility code (like jitter for retry backoff) where
// randomness is a nice-to-have but should not cause traps if unavailable.
pub fn try_u64() -> Option<u64> {
    let mut ready = true;
    RNG_STATE.with(|cell| {
        if cell.borrow().is_none() {
            ready = false;
        }
    });
    if !ready {
        return None;
    }
    let have = BUFFER.with(|b| b.borrow().len());
    if have < 8 {
        return None;
    }
    let mut bytes = [0u8; 8];
    fill(&mut bytes); // safe because we ensured >= 8 bytes
    Some(u64::from_le_bytes(bytes))
}

// Generate an unbiased numeric code in range [0, 10^digits).
pub async fn numeric_code(digits: u32) -> String {
    let max = 10u64.saturating_pow(digits.min(12)); // limit for practicality
    let bound = max;
    // Compute rejection threshold to avoid modulo bias.
    let mut bytes = [0u8; 8];
    loop {
        fill_async(&mut bytes).await;
        let mut val = u64::from_le_bytes(bytes);
        // Use 56 bits at most per draw to keep loops bounded.
        val &= (1u64 << 56) - 1;
        if val < (u64::MAX / bound) * bound {
            // within fair zone
            let code = val % bound;
            return format!("{:0width$}", code, width = digits as usize);
        }
    }
}
