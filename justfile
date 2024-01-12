TARGET := `rustc -vV | sed -n 's|host: ||p'`
export DEP_OCCT_ROOT := `pwd` / "occt_prebuilt" / TARGET / "out"

CORES := `nproc --all`

run *FLAGS: build-occt
    DEP_OCCT_ROOT={{DEP_OCCT_ROOT}} cargo run --target {{ TARGET }} {{ FLAGS }}

build *FLAGS: build-occt
    DEP_OCCT_ROOT={{DEP_OCCT_ROOT}} cargo build --bin dslcad --target {{ TARGET }} {{ FLAGS }}

build-preview *FLAGS:
    cargo build --bin preview --target {{ TARGET }} {{ FLAGS }}

build-wasm *FLAGS:
    just TARGET=wasm32-unknown-emscripten build --no-default-features {{ FLAGS }}
    just TARGET=wasm32-unknown-unknown build-preview {{ FLAGS }}

check: build-occt
    cargo +nightly fmt --check
    DEP_OCCT_ROOT={{DEP_OCCT_ROOT}} cargo clippy --target {{ TARGET }} --all-targets -- -Dwarnings
    DEP_OCCT_ROOT={{DEP_OCCT_ROOT}} cargo test --target {{ TARGET }}

install: build-occt
    DEP_OCCT_ROOT={{DEP_OCCT_ROOT}} cargo install --path crates/dslcad

clean:
    cargo clean

build-occt: setup-env
    #!/bin/bash
    set -ex
    if [ ! -d "occt_prebuilt/{{ TARGET }}" ]; then
      export CMAKE_BUILD_PARALLEL_LEVEL={{CORES}}
      cargo clean --manifest-path tools/opencascade_builder/Cargo.toml
      cargo build --manifest-path tools/opencascade_builder/Cargo.toml --release --target {{ TARGET }} -vv

      # Copy built files
      mkdir -p occt_prebuilt/{{ TARGET }}
      cp -r tools/opencascade_builder/target/{{ TARGET }}/release/build/occt-sys-*/out occt_prebuilt/{{ TARGET }}
    fi

setup-env:
    #!/bin/bash
    mkdir -p .cargo
    echo -e "[env]\nDEP_OCCT_ROOT = \"{{DEP_OCCT_ROOT}}\"" > .cargo/config
