#!/usr/bin/env bash
set -ex

./scripts/cheatsheet.sh > cheatsheet.md
(cd examples && zip -r ../examples.zip *)

cargo build --release --target x86_64-unknown-linux-gnu
(cd target/x86_64-unknown-linux-gnu/release/ && zip ../../../linux.zip model-script)

WIN_DLLS="/usr/lib/gcc/x86_64-w64-mingw32/10-posix"
cargo build --release --target x86_64-pc-windows-gnu
(cd target/x86_64-pc-windows-gnu/release/ && cp $WIN_DLLS/*.dll . && zip ../../../windows.zip model-script.exe *.dll)
