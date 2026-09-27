# Issue 674 validation

This branch builds on PR #720 (issues #675, #677 and #678).

Acceptance evidence:
- Release checklist: `docs/RELEASE_PREFLIGHT.md` and generated `release-ticket.md`.
- Proof: per-gate logs and machine-readable report are retained even on failure.
- Waivers: exact candidate/run, reviewer, issue/PR, reason and expiry; parity/freshness/budgets cannot be waived.
- 156 TypeScript tests pass, including synthetic external-result cases through the real preflight orchestration. Synthetic results are gate tests, not nightly proof.
- Three Rust fixture tests pass. Event sizes are compared with actual Rust XDR serialization; all seven event budgets pass.
- Typecheck, read budget, governance, metadata, persistence freshness and format pass (attached logs).

## Live nightly finding

The latest upstream scheduled run inspected was [35830444403](https://github.com/ApexChainx/ApexChainx-Contracts/actions/runs/35830444403), from 2026-09-23. All five targets failed because their artifact directories were missing, before fuzzing began. The workflow now creates an absolute artifact directory and retains build and fuzz logs, with failures propagated through pipefail.

Branch push runs verify the repaired fuzzer without creating crash issues or committing corpus updates. They are not eligible scheduled-main evidence. Once merged, a fresh scheduled main run with all five retained evidence artifacts must pass, or an eligible failed/stale nightly must have a reviewed exact-candidate waiver. No waiver was fabricated for this change.
