#!/usr/bin/env bash
set -ex
if [[ "$CARGO_ARGS" =~ "--release" ]]; then
  BUILD=release
else
  BUILD=debug
fi
echo Using mode $BUILD

cargo build --bin dslcad_wasm_server --target wasm32-unknown-emscripten $CARGO_ARGS
cargo build --bin dslcad --target wasm32-unknown-unknown $CARGO_ARGS

mkdir -p browser/lib
cp target/wasm32-unknown-emscripten/$BUILD/dslcad_wasm_server.js browser/lib
cp target/wasm32-unknown-emscripten/$BUILD/dslcad_wasm_server.wasm browser/lib

wasm-bindgen --out-dir browser/lib --target web ./target/wasm32-unknown-unknown/$BUILD/dslcad.wasm
# disable running main on startup
sed -i 's/wasm.__wbindgen_start();//g' browser/lib/dslcad.js

sed '/CODE/r browser/lib/dslcad_wasm_server.js' browser/dslcad_server.template.js > browser/lib/dslcad_server.js
