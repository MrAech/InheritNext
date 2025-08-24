use candid::{CandidType, Deserialize};

#[derive(Clone, CandidType, Deserialize)]
pub struct DocumentEntry {
    pub id: u64,
    pub name: String,
    pub mime_type: String,
    pub size: u64,
    pub encrypted_data: Vec<u8>,
    pub nonce: [u8; 24],
    pub created_at: u64,
    #[serde(default)]
    pub checksum_sha256: Option<[u8; 32]>,
}

#[derive(Clone, CandidType, Deserialize)]
pub struct DocumentAddInput {
    pub name: String,
    pub mime_type: String,
    pub data: Vec<u8>,
}

// New chunked upload scaffolding
#[derive(Clone, CandidType, Deserialize)]
pub struct DocumentUploadInit {
    pub name: String,
    pub mime_type: String,
    pub expected_plain_size: u64,
    pub expected_sha256: Option<[u8; 32]>,
}

#[derive(Clone, CandidType, Deserialize)]
pub struct DocumentUploadSession {
    pub upload_id: u64,
    pub name: String,
    pub mime_type: String,
    pub expected_plain_size: u64,
    pub expected_sha256: Option<[u8; 32]>,
    pub received_plain: u64,
    pub plaintext: Vec<u8>,
    pub started_at: u64,
}

#[derive(Clone, CandidType, Deserialize)]
pub struct DocumentChunk {
    pub upload_id: u64,
    pub data: Vec<u8>,
}

// Hard size ceiling to guard memory bloat (initial pragmatic limit ~10MB).
pub const MAX_DOC_BYTES: usize = 10_000_000;
// Limit for in-progress chunked uploads kept in memory
pub const MAX_CONCURRENT_UPLOADS: usize = 4;
pub const MAX_CHUNK_BYTES: usize = 512 * 1024; // 512KB per chunk guard
