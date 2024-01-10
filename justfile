TARGET := `rustc -vV | sed -n 's|host: ||p'`
DEP_OCCT_ROOT := `pwd` / "occt_prebuilt" / TARGET / "out"

run *FLAGS: build-occt
    DEP_OCCT_ROOT={{DEP_OCCT_ROOT}} cargo run {{ FLAGS }}

build *FLAGS: build-occt
    DEP_OCCT_ROOT={{DEP_OCCT_ROOT}} cargo build {{ FLAGS }}

check: build-occt
    cargo +nightly fmt --check
    DEP_OCCT_ROOT={{DEP_OCCT_ROOT}} cargo clippy --target {{ TARGET }} --all-targets -- -Dwarnings
    DEP_OCCT_ROOT={{DEP_OCCT_ROOT}} cargo test --target {{ TARGET }}

install: build-occt
    DEP_OCCT_ROOT={{DEP_OCCT_ROOT}} cargo install --path crates/dslcad

clean:
    cargo clean

build-occt:
    #!/bin/sh
    if [ ! -d "occt_prebuilt/{{ TARGET }}" ]; then
      cargo build --manifest-path tools/opencascade_builder/Cargo.toml --target {{ TARGET }} -vv
    fi
