//! Optional validation for nonempty Soroban outage symbols (Issue #662).
//!
//! Symbols use ASCII letters, digits and underscores, with at most 32 bytes.
//! Case is preserved: `SF_001` and `sf_001` remain distinct identifiers.
//! Hyphens are rejected by Soroban before a Symbol can be constructed.
//!
//! This helper is available to callers; existing contract entrypoints retain
//! their current input policy. Declaring this module does not change that policy.

use soroban_sdk::{Env, Symbol, SymbolStr, TryFromVal};

use crate::SLAError;

/// Maximum byte length of a canonical outage ID.
pub const MAX_OUTAGE_ID_LEN: u32 = 32;

/// Validate that `outage_id` is in canonical form.
///
/// Returns `Ok(())` for a valid ID, or `Err(SLAError::InvalidInput)` if the
/// ID is empty, too long, or contains disallowed characters.
///
/// Validate without normalizing case or changing the supplied identifier.
pub fn validate_outage_id(env: &Env, outage_id: &Symbol) -> Result<(), SLAError> {
    let raw = SymbolStr::try_from_val(env, &outage_id.to_symbol_val()).map_err(|_| SLAError::InvalidInput)?;
    let bytes: &[u8] = raw.as_ref();
    if bytes.is_empty()
        || bytes.len() > MAX_OUTAGE_ID_LEN as usize
        || !bytes.iter().all(|b| b.is_ascii_alphanumeric() || *b == b'_')
    {
        return Err(SLAError::InvalidInput);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_symbol_alphabet_and_length_boundaries_without_normalizing() {
        let env = Env::default();
        for value in ["A", "SF_001", "sf_001", "01234567890123456789012345678901"] {
            assert_eq!(validate_outage_id(&env, &Symbol::new(&env, value)), Ok(()));
        }
        assert_eq!(
            validate_outage_id(&env, &Symbol::new(&env, "")),
            Err(SLAError::InvalidInput)
        );
        assert_ne!(Symbol::new(&env, "SF_001"), Symbol::new(&env, "sf_001"));
    }
}
