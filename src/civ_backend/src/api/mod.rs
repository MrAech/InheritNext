// Domain segmented API modules extracted from former monolithic api.rs
// Each submodule exposes public canister methods. Shared helpers live in common.rs.

pub mod assets;
pub mod bridge;
pub mod ckbridge; // newly extracted ck (chain-wrapped) withdrawal logic
pub mod common;
pub mod custody;
pub mod distributions;
pub mod documents;
pub mod escrow;
pub mod executor; // estate lifecycle, execution, timers, stable upgrade hooks
pub mod heirs;
pub mod nft_adapter; // unified NFT transfer adapter
pub mod notify; // notification queue & dispatch scaffold
pub mod reconciliation; // added
pub mod retry; // now modularized (folder with submodules) // RNG seeding & utilities
               // Re-export frequently used DTO structs so callers can use api::Type
pub use distributions::ReadinessReport;
pub use escrow::{ApprovalSetInput, EscrowDepositInput};
pub use heirs::{HeirClaimInput, HeirClaimResult};
