#!/usr/bin/env sh
set -eu
cargo fmt --all -- --check
python3 scripts/test_economic_vectors.py
cargo check --workspace --locked
cargo test --workspace --locked
