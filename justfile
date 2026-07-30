# =============================================================================
# ApexChainx Contracts — common developer commands (issue #113)
# =============================================================================
#
# One-liners for the sequences that CI runs, so a contributor can reproduce a
# CI failure locally without reading the workflow YAML. Every cargo recipe
# mirrors the matching step in .github/workflows/ci.yml (noted per recipe) and
# runs in the contract crate, which is CI's `working-directory`.
#
# Install just:  brew install just  |  cargo install just  |  https://just.systems
# Usage:         just <recipe>      |  `just` on its own lists every recipe.
# =============================================================================

# Contract crate — matches `working-directory: apexchainx_calculator` in CI.
crate := "apexchainx_calculator"
wasm_target := "wasm32-unknown-unknown"

# List available recipes.
default:
    @just --list

# ----------------------------------------------------------- bootstrap ------

# Bootstrap the dev environment — install toolchain, targets, and tools.
bootstrap:
    @echo "🔧 Bootstrapping ApexChainx development environment..."
    rustup target add wasm32-unknown-unknown
    @echo "✅ Bootstrap complete. Run 'just ci' to verify."

# ---------------------------------------------------------------- test ------

# Run the library test suite.            [CI: E2E Tests]
test:
    cd {{crate}} && cargo test --lib

# Run the property-based fuzz tests.     [CI: Fuzz Tests (proptest)]
fuzz:
    cd {{crate}} && cargo test --lib fuzz_tests::

# ---------------------------------------------------------------- lint ------

# Format the crate in place.
fmt:
    cd {{crate}} && cargo fmt

# Verify formatting without writing.     [CI: Format check]
fmt-check:
    cd {{crate}} && cargo fmt --check

# Clippy with warnings denied.           [CI: Clippy]
lint:
    cd {{crate}} && cargo clippy --all-targets -- -D warnings

# Type-check the crate.                  [CI: Cargo check]
check:
    cd {{crate}} && cargo check

# --------------------------------------------------------------- build ------

# Build natively.                        [CI: Build native]
build:
    cd {{crate}} && cargo build

# Build the WASM contract.               [CI: Build WASM]
wasm:
    cd {{crate}} && cargo build --target {{wasm_target}}

# Build the release WASM.                [CI: Provenance & Hashes]
wasm-release:
    cd {{crate}} && cargo build --target {{wasm_target}} --release

# Assert no_std compliance for wasm32.   [CI: WASM no-std compliance check]
no-std:
    cd {{crate}} && cargo check --target {{wasm_target}} --lib

# sha256 of the release WASM.            [CI: Generate hash]
hash: wasm-release
    #!/usr/bin/env bash
    set -euo pipefail
    # Workspace build — artifacts land in the ROOT target/, not {{crate}}/target/.
    wasm="target/{{wasm_target}}/release/{{crate}}.wasm"
    if [ ! -f "$wasm" ]; then
        echo "WASM file not found at $wasm" >&2
        exit 1
    fi
    # sha256sum on Linux/CI, shasum on macOS.
    if command -v sha256sum >/dev/null 2>&1; then
        sha256sum "$wasm" | awk '{print $1 "  {{crate}}.wasm"}'
    else
        shasum -a 256 "$wasm" | awk '{print $1 "  {{crate}}.wasm"}'
    fi

# Save release WASM hash to artifacts/.    [CI: Save hash for provenance]
hash-save: wasm-release
    #!/usr/bin/env bash
    set -euo pipefail
    wasm="target/{{wasm_target}}/release/{{crate}}.wasm"
    hash_file="artifacts/{{crate}}.wasm.sha256"
    mkdir -p artifacts
    if [ ! -f "$wasm" ]; then
        echo "WASM file not found at $wasm" >&2
        exit 1
    fi
    if command -v sha256sum >/dev/null 2>&1; then
        sha256sum "$wasm" | awk '{print $1 "  {{crate}}.wasm"}' > "$hash_file"
    else
        shasum -a 256 "$wasm" | awk '{print $1 "  {{crate}}.wasm"}' > "$hash_file"
    fi
    echo "Hash saved to $hash_file"
    cat "$hash_file"

# Verify release WASM hash against committed file. [CI: Verify hash provenance]
hash-verify: wasm-release
    #!/usr/bin/env bash
    set -euo pipefail
    wasm="target/{{wasm_target}}/release/{{crate}}.wasm"
    hash_file="artifacts/{{crate}}.wasm.sha256"
    if [ ! -f "$hash_file" ]; then
        echo "Hash file not found at $hash_file" >&2
        echo "Run 'just hash-save' to generate it." >&2
        exit 1
    fi
    if [ ! -f "$wasm" ]; then
        echo "WASM file not found at $wasm" >&2
        exit 1
    fi
    if command -v sha256sum >/dev/null 2>&1; then
        sha256sum -c "$hash_file"
    else
        shasum -a 256 -c "$hash_file"
    fi

# --------------------------------------------------------------- tooling -----

# Generate a ship-review note from CHANGELOG.md.
# Pass a version tag to summarise a released block: just release-summary 0.3.0
# Defaults to the [Unreleased] block when no argument is given.
release-summary version="Unreleased":
    npx --yes tsx tooling/releaseSummary.ts --version {{version}}

# ----------------------------------------------------------------- all ------

# Remove build artifacts.
clean:
    cd {{crate}} && cargo clean

# Everything CI gates on, in CI's order. Run before opening a PR.
ci: fmt-check lint check no-std test fuzz wasm
    @echo "✓ local CI equivalent passed"
