//! Shared history trim helper (Issue #655).
//!
//! Available to callers that already computed the retained entries and removal
//! count. It rebuilds canonical history and emits a versioned trim event.
//! Declaring this module does not reroute existing contract pruning entrypoints.

use crate::{SLAResult, EVENT_PRUNED, EVENT_PRUNED_AGE};
use soroban_sdk::{Address, Env, Vec};

/// The reason a trim was triggered — drives event selection.
pub enum TrimReason {
    /// Explicit admin call to `prune_history(keep_latest)`.
    AdminCount,
    /// Explicit admin call to `prune_history_by_age`.
    AdminAge,
    /// Automatic capacity enforcement during `calculate_sla` append.
    AutoCapacity,
}

/// Remove `entries` from the front of `history`, update `HISTORY_LEN_KEY`,
/// and emit the appropriate event. Returns the kept count.
///
/// This is the single implementation of "trim history" — all three call
/// sites must route through here so event emission and cache maintenance
/// are always consistent.
pub fn trim_history(
    env: &Env,
    new_history: Vec<SLAResult>,
    removed_count: u32,
    caller: &Address,
    reason: TrimReason,
) {
    if removed_count == 0 {
        return;
    }
    let kept = new_history.len();
    crate::history::rebuild_history(env, &new_history);

    match reason {
        TrimReason::AdminAge => {
            env.events().publish(
                (
                    EVENT_PRUNED_AGE,
                    crate::event_schema::event_version(EVENT_PRUNED_AGE),
                    caller.clone(),
                ),
                (removed_count, kept),
            );
        }
        TrimReason::AdminCount | TrimReason::AutoCapacity => {
            env.events().publish(
                (
                    EVENT_PRUNED,
                    crate::event_schema::event_version(EVENT_PRUNED),
                    caller.clone(),
                ),
                (removed_count, kept),
            );
        }
    }
}
