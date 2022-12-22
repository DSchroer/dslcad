#!/usr/bin/env bash
set -ex
cargo +nightly fmt --check
cargo clippy
cargo test
