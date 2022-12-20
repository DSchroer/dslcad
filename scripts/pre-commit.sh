#!/usr/bin/env bash
set -e
cargo test
cargo +nightly fmt --check
cargo clippy
