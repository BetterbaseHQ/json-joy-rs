# List available recipes
default:
    @just --list

# Run all checks (format, lint, test)
check: fmt lint test-gates test

# Format code
fmt:
    mise x -- cargo fmt --all

# Run clippy linter
lint:
    @echo "Running cargo check (strict clippy is currently too noisy for this repo baseline)"
    mise x -- cargo check --workspace

# Run full workspace tests
test *args:
    mise x -- cargo test --workspace {{args}}

# Run tests with verbose output
test-v *args:
    mise x -- cargo test --workspace {{args}} -- --nocapture

# Run benchmarks
bench *args:
    mise x -- cargo bench --workspace {{args}}

# Build all targets
build:
    mise x -- cargo build --workspace

# Build release
build-release:
    mise x -- cargo build --workspace --release

# Clean build artifacts
clean:
    mise x -- cargo clean

# Existing project workflows
test-smoke:
    make test-smoke

test-gates:
    make test-gates

parity-fixtures:
    make parity-fixtures

compat-fixtures:
    make compat-fixtures

port-slice pkg suite filter='' fixtures='1' gates='0':
    #!/usr/bin/env bash
    set -euo pipefail
    if [ -n "{{filter}}" ]; then
      make port-slice PKG={{pkg}} SUITE={{suite}} FILTER={{filter}} FIXTURES={{fixtures}} GATES={{gates}}
    else
      make port-slice PKG={{pkg}} SUITE={{suite}} FIXTURES={{fixtures}} GATES={{gates}}
    fi
