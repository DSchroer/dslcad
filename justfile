TARGET := `rustc -vV | sed -n 's|host: ||p'`
DEP_OCCT_ROOT := `pwd` / "occt_prebuilt" / TARGET / "out"

run *FLAGS: build-occt
    cargo run {{ FLAGS }}

build *FLAGS: build-occt
    cargo build {{ FLAGS }}

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
    if [ ! -d "occt_prebuilt/{{ TARGET }}" ]; then
      cargo build --manifest-path tools/opencascade_builder/Cargo.toml --target {{ TARGET }} -vv
    fi

setup-env:
    mkdir -p .cargo
    echo -e "[env]\nDEP_OCCT_ROOT = \"{{DEP_OCCT_ROOT}}\"" > .cargo/config
