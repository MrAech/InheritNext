// Document vault related APIs extracted from former monolithic api.rs
use crate::api::common::{assert_mutable, user_id, require_owner};
use crate::audit::push_audit;
use crate::crypto::{decrypt_xchacha, encrypt_xchacha, generate_master_key};
use crate::models::document::{
    DocumentChunk, DocumentUploadInit, DocumentUploadSession, MAX_CHUNK_BYTES,
    MAX_CONCURRENT_UPLOADS,
};
use crate::models::*;
use crate::storage::USERS;
use crate::time::now_secs;
use sha2::{Digest, Sha256};

// Add (store) an encrypted document for the calling user. Returns new document id.
pub fn add_document(input: DocumentAddInput) -> Result<u64, CivError> {
    let caller = user_id();
    USERS.with(|users| {
        let mut users = users.borrow_mut();
        if let Some(u) = users.get_mut(&caller) {
            require_owner(u)?; // explicit owner check (heirs cannot add documents)
            assert_mutable(u)?;
            let next_id = u.documents.iter().map(|d| d.id).max().unwrap_or(0) + 1;
            let now = now_secs();
            if input.data.len() > crate::models::document::MAX_DOC_BYTES {
                return Err(CivError::Other("document_too_large".into()));
            }
            if u.doc_master_key.is_none() {
                u.doc_master_key = Some(generate_master_key().to_vec());
            }
            let mut key = [0u8; 32];
            key.copy_from_slice(u.doc_master_key.as_ref().unwrap());
            // Pre-encryption checksum over plaintext to attest integrity.
            let mut hasher = Sha256::new();
            hasher.update(&input.data);
            let checksum = hasher.finalize();
            let mut checksum_arr = [0u8; 32];
            checksum_arr.copy_from_slice(&checksum);
            let (ciphertext, nonce) = encrypt_xchacha(&key, &input.data);
            u.documents.push(DocumentEntry {
                id: next_id,
                name: input.name,
                mime_type: input.mime_type,
                size: ciphertext.len() as u64,
                encrypted_data: ciphertext,
                nonce,
                created_at: now,
                checksum_sha256: Some(checksum_arr),
            });
            Ok(next_id)
        } else {
            Err(CivError::UserNotFound)
        }
    })
}

// List metadata (still encrypted content) for all documents of caller.
pub fn list_documents() -> Vec<DocumentEntry> {
    let caller = user_id();
    USERS.with(|u| {
        let u = u.borrow();
        u.get(&caller)
            .map(|x| x.documents.clone())
            .unwrap_or_default()
    })
}

// Heir fetch of a single document (decrypts) once estate locked/executed and heir verified.
pub fn heir_get_document(
    heir_id: u64,
    doc_id: u64,
) -> Result<Option<(DocumentEntry, Vec<u8>)>, CivError> {
    let caller = user_id(); // owner-centric for now; future: separate heir principal flow
    USERS.with(|users| {
        let mut users = users.borrow_mut();
        if let Some(u) = users.get_mut(&caller) {
            // Phase gating: only allow access once estate is fully Executed (stricter policy)
            if u.phase != EstatePhase::Executed {
                return Ok(None);
            }
            // ensure heir is verified
            if !u
                .heirs_v2
                .iter()
                .any(|h| h.id == heir_id && h.identity_secret.status == HeirSecretStatus::Verified)
            {
                return Ok(None);
            }
            // Find index first to avoid holding immutable borrow while mutating (for audit push)
            if let Some(idx) = u.documents.iter().position(|d| d.id == doc_id) {
                // Clone out the document so we release borrows before mutating audit log
                let doc = u.documents[idx].clone();
                let ok_plain = (|| {
                    if let Some(key_bytes) = &u.doc_master_key {
                        if key_bytes.len() == 32 {
                            let mut key = [0u8; 32];
                            key.copy_from_slice(key_bytes);
                            if let Some(plaintext) =
                                decrypt_xchacha(&key, &doc.encrypted_data, &doc.nonce)
                            {
                                return Some(plaintext);
                            }
                        }
                    }
                    None
                })();

                if let Some(plaintext) = ok_plain {
                    push_audit(u, AuditEventKind::DocAccessed { doc_id, heir_id });
                    return Ok(Some((doc, plaintext)));
                }
                return Ok(None);
            } else {
                Ok(None)
            }
        } else {
            Err(CivError::UserNotFound)
        }
    })
}

