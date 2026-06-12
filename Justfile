# Justfile for sofa-reader
# https://github.com/casey/just

set shell := ["bash", "-c"]

# Show available recipes
default:
    @just --list

# Run all checks: format check, clippy, tests, and doc
all: check fmt-check clippy test doc

# Build the crate
check:
    cargo check

# Build in release mode
build:
    cargo build --release

# Run the test suite (lib + integration)
test:
    cargo test --lib
    cargo test --test property_tests

# Run tests with all features
test-all:
    cargo test --all-features

# Run clippy (warnings reported but do not fail the build)
clippy:
    cargo clippy --all-targets --all-features

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
