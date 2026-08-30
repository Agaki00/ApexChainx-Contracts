//! Storage footprint and rent cost estimation helper functions.
//!
//! Provides functions to calculate the byte size footprint of stored history entries
//! and estimate per-ledger storage rent costs for administrators.

use crate::{SLAConfig, SLAError, CUSTOM_CONFIG_KEY, HISTORY_KEY};
use soroban_sdk::{Env, Map, Symbol, Vec};

/// Calculates the estimated total storage footprint (in bytes) of the contract,
/// including fixed instance storage keys, history records, and custom severities.
pub fn get_storage_footprint_estimate(env: &Env) -> Result<u64, SLAError> {
    crate::SLACalculatorContract::check_version(env)?;

    let history_len = env
        .storage()
        .instance()
        .get::<Symbol, Vec<crate::SLAResult>>(&HISTORY_KEY)
        .map_or(0, |h| h.len() as u64);

    let custom_count = env
        .storage()
        .instance()
        .get::<Symbol, Map<Symbol, SLAConfig>>(&CUSTOM_CONFIG_KEY)
        .map_or(0, |m| m.len() as u64);

    // Base contract instance overhead (~1,024 bytes)
    let base_bytes: u64 = 1024;
    // Estimated byte size per SLAResult history entry (~120 bytes)
    let bytes_per_history_entry: u64 = 120;
    // Estimated byte size per custom severity entry (~150 bytes)
    let bytes_per_custom_severity: u64 = 150;

    let footprint =
        base_bytes + (history_len * bytes_per_history_entry) + (custom_count * bytes_per_custom_severity);

    Ok(footprint)
}

/// Calculates an **approximate** per-ledger storage rent cost (in stroops)
/// based on the current storage footprint.
///
/// **Disclaimer (#459):** This is a relative growth proxy, not an
/// authoritative rent figure. The formula (`footprint / 10 + 1`) is a
/// placeholder approximation. Actual Stellar rent depends on network
/// parameters (rent fee per byte per ledger, minimum rent, etc.) that
/// are not available to the Soroban host in this SDK version.
///
/// Operators should use this value to track **relative** storage cost
/// growth over time, not as an absolute budgeting number.
pub fn get_rent_estimate(env: &Env) -> Result<i128, SLAError> {
    crate::SLACalculatorContract::check_version(env)?;
    let footprint = get_storage_footprint_estimate(env)? as i128;
    // Relative proxy: ~1 stroop per 10 bytes per ledger + 1 base stroop.
    // See doc comment — this is not derived from network parameters.
    let rent_per_ledger = (footprint / 10) + 1;
    Ok(rent_per_ledger)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{SLACalculatorContract, SLACalculatorContractClient};
    use soroban_sdk::{symbol_short, testutils::Address as _, Address};

    fn setup() -> (Env, SLACalculatorContractClient<'static>, Address, Address) {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register_contract(None, SLACalculatorContract);
        let client = SLACalculatorContractClient::new(&env, &contract_id);
        let admin = Address::generate(&env);
        let operator = Address::generate(&env);
        client.initialize(&admin, &operator);
        (env, client, admin, operator)
    }

    #[test]
    fn test_storage_footprint_estimate_grows_with_history() {
        let (_env, client, _admin, operator) = setup();

        let initial_footprint = client.get_storage_footprint_estimate();
        assert!(initial_footprint >= 1024);

        let initial_rent = client.get_rent_estimate();
        assert!(initial_rent > 0);

        // Add 5 history entries with distinct outage IDs
        let outage_ids = [
            symbol_short!("SF001"),
            symbol_short!("SF002"),
            symbol_short!("SF003"),
            symbol_short!("SF004"),
            symbol_short!("SF005"),
        ];
        for (i, outage_id) in outage_ids.iter().enumerate() {
            client.calculate_sla(
                &operator,
                outage_id,
                &symbol_short!("critical"),
                &((i as u32) + 1),
            );
        }

        let updated_footprint = client.get_storage_footprint_estimate();
        assert!(updated_footprint > initial_footprint);
        assert_eq!(updated_footprint, initial_footprint + (5 * 120));

        let updated_rent = client.get_rent_estimate();
        assert!(updated_rent >= initial_rent);
    }
}
