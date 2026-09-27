//! Reusable SLA event publication helpers (Issue #656).
//!
//! These helpers use the canonical per-event versions. Existing contract
//! entrypoints also publish directly; this module's declaration does not reroute
//! those entrypoints or change their payloads. `event_schema.rs` remains the
//! authoritative event catalog.

use soroban_sdk::{Address, Env, Symbol};

use crate::{
    SLAResult, EVENT_CONFIG_FREEZE, EVENT_CONFIG_REM, EVENT_CONFIG_UNFREEZE, EVENT_CONFIG_UPD,
    EVENT_DUP_INPUT, EVENT_PAUSED, EVENT_PRUNED, EVENT_PRUNED_AGE, EVENT_SETTLE_INTENT, EVENT_SEV_ADD,
    EVENT_SEV_UPD, EVENT_SLA_CALC, EVENT_UNPAUSED,
};

/// Stateless event publisher — wraps every `env.events().publish()` call
/// behind a named method so the event catalog is enumerable in one place.
pub struct EventPublisher<'a> {
    env: &'a Env,
}

impl<'a> EventPublisher<'a> {
    pub fn new(env: &'a Env) -> Self {
        Self { env }
    }

    /// Emit the primary SLA calculation event (`sla_calc`).
    pub fn sla_calc(&self, severity: Symbol, result: &SLAResult) {
        self.env.events().publish(
            (
                EVENT_SLA_CALC,
                crate::event_schema::event_version(EVENT_SLA_CALC),
                severity,
            ),
            (
                result.outage_id.clone(),
                result.status.clone(),
                result.mttr_minutes,
                result.threshold_minutes,
                result.amount,
                result.payment_type.clone(),
                result.rating.clone(),
                result.config_version_hash,
                result.recorded_at,
            ),
        );
    }

    /// Emit the settlement intent event (`set_int`).
    pub fn settlement_intent(&self, severity: Symbol, result: &SLAResult, correlation_id: u64) {
        self.env.events().publish(
            (
                EVENT_SETTLE_INTENT,
                crate::event_schema::event_version(EVENT_SETTLE_INTENT),
                severity,
            ),
            (
                result.outage_id.clone(),
                result.status.clone(),
                result.mttr_minutes,
                result.threshold_minutes,
                result.amount,
                result.payment_type.clone(),
                result.rating.clone(),
                result.config_version_hash,
                result.recorded_at,
                correlation_id,
            ),
        );
    }

    /// Emit the duplicate-input rejection event (`dup_input`).
    /// Includes the stored decision AND the rejected attempted inputs.
    pub fn duplicate_input(
        &self,
        severity: Symbol,
        existing: &SLAResult,
        attempted_mttr: u32,
        attempted_threshold: u32,
    ) {
        self.env.events().publish(
            (
                EVENT_DUP_INPUT,
                crate::event_schema::event_version(EVENT_DUP_INPUT),
                severity,
            ),
            (
                existing.outage_id.clone(),
                existing.status.clone(),
                existing.mttr_minutes,
                existing.threshold_minutes,
                existing.amount,
                existing.payment_type.clone(),
                existing.rating.clone(),
                existing.config_version_hash,
                existing.recorded_at,
                attempted_mttr,
                attempted_threshold,
            ),
        );
    }

    /// Emit a config update event (`cfg_upd`).
    pub fn config_updated(
        &self,
        severity: Symbol,
        threshold_minutes: u32,
        penalty_per_minute: i128,
        reward_base: i128,
    ) {
        self.env.events().publish(
            (
                EVENT_CONFIG_UPD,
                crate::event_schema::event_version(EVENT_CONFIG_UPD),
                severity,
            ),
            (threshold_minutes, penalty_per_minute, reward_base),
        );
    }

    /// Emit a new custom severity added event (`sev_add`).
    pub fn severity_added(
        &self,
        severity: Symbol,
        threshold_minutes: u32,
        penalty_per_minute: i128,
        reward_base: i128,
    ) {
        self.env.events().publish(
            (
                EVENT_SEV_ADD,
                crate::event_schema::event_version(EVENT_SEV_ADD),
                severity,
            ),
            (threshold_minutes, penalty_per_minute, reward_base),
        );
    }

    /// Emit a custom severity updated event (`sev_upd`).
    pub fn severity_updated(
        &self,
        severity: Symbol,
        threshold_minutes: u32,
        penalty_per_minute: i128,
        reward_base: i128,
    ) {
        self.env.events().publish(
            (
                EVENT_SEV_UPD,
                crate::event_schema::event_version(EVENT_SEV_UPD),
                severity,
            ),
            (threshold_minutes, penalty_per_minute, reward_base),
        );
    }

    /// Emit a custom severity removed event (`cfg_rem`).
    pub fn severity_removed(&self, severity: Symbol) {
        self.env.events().publish(
            (
                EVENT_CONFIG_REM,
                crate::event_schema::event_version(EVENT_CONFIG_REM),
                severity,
            ),
            (),
        );
    }

    /// Emit the contract paused event (`paused`).
    pub fn paused(&self, caller: Address) {
        self.env.events().publish(
            (
                EVENT_PAUSED,
                crate::event_schema::event_version(EVENT_PAUSED),
                caller,
            ),
            (true,),
        );
    }

    /// Emit the contract unpaused event (`unpause`).
    pub fn unpaused(&self, caller: Address) {
        self.env.events().publish(
            (
                EVENT_UNPAUSED,
                crate::event_schema::event_version(EVENT_UNPAUSED),
                caller,
            ),
            (false,),
        );
    }

    /// Emit the config frozen event (`cfg_frz`).
    pub fn config_frozen(&self, caller: Address) {
        self.env.events().publish(
            (
                EVENT_CONFIG_FREEZE,
                crate::event_schema::event_version(EVENT_CONFIG_FREEZE),
                caller,
            ),
            (),
        );
    }

    /// Emit the config unfrozen event (`cfg_unfrz`).
    pub fn config_unfrozen(&self, caller: Address) {
        self.env.events().publish(
            (
                EVENT_CONFIG_UNFREEZE,
                crate::event_schema::event_version(EVENT_CONFIG_UNFREEZE),
                caller,
            ),
            (),
        );
    }

    /// Emit the pruned event (`pruned`).
    pub fn pruned(&self, caller: Address, removed_count: u32, kept_count: u32) {
        self.env.events().publish(
            (
                EVENT_PRUNED,
                crate::event_schema::event_version(EVENT_PRUNED),
                caller,
            ),
            (removed_count, kept_count),
        );
    }

    /// Emit the pruned-by-age event (`pruned_a`).
    pub fn pruned_by_age(&self, caller: Address, removed_count: u32, kept_count: u32) {
        self.env.events().publish(
            (
                EVENT_PRUNED_AGE,
                crate::event_schema::event_version(EVENT_PRUNED_AGE),
                caller,
            ),
            (removed_count, kept_count),
        );
    }
}
