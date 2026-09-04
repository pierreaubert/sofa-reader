# Justfile for sofa-reader
# https://github.com/casey/just

set shell := ["bash", "-c"]

# Show available recipes
default:
    @just --list

# Run all checks: format check, clippy, tests, and doc
all: check check-minimal fmt-check clippy test doc

# Build the crate
check:
    cargo check --all-targets

# Check with minimal features (sqlite/deflate paths must stay optional-safe)
check-minimal:
    cargo check --all-targets --no-default-features

# Build in release mode
build:
    cargo build --release

# Run the full test suite (lib + integration + doctests)
test:
    cargo test --all-features

# Run tests without default features
test-minimal:
    cargo test --no-default-features --lib

# Run tests with all features
test-all:
    cargo test --all-features

# Run clippy (warnings fail the build, same as CI)
clippy:
    cargo clippy --all-targets --all-features -- -D warnings

# Format code
fmt:
    cargo fmt

# Check formatting without modifying files
fmt-check:
    cargo fmt -- --check

# Build documentation
doc:
    cargo doc --no-deps

# Clean build artifacts
clean:
    cargo clean
