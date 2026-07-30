<p align="center">
  <img src="https://img.shields.io/badge/status-active-success.svg" alt="Status: Active">
  <img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="License: MIT">
  <img src="https://img.shields.io/badge/version-0.1.0-blueviolet" alt="Version: 0.1.0">
  <img src="https://img.shields.io/badge/Soroban_SDK-21.0.0-important" alt="Soroban SDK: 21.0.0">
  <img src="https://img.shields.io/badge/rustc-stable-success" alt="Rust: stable">
  <img src="https://img.shields.io/badge/platform-Stellar_Network-000" alt="Platform: Stellar Network">
  <a href="https://codecov.io/gh/ApexChainx/ApexChainx-Contracts"><img src="https://codecov.io/gh/ApexChainx/ApexChainx-Contracts/branch/main/graph/badge.svg" alt="Coverage"></a>
</p>

# ApexChainx Smart Contracts

## Frequently Asked Questions

### What is ApexChainx?

ApexChainx is a smart contract platform built on the Stellar network for
deterministic SLA (Service Level Agreement) calculation, payment escrow,
and multi-party settlement.

### What blockchain does this use?

These contracts run on the **Stellar network** using the **Soroban** smart
contract platform.

### How is SLA calculated?

The contract takes severity level, measured MTTR (Mean Time To Repair), and
configured thresholds to determine whether SLA targets were met. Results include
status (met/violated), payment type (reward/penalty), and rating.

### Can I call contracts directly from the frontend?

**No.** All contract invocations must go through the backend API layer. The
frontend never interacts with contracts directly.

### How are contract upgrades handled?

The contract includes a version negotiation protocol (`get_version_info()`) that
allows backends to verify compatibility before deployment. Full upgrade procedures
— including preflight checks, migration execution, post-upgrade verification, and
rollback — are documented in the **[Upgrade Playbook](docs/UPGRADE_PLAYBOOK.md)**.

### What stops an operator from spamming the same outage ID?

`calculate_sla` is idempotent: resubmitting an outage with an unchanged config
hash and identical inputs returns the stored result and writes nothing — no
history entry, no statistics, no telemetry, no events — so retries are safe and
cannot skew reported violation rates. Resubmitting the *same* outage with
different inputs is rejected (`DuplicateOutageInput`), and a config change opens
a new stored generation for that outage, capped at 16 retained entries
(`OutageRecalcLimit`) so one outage cannot crowd others out of the retention
window. Admin pruning frees that headroom again.

### Is the contract upgradeable?

No. The contract is not natively upgradeable. Upgrades require deploying a new
contract and migrating state through the backend.

> **Soroban-based SLA calculator and multi-contract coordination suite for the Stellar network.**

This repository is the execution-layer side of the 3-repo architecture.

## Contract API Archetypes

Every public entrypoint in `apexchainx_calculator` belongs to one of three
archetypes. Knowing which category a method falls into tells you immediately
whether a call can be made freely, requires operator credentials, or is gated
behind admin authority.

### Read-only (no auth required, no state written)

These functions can be called by anyone at any time — including while the
contract is paused — and never modify on-chain state or emit events.

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

These functions write state and emit events. Only the current **operator**
address may call them.

| Method | What it writes |
|--------|----------------|
| `calculate_sla` | Appends a result to history, updates cumulative stats and per-severity telemetry, emits `sla_calc` and `set_int` events. Idempotent on exact replay; rejects conflicting duplicates. |

### Privileged — admin role

These functions can only be called by the current **admin** address. They
control the contract's lifecycle, configuration, and role management. Several
of them are irreversible (e.g., `renounce_admin`) or have broad blast radius
(e.g., `prune_history`), so treat them accordingly.

| Group | Methods |
|-------|---------|
| Lifecycle | `initialize`, `migrate` |
| Config management | `set_config`, `set_custom_severity`, `remove_custom_severity`, `freeze_config`, `unfreeze_config` |
| Operational controls | `pause`, `unpause`, `set_retention_limit` |
| Admin transfer | `propose_admin`, `accept_admin`, `cancel_admin_proposal`, `renounce_admin` |
| Operator transfer | `set_operator` *(legacy direct)*, `propose_operator`, `accept_operator`, `cancel_operator_proposal` |
| History pruning | `prune_history`, `prune_history_by_age` |

> **Quick rule of thumb for contributors:** if the method name starts with
> `get_`, `is_`, `list_`, `healthcheck`, `calculate_sla_view`, or
> `replay_calculate_sla` it is read-only and safe to call freely. Everything
> else either requires the operator role (`calculate_sla`) or the admin role
> (everything remaining).

## Related Repositories

| Repository | Description |
|------------|-------------|
| [apexchainx-fe](https://github.com/ApexChainx/apexchainx-fe) | Frontend application (React/TypeScript) |
| [apexchainx-be](https://github.com/ApexChainx/apexchainx-be) | Backend API and contract bridge |

## Development Setup

### Quick Start with Dev Container (#281)

A [dev container](.devcontainer/) is provided for GitHub Codespaces and VS Code:

```bash
# Open in Codespaces or VS Code — the devcontainer auto-configures:
# - Rust toolchain + wasm32-unknown-unknown target
# - just command runner
# - Node.js + npx for tooling scripts
```

### Local Setup

```bash
# Bootstrap the dev environment
just bootstrap

# Run the full CI pipeline locally
just ci
```

See [CONTRIBUTING.md](CONTRIBUTING.md) for detailed setup instructions.

## Security & Supply Chain

- **[WASM Binary Reproducibility Policy](docs/WASM_REPRODUCIBILITY_POLICY.md)** — Build input recording, artifact checksum provenance, and one-step maintainer verification for release WASM binaries.
- **[Release Artifact Provenance Policy](docs/RELEASE_PROVENANCE_POLICY.md)** — Guidelines for WASM output checksums and snapshot check-ins.
- **[Release Summary Format](docs/RELEASE_SUMMARY_FORMAT.md)** — Structured ship-review note format for maintainer release triage. Generate one with `just release-summary` (or `just release-summary <version>`).
- **Dependency auditing:** `cargo audit` runs on CI for every push
- **WASM integrity:** Release artifacts include SHA-256 manifests
- **Reproducible builds:** Local builds can be verified against CI-generated manifests
