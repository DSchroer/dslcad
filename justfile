TARGET := `rustc -vV | sed -n 's|host: ||p'`
export CMAKE_BUILD_PARALLEL_LEVEL := `nproc --all`

run *FLAGS:
    cargo run --target {{ TARGET }} {{ FLAGS }}

build *FLAGS:
    #!/usr/bin/env bash
    set -ex

    if [ "{{ TARGET }}" == "wasm32-unknown-emscripten" ]; then
      FLAGS="--no-default-features";
    elif [ "{{ TARGET }}" == "wasm32-unknown-unknown" ]; then
        exit 0
    fi

    cargo build --bin dslcad --target {{ TARGET }} $FLAGS {{ FLAGS }}

build-examples:
    cd examples && zip -r ../examples.zip *

build-vscode-extension:
    cd extensions/vscode && vsce package
    mv extensions/vscode/*.vsix .

build-viewer *FLAGS:
    #!/usr/bin/env bash
    set -ex

    if [ "{{ TARGET }}" == "wasm32-unknown-emscripten" ]; then
        exit 0
    fi

    cargo build --bin dslcad-viewer --target {{ TARGET }} --release {{ FLAGS }}

    if [ "{{ TARGET }}" == "wasm32-unknown-unknown" ]; then
      wasm-bindgen --out-dir ./target/wasm32-unknown-unknown/release --target web ./target/wasm32-unknown-unknown/release/dslcad-viewer.wasm
      sed -i 's/wasm.__wbindgen_start();//' ./target/wasm32-unknown-unknown/release/dslcad-viewer.js
    fi

build-docs-editor *FLAGS:
    just TARGET=wasm32-unknown-emscripten build --release {{ FLAGS }}
    just TARGET=wasm32-unknown-unknown build-viewer {{ FLAGS }}

    mkdir -p docs/editor
    cp target/wasm32-unknown-emscripten/release/dslcad* ./docs/editor/
    cp target/wasm32-unknown-unknown/release/dslcad-viewer* ./docs/editor/

pack: (build "--release") build-viewer
    -rm {{ TARGET }}.zip
    rm target/{{ TARGET }}/release/*.d
    zip -j {{ TARGET }}.zip target/{{ TARGET }}/release/*

check:
    cargo +nightly fmt --check
    cargo clippy --target {{ TARGET }} --all-targets -- -Dwarnings
    cargo test --target {{ TARGET }}

install:
    cargo install --path crates/dslcad

clean:
    cargo clean
