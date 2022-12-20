#!/usr/bin/env bash
set -e
cargo +nightly fmt --check
cargo clippy
cargo test
