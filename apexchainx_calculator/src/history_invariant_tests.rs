//! History metadata must agree with retained shards after every mutation.
use crate::{SLACalculatorContract, SLACalculatorContractClient, HISTORY_LEN_KEY};
use soroban_sdk::{
    symbol_short,
    testutils::{Address as _, Ledger as _},
    Address, Env, Symbol,
};

fn assert_consistent(env: &Env, contract: &Address, expected: u32) {
    env.as_contract(contract, || {
        let entries = crate::history::read_all_entries(env);
        let cached: u32 = env.storage().instance().get(&HISTORY_LEN_KEY).unwrap();
        assert_eq!(entries.len(), expected);
        assert_eq!(cached, expected);
        assert_eq!(crate::history::len(env), expected);
        for entry in entries {
            assert!(!crate::history::entry_indices_for_outage(env, &entry.outage_id).is_empty());
        }
    });
}

#[test]
fn cached_count_tracks_append_retention_count_prune_and_age_prune() {
    let env = Env::default();
    env.mock_all_auths();
    let contract = env.register_contract(None, SLACalculatorContract);
    let client = SLACalculatorContractClient::new(&env, &contract);
    let admin = Address::generate(&env);
    let operator = Address::generate(&env);
    client.initialize(&admin, &operator);
    assert_consistent(&env, &contract, 0);
    client.set_retention_limit(&admin, &3);
    for i in 0..5 {
        env.ledger().set_timestamp(100 + i as u64);
        client.calculate_sla(
            &operator,
            &Symbol::new(&env, &alloc::format!("INC{i}")),
            &symbol_short!("high"),
            &10,
        );
        assert_consistent(&env, &contract, (i + 1).min(3));
    }
    client.prune_history(&admin, &2);
    assert_consistent(&env, &contract, 2);
    env.ledger().set_timestamp(1000);
    client.prune_history_by_age(&admin, &100);
    assert_consistent(&env, &contract, 0);
}
