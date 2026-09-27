//! Canonical rent estimate with a machine-readable approximation flag.
//!
//! Re-export the contract endpoint's actual wire type. A second contracttype
//! with the same wire name would make the generated ABI ambiguous.
pub use crate::RentEstimate;
