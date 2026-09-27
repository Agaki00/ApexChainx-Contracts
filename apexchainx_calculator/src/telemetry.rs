//! Grouped telemetry helpers (Issue #654).
//!
//! These helpers store a separate grouped record. Existing contract entrypoints
//! still use the canonical packed telemetry keys; compiling this module does not
//! migrate their storage or switch the calculation hot path.

use soroban_sdk::{contracttype, symbol_short, Env, Map, Symbol};

/// On-chain key for the grouped telemetry snapshot.
pub(crate) const TELEMETRY_KEY: Symbol = symbol_short!("TELMET");

/// All per-severity telemetry counters grouped into one storage value.
/// This is separate from the currently deployed packed telemetry representation.
#[contracttype]
#[derive(Clone)]
pub struct SeverityTelemetryRecord {
    /// Map of severity → cumulative calculation count.
    pub calc_counts: Map<Symbol, u32>,
    /// Map of severity → cumulative violation count.
    pub viol_counts: Map<Symbol, u32>,
    /// Map of severity → timestamp of last calculation.
    pub last_calc_ts: Map<Symbol, u64>,
    /// Map of severity → timestamp of last violation.
    pub last_viol_ts: Map<Symbol, u64>,
}

/// Load the telemetry record in one storage read.
pub fn load_telemetry(env: &Env) -> SeverityTelemetryRecord {
    env.storage()
        .instance()
        .get(&TELEMETRY_KEY)
        .unwrap_or_else(|| SeverityTelemetryRecord {
            calc_counts: Map::new(env),
            viol_counts: Map::new(env),
            last_calc_ts: Map::new(env),
            last_viol_ts: Map::new(env),
        })
}

/// Persist the telemetry record in one storage write.
pub fn store_telemetry(env: &Env, record: &SeverityTelemetryRecord) {
    env.storage().instance().set(&TELEMETRY_KEY, record);
}
