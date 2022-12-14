#!/usr/bin/env bash
set -ex

cargo run --bin cheat_sheet > cheatsheet.md
(cd examples && zip -r ../examples.zip *)

# Build Linux
(
  cargo build --release --target x86_64-unknown-linux-gnu
  (cd target/x86_64-unknown-linux-gnu/release/ && zip ../../../linux.zip dslcad)
)

# Build Windows
(
  WIN_DLLS="/usr/lib/gcc/x86_64-w64-mingw32/10-win32"
  cargo build --release --target x86_64-pc-windows-gnu
  (cd target/x86_64-pc-windows-gnu/release/ && cp $WIN_DLLS/*.dll . && zip ../../../windows.zip dslcad.exe *.dll)
)

# Build MacOSX
(
  PATH="/osxcross/target/bin:$PATH" \
  CC=x86_64-apple-darwin20.4-clang \
  CXX=x86_64-apple-darwin20.4-clang++ \
  cargo build --release --target x86_64-apple-darwin
  (cd target/x86_64-apple-darwin/release/ && zip ../../../macosx.zip dslcad)
)
