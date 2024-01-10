TARGET := `rustc -vV | sed -n 's|host: ||p'`
cwd := `pwd`
export DEP_OCCT_ROOT := cwd / "occt_prebuilt" / TARGET / "out"

run *FLAGS: build-occt
    cargo run {{FLAGS}}

build *FLAGS: build-occt
    cargo build {{FLAGS}}

check:
    cargo +nightly fmt --check
    cargo clippy --all-targets -- -Dwarnings
    cargo test

clean:
    cargo clean

install: build-occt
    cargo install --path crates/dslcad

build-occt:
    #!/bin/sh
    if [ ! -d "occt_prebuilt/{{TARGET}}" ]; then
      cargo build --manifest-path tools/opencascade_builder/Cargo.toml --target {{TARGET}} -vv
    fi
