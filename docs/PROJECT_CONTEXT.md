# ApexChainx System — Project Context

> **Purpose:** This document describes the high-level system architecture, repository landscape,
> and future contract roadmap for the ApexChainx platform.

## Table of Contents

- [Repository Architecture](#repository-architecture)
- [System Flow](#system-flow)
- [Architectural Rules](#architectural-rules)
- [Contract API Archetypes](#contract-api-archetypes)
- [Contract Lifecycle](#contract-lifecycle)- [SC-100: Future Contract Roadmap](#sc-100-future-contract-roadmap)
- [Pruning benchmark note](PRUNING_BENCHMARK_NOTE.md)

---

## Repository Architecture

The ApexChainx platform is composed of three repositories:

| Repository | Role | Technology |
|------------|------|------------|
| `apexchainx-fe` | Frontend application | React / TypeScript |
| `apexchainx-be` | Backend API and integration layer | Python / FastAPI |
| `apexchainx-contracts` | Soroban smart contracts (this repo) | Rust / Soroban SDK |

## System Flow

```
 User
  |
  v
┌─────────┐     ┌─────────┐     ┌──────────────┐
│   FE    │ ──→ │   BE    │ ──→ │  Contracts   │
│ (React) │ ←── │ (API)   │ ←── │  (Soroban)   │
└─────────┘     └─────────┘     └──────────────┘
```

## Architectural Rules

1. **Frontend never calls contracts directly** — all contract interactions go through the backend
2. **Backend is the exclusive bridge** — translates contract data to frontend-friendly responses
3. **Contracts are execution-layer only** — pure deterministic computation, no external dependencies

---

## Contract API Archetypes

Every public entrypoint in `apexchainx_calculator` falls into one of three
archetypes. This classification is the fastest way for contributors and
integrators to decide whether a call is safe to make freely, requires the
operator role, or is gated behind admin authority.

### Read-only (no auth required, no state written)

Safe to call by anyone at any time, including while the contract is paused.
No on-chain state is written and no events are emitted.

| Group | Methods |
|-------|---------|
| Healthcheck & version | `healthcheck`, `get_version_info`, `get_migration_state` |
| Pause / freeze status | `is_paused`, `get_pause_info`, `is_config_frozen` |
| Config views | `get_config`, `get_config_snapshot`, `get_config_version_hash`, `list_configs`, `get_last_config_update`, `get_config_bundle` |
| Custom severity views | `get_custom_severity`, `get_custom_config_snapshot` |
| Stats & telemetry | `get_stats`, `get_economic_exposure`, `get_severity_telemetry` |
| History views | `get_history`, `get_history_page`, `get_history_by_outage`, `get_latest_by_outage` |
| Role queries | `get_admin`, `get_operator`, `get_pending_admin`, `get_pending_operator` |
| Introspection | `get_result_schema`, `get_failure_schema`, `get_contract_metadata`, `get_full_audit_state` |
| Retention helpers | `get_retention_limit`, `get_config_count`, `get_storage_version` |
| View-mode calculation | `calculate_sla_view`, `replay_calculate_sla` |

### Mutating — operator role

Writes state and emits events. Only the current **operator** address may call
these.

| Method | What it writes |
|--------|----------------|
| `calculate_sla` | Appends a result to history, updates cumulative stats and per-severity telemetry, emits `sla_calc` and `set_int` events. Idempotent on exact replay; rejects conflicting duplicates. |

### Privileged — admin role

Only the current **admin** address may call these. They control lifecycle,
configuration, and role management. Some are irreversible (`renounce_admin`)
or have broad blast radius (`prune_history`).

| Group | Methods |
|-------|---------|
| Lifecycle | `initialize`, `migrate` |
| Config management | `set_config`, `set_custom_severity`, `remove_custom_severity`, `freeze_config`, `unfreeze_config` |
| Operational controls | `pause`, `unpause`, `set_retention_limit` |
| Admin transfer | `propose_admin`, `accept_admin`, `cancel_admin_proposal`, `renounce_admin` |
| Operator transfer | `set_operator` *(legacy direct)*, `propose_operator`, `accept_operator`, `cancel_operator_proposal` |
| History pruning | `prune_history`, `prune_history_by_age` |

> **Quick rule of thumb:** methods whose names start with `get_`, `is_`,
> `list_`, `healthcheck`, `calculate_sla_view`, or `replay_calculate_sla` are
> read-only and safe to call freely. `calculate_sla` requires the operator
> role. Everything else requires the admin role.
## Telemetry & Weekly Reset Semantics

The `apexchainx_calculator` contract maintains per-severity telemetry (`SeverityTelemetry`) tracking calculation counts, violation counts, and violation rates.

### Operator Posture & Reset Behavior

1. **Lazy On-Execution Evaluation**:
   - The contract checks the timestamp of the last calculation/violation for the invoked severity lane when `calculate_sla` is called.
   - If $\ge 7$ days ($604,800$ seconds) have passed since the recorded timestamp for that severity lane, the calculation and violation counters for that specific lane are reset to `0` before processing the current calculation.

2. **Per-Severity Isolation**:
   - Resets are evaluated per severity lane (`critical`, `high`, `medium`, `low`).
   - Activity in one lane does not reset or refresh timestamps for other lanes.

3. **Impact on Backend Consumers and Monitoring Dashboards**:
   - `get_severity_telemetry()` reflects stored counters. Inactive lanes retain their last updated telemetry state until the next invocation in that lane triggers a lazy reset.
   - Replays and duplicate resubmissions with identical inputs/configs do NOT update or reset telemetry counters.
   - Off-chain monitoring systems or backend consumers desiring continuous 7-day rolling window analytics should aggregate on-chain `EVENT_SLA_CALC` events or poll `get_severity_telemetry()` periodically alongside contract calls.
## Contract Lifecycle

The `apexchainx_calculator` contract has four independent state axes
(initialized, version-matched, paused, config-frozen) that combine to determine
which operations are permitted at any moment.

**→ See the full state-transition diagram: [docs/CONTRACT_LIFECYCLE.md](CONTRACT_LIFECYCLE.md)**

Quick overview of the main lifecycle states:

```
[Uninitialized] ──initialize()──→ [Active]
    [Active] ──pause()──→ [Paused] ──unpause()──→ [Active]
    [Active] ──(binary upgrade)──→ [NeedsMigration] ──migrate()──→ [Active]
    [Active] ──freeze_config()──→ [ConfigFrozen] ──unfreeze_config()──→ [Active]
    [Active] ──renounce_admin()──→ [AdminRenounced]  ← irreversible
```

See [`CONTRACT_LIFECYCLE.md`](CONTRACT_LIFECYCLE.md) for Mermaid diagrams of
each flow, the combined state matrix, and the full invariants table.
---

## SC-100: Future Contract Roadmap

This section documents the planned evolution of `apexchainx-contracts` based on
current backend integration needs and business requirements.

### Versioning Strategy

| Version | Scope | Timeline |
|---------|-------|----------|
| v1.0 | Single crate (`apexchainx_calculator`) | ✅ Current |
| v1.1 | Multi-contract version negotiation | ✅ Current |
| v2.0 | Payment escrow integration | Planned |
| v2.1 | Multi-party settlement | Planned |
| v3.0 | On-chain governance with timelocks | Planned |

### Current State

Only one contract crate exists in this repository:

| Crate | Status | Description | Key Features |
|-------|--------|-------------|--------------|
| `apexchainx_calculator` | **Production-ready** | SLA calculator contract | Config management, role-based auth, event emission, version negotiation, result schema |

### Planned Additions

The following crates are planned but **not yet implemented**. Do not import or
reference them until they appear in the repository.

| Crate | Status | Depends On | Description |
|-------|--------|------------|-------------|
| `payment_escrow` | Planned | `apexchainx_calculator` | Locks and conditionally releases Stellar token payments based on SLA results |
| `settlement` | Planned | `payment_escrow` | Splits shared outage costs between multiple parties |
| `governance` | Planned | — | On-chain admin config changes with time-locked execution |

### Event Ordering Guarantees

For the documented event ordering contract that backend consumers can rely on
for correct event processing, see **[Event Ordering Guarantees](./event-ordering-guarantees.md)**.
This document describes the deterministic event sequencing within a single
ledger, backed by the test suite in `apexchainx_calculator/src/event_ordering_tests.rs`.

### Integration Expectations

- The backend (`apexchainx-be`) currently integrates only with `apexchainx_calculator`
- New crates will be introduced incrementally
- Each new crate must expose a `get_result_schema()` equivalent for safe version pinning
- Frontend never calls contracts directly — all invocations go through the backend
- Backend indexers and operators should follow the [Observability Contract](OBSERVABILITY_CONTRACT.md) for health-signal monitoring and alerting guidance

### Upgrade & Migration

Storage-version upgrades follow the [Upgrade Playbook](UPGRADE_PLAYBOOK.md), which documents the complete preflight → migration → verification → rollback workflow for operators.

### API Stability

All public contract entrypoints are classified by compatibility risk in the **[API Stability Scorecard](API_STABILITY_SCORECARD.md)**. Contributors must consult this scorecard before modifying any public function signature to determine whether the change is additive or breaking.

### Contribution Guidelines for New Crates

1. **Open a tracking issue** before creating the crate directory
2. **Follow the established layout**: `src/lib.rs`, `src/tests.rs`, `Cargo.toml`
3. **Add to CI matrix** in `.github/workflows/`
4. **Export a result schema** function so the backend can detect breaking changes
5. **Include version negotiation** support for multi-contract compatibility
6. **Update the upgrade playbook** if the new crate introduces its own storage-version migration path
