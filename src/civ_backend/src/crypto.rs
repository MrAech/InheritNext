use crate::rng;
use chacha20poly1305::{
    aead::{Aead, Payload},
    KeyInit, XChaCha20Poly1305, XNonce,
};
use sha2::{Digest, Sha256};

pub fn hash_secret_with_salt(secret: &str, salt: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(salt);
    hasher.update(secret.as_bytes());
    let res = hasher.finalize();
    let mut out = [0u8; 32];
    out.copy_from_slice(&res);
    out
}

pub fn generate_master_key() -> [u8; 32] {
    let mut k = [0u8; 32];
    // Use sync fill (requires pre-init in canister init). If buffer underflow occurs it will trap.
    rng::fill(&mut k);
    k
}

pub fn encrypt_xchacha(key: &[u8; 32], plaintext: &[u8]) -> (Vec<u8>, [u8; 24]) {
    let cipher = XChaCha20Poly1305::new(key.into());
    let mut nonce_bytes = [0u8; 24];
    rng::fill(&mut nonce_bytes);
    let nonce = XNonce::from_slice(&nonce_bytes);
    let ct = cipher
        .encrypt(
            nonce,
            Payload {
                msg: plaintext,
                aad: b"DOC",
            },
        )
        .expect("encryption failure");
    (ct, nonce_bytes)
}

pub fn decrypt_xchacha(
    key: &[u8; 32],
    ciphertext: &[u8],
    nonce_bytes: &[u8; 24],
) -> Option<Vec<u8>> {
    let cipher = XChaCha20Poly1305::new(key.into());
    let nonce = XNonce::from_slice(nonce_bytes);
    cipher
        .decrypt(
            nonce,
            Payload {
                msg: ciphertext,
                aad: b"DOC",
            },
        )
        .ok()
}

// Constant time comparison to mitigate timing side-channels on secret / identity hash checks
pub fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff: u8 = 0;
    for i in 0..a.len() {
        diff |= a[i] ^ b[i];
    }
    diff == 0
}

// Deterministic custody subaccount derivation: sha256("CUSTODY" || user_principal_text || heir_id_le)
// Returns 32-byte array suitable for ICRC subaccount usage.
pub fn derive_custody_subaccount(user_principal: &str, heir_id: u64) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(b"CUSTODY");
    hasher.update(user_principal.as_bytes());
    hasher.update(&heir_id.to_le_bytes());
    let digest = hasher.finalize();
    let mut out = [0u8; 32];
    out.copy_from_slice(&digest);
    out
}

// Deterministic escrow subaccount derivation (asset-level escrow): sha256("ESCROW" || user_principal || asset_id_le)
pub fn derive_escrow_subaccount(user_principal: &str, asset_id: u64) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(b"ESCROW");
    hasher.update(user_principal.as_bytes());
    hasher.update(&asset_id.to_le_bytes());
    let digest = hasher.finalize();
    let mut out = [0u8; 32];
    out.copy_from_slice(&digest);
    out
}
