TARGET := `rustc -vV | sed -n 's|host: ||p'`
export DEP_OCCT_ROOT := `pwd` / "occt_prebuilt" / TARGET / "out"

CORES := `nproc --all`

run *FLAGS: build-occt
    cargo run --target {{ TARGET }} {{ FLAGS }}

build *FLAGS: build-occt
    cargo build --bin dslcad --target {{ TARGET }} {{ FLAGS }}

build-preview *FLAGS:
    cargo build --bin preview --target {{ TARGET }} {{ FLAGS }}

build-wasm *FLAGS:
    just TARGET=wasm32-unknown-emscripten build --no-default-features {{ FLAGS }}
    just TARGET=wasm32-unknown-unknown build-preview {{ FLAGS }}

check: build-occt
    cargo +nightly fmt --check
    cargo clippy --target {{ TARGET }} --all-targets -- -Dwarnings
    cargo test --target {{ TARGET }}

install: build-occt
    cargo install --path crates/dslcad

clean:
    cargo clean

build-occt: setup-env
    #!/bin/sh
    set -ex
    if [ ! -d "occt_prebuilt/{{ TARGET }}" ]; then
      export CMAKE_BUILD_PARALLEL_LEVEL={{CORES}}
      cargo clean --manifest-path tools/opencascade_builder/Cargo.toml
      cargo build --manifest-path tools/opencascade_builder/Cargo.toml --target {{ TARGET }} -vv
    fi

setup-env:
    #!/bin/bash
    mkdir -p .cargo
    echo -e "[env]\nDEP_OCCT_ROOT = \"{{DEP_OCCT_ROOT}}\"" > .cargo/config