// Start a chunked upload session, returning an upload id.
pub fn start_document_upload(init: DocumentUploadInit) -> Result<u64, CivError> {
    let caller = user_id();
    if init.expected_plain_size as usize > crate::models::document::MAX_DOC_BYTES {
        return Err(CivError::Other("document_too_large".into()));
    }
    USERS.with(|users| {
        let mut users = users.borrow_mut();
        if let Some(u) = users.get_mut(&caller) {
            assert_mutable(u)?;
            // Limit the scope of the &mut borrow into doc_uploads
            // Hard cap for combined in-flight plaintext memory (simple heuristic: 3 * MAX_DOC_BYTES)
            const MAX_INFLIGHT_BYTES: usize = crate::models::document::MAX_DOC_BYTES * 3;
            let (next_id, name_for_audit) = {
                let uploads = u.doc_uploads.get_or_insert_with(|| vec![]);
                // Evict oldest (by started_at) if exceeding concurrent limit
                if uploads.len() >= MAX_CONCURRENT_UPLOADS {
                    // find oldest
                    if let Some((idx, _)) = uploads
                        .iter()
                        .enumerate()
                        .min_by_key(|(_, s)| s.started_at)
                    {
                        uploads.remove(idx);
                    }
                }
                // Enforce aggregate memory guard
                let aggregate: usize = uploads.iter().map(|s| s.expected_plain_size as usize).sum();
                if aggregate + (init.expected_plain_size as usize) > MAX_INFLIGHT_BYTES {
                    return Err(CivError::Other("inflight_upload_memory_limit".into()));
                }
                let next_id = uploads.iter().map(|s| s.upload_id).max().unwrap_or(0) + 1;
                uploads.push(DocumentUploadSession {
                    upload_id: next_id,
                    name: init.name.clone(),
                    mime_type: init.mime_type.clone(),
                    expected_plain_size: init.expected_plain_size,
                    expected_sha256: init.expected_sha256,
                    received_plain: 0,
                    plaintext: Vec::with_capacity(init.expected_plain_size as usize),
                    started_at: now_secs(),
                });
                (next_id, init.name)
            };
            // uploads borrow is dropped here
            push_audit(
                u,
                AuditEventKind::DocUploadStarted {
                    upload_id: next_id,
                    name: name_for_audit,
                },
            );
            Ok(next_id)
        } else {
            Err(CivError::UserNotFound)
        }
    })
}

// Append a chunk to an existing upload. Returns current received bytes.
pub fn upload_document_chunk(chunk: DocumentChunk) -> Result<u64, CivError> {
    if chunk.data.len() > MAX_CHUNK_BYTES {
        return Err(CivError::Other("chunk_too_large".into()));
    }
    let caller = user_id();
    USERS.with(|users| {
        let mut users = users.borrow_mut();
        if let Some(u) = users.get_mut(&caller) {
            // Work inside a block to drop the inner borrow before auditing
            let result_and_bytes = {
                if let Some(list) = u.doc_uploads.as_mut() {
                    if let Some(sess) = list.iter_mut().find(|s| s.upload_id == chunk.upload_id) {
                        let new_total = sess.received_plain as usize + chunk.data.len();
                        if new_total > sess.expected_plain_size as usize {
                            return Err(CivError::Other("exceeds_expected_size".into()));
                        }
                        if new_total > crate::models::document::MAX_DOC_BYTES {
                            return Err(CivError::Other("document_too_large".into()));
                        }
                        sess.plaintext.extend_from_slice(&chunk.data);
                        sess.received_plain = new_total as u64;
                        // capture for audit after borrow ends
                        Ok::<(u64, u64), CivError>((sess.upload_id, sess.received_plain))
                    } else {
                        Err(CivError::Other("upload_not_found".into()))
                    }
                } else {
                    Err(CivError::Other("upload_not_found".into()))
                }
            };

            match result_and_bytes {
                Ok((upload_id, received_plain)) => {
                    push_audit(
                        u,
                        AuditEventKind::DocUploadChunkAppended {
                            upload_id,
                            bytes: chunk.data.len() as u64,
                        },
                    );
                    Ok(received_plain)
                }
                Err(e) => Err(e),
            }
        } else {
            Err(CivError::UserNotFound)
        }
    })
}

