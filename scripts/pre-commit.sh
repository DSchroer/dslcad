#!/usr/bin/env bash
set -ex
cargo +nightly fmt --check
cargo clippy --all-targets -- -Dwarnings
cargo test
