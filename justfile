default_target := `rustc -vV | sed -n 's|host: ||p'`
wd := `pwd`

run *FLAGS: (run-dslcad default_target FLAGS)

build *FLAGS: (build-dslcad default_target FLAGS)

check:
    DEP_OCCT_ROOT="{{wd}}/occt_prebuilt/{{default_target}}/out" cargo +nightly fmt --check
    DEP_OCCT_ROOT="{{wd}}/occt_prebuilt/{{default_target}}/out" cargo clippy --all-targets -- -Dwarnings
    DEP_OCCT_ROOT="{{wd}}/occt_prebuilt/{{default_target}}/out" cargo test

clean:
    cargo clean

install *FLAGS: (build-occt default_target)
    DEP_OCCT_ROOT="{{wd}}/occt_prebuilt/{{default_target}}/out" cargo install --path crates/dslcad

run-dslcad TARGET FLAGS: (build-occt TARGET)
    DEP_OCCT_ROOT="{{wd}}/occt_prebuilt/{{TARGET}}/out" cargo run --target {{TARGET}} {{FLAGS}}

build-dslcad TARGET FLAGS: (build-occt TARGET)
    DEP_OCCT_ROOT="{{wd}}/occt_prebuilt/{{TARGET}}/out" cargo build --target {{TARGET}} {{FLAGS}}

build-occt TARGET:
    #!/bin/sh
    if [ ! -d "occt_prebuilt/{{TARGET}}" ]; then
      cargo build --manifest-path tools/opencascade_builder/Cargo.toml --target {{TARGET}} -vv
    fi
