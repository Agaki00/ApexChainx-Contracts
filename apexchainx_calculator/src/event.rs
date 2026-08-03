use soroban_sdk::{contracttype, Env, Symbol};

/// Structured event payload for calculation execution events.
///
/// This struct is the canonical event shape emitted when a business-logic
/// calculation completes. It is defined here rather than in `calculation.rs`
/// so that the event schema and the computation logic can evolve independently.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CalculationExecutedEventV1 {
    /// Input key associated with the calculation (e.g. outage_id).
    pub input_key: Symbol,
    /// Input value for the calculation.
    pub input_value: i128,
    /// Computed result value.
    pub result_value: i128,
    /// Ledger timestamp at calculation time.
    pub timestamp: u64,
}

/// A stateless event publisher that owns no state and only emits events.
///
/// # Boundary between business logic and side effects
///
/// `EventPublisher` is intentionally decoupled from computation functions
/// like [`crate::calculation::compute_result`]. The business logic returns
/// a pure result, and callers (e.g. `calculate_sla`) decide whether and
/// when to publish events. This separation:
///
/// - Makes business logic testable without needing an event assertion harness.
/// - Keeps event schemas versioned independently from computation rules.
/// - Allows the same computation to be used in view-only (`calculate_sla_view`)
///   and mutating (`calculate_sla`) paths without conditional event logic.
///
/// Events are always published through this struct to ensure consistent
/// topic layout and payload formatting.
pub struct EventPublisher;

impl EventPublisher {
    /// Publishes a calculation execution event with strict field ordering.
    ///
    /// Topic layout: `(topic, input_key)` for efficient filtering.
    /// Payload: `CalculationExecutedEventV1` with all fields in canonical order.
    pub fn publish_calculation_executed(
        env: &Env,
        topic: Symbol,
        input_key: Symbol,
        input_value: i128,
        result_value: i128,
        timestamp: u64,
    ) {
        let payload = CalculationExecutedEventV1 {
            input_key,
            input_value,
            result_value,
            timestamp,
        };

        env.events().publish((topic, input_key), payload);
    }
}
