use std::env;

fn main() {
    let occt = occt_sys::occt_path();
    let include = occt.join("include");

    let mut build = cc::Build::new();
    build
        .cpp(true)
        .file("src/bend.cc")
        .include(&include)
        .std("c++17")
        .warnings(false);

    if env::var("TARGET").unwrap().contains("windows-gnu") {
        build.define("OCC_CONVERT_SIGNALS", "TRUE");
    }

    build.compile("dslcad_bend");

    println!("cargo:rerun-if-changed=src/bend.cc");
}
