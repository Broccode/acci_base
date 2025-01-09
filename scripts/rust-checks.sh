#!/bin/bash
set -e

# This file is used to run the pre-commit checks for the Rust codebase.
# Copy it to .git/hooks/pre-commit, make it executable and it will be run 
# automatically on commit.

echo "Running pre-commit checks..."

# # Run hardcoded strings check
# echo "→ Checking for hardcoded strings..."
# ./scripts/check-hardcoded-strings.sh

# Run cargo fmt
echo "→ Running cargo fmt..."
cargo fmt --all --verbose

# Run cargo check
echo "→ Running cargo check..."
cargo check --all-features

# Run cargo test
echo "→ Running cargo test..."
cargo test --all-features

# Run cargo clippy
echo "→ Running cargo clippy..."
cargo clippy --all-features -- -D warnings

# If we got here, all checks passed
echo "✓ All pre-commit checks passed!"
exit 0