// Finalize an upload, encrypting and promoting to a DocumentEntry. Returns new document id.
pub fn finalize_document_upload(upload_id: u64) -> Result<u64, CivError> {
    let caller = user_id();
    USERS.with(|users| {
        let mut users = users.borrow_mut();
        if let Some(u) = users.get_mut(&caller) {
            assert_mutable(u)?;
            if u.doc_master_key.is_none() {
                u.doc_master_key = Some(generate_master_key().to_vec());
            }
            let mut key = [0u8; 32];
            key.copy_from_slice(u.doc_master_key.as_ref().unwrap());

            // Remove the session first (moves it out; no borrow remains)
            let sess = {
                if let Some(list) = u.doc_uploads.as_mut() {
                    if let Some(idx) = list.iter().position(|s| s.upload_id == upload_id) {
                        list.remove(idx)
                    } else {
                        return Err(CivError::Other("upload_not_found".into()));
                    }
                } else {
                    return Err(CivError::Other("upload_not_found".into()));
                }
            };

            if sess.received_plain != sess.expected_plain_size {
                return Err(CivError::Other("size_mismatch".into()));
            }
            if let Some(exp) = sess.expected_sha256 {
                let mut hasher = Sha256::new();
                hasher.update(&sess.plaintext);
                let got = hasher.finalize();
                if got[..] != exp {
                    return Err(CivError::Other("checksum_mismatch".into()));
                }
            }

            // Compute checksum for storage metadata (over plaintext)
            let mut hasher = Sha256::new();
            hasher.update(&sess.plaintext);
            let checksum = hasher.finalize();
            let mut checksum_arr = [0u8; 32];
            checksum_arr.copy_from_slice(&checksum);

            let (ciphertext, nonce) = encrypt_xchacha(&key, &sess.plaintext);
            let next_id = u.documents.iter().map(|d| d.id).max().unwrap_or(0) + 1;
            let now = now_secs();
            let stored_size = ciphertext.len() as u64;

            u.documents.push(DocumentEntry {
                id: next_id,
                name: sess.name,
                mime_type: sess.mime_type,
                size: stored_size,
                encrypted_data: ciphertext,
                nonce,
                created_at: now,
                checksum_sha256: Some(checksum_arr),
            });

            // Use the actual stored size in audit
            push_audit(
                u,
                AuditEventKind::DocUploadFinalized {
                    upload_id,
                    doc_id: next_id,
                    size: stored_size,
                },
            );
            Ok(next_id)
        } else {
            Err(CivError::UserNotFound)
        }
    })
}

// Abort (discard) an in-progress upload.
pub fn abort_document_upload(upload_id: u64, reason: String) -> Result<(), CivError> {
    let caller = user_id();
    USERS.with(|users| {
        let mut users = users.borrow_mut();
        if let Some(u) = users.get_mut(&caller) {
            // Remove inside a block to drop the &mut borrow into doc_uploads before auditing
            let removed = {
                if let Some(list) = u.doc_uploads.as_mut() {
                    if let Some(idx) = list.iter().position(|s| s.upload_id == upload_id) {
                        list.remove(idx);
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            };
            if removed {
                push_audit(u, AuditEventKind::DocUploadAborted { upload_id, reason });
                Ok(())
            } else {
                Err(CivError::Other("upload_not_found".into()))
            }
        } else {
            Err(CivError::UserNotFound)
        }
    })
}
