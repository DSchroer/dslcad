use std::{env, path::PathBuf};

fn main() {
    let include = occt_include();

    let mut build = cc::Build::new();
    build
        .cpp(true)
        .file("src/bend.cc")
        .file("src/bounds.cc")
        .include(&include)
        .std("c++17")
        .warnings(false);

    if env::var("TARGET").unwrap().contains("windows-gnu") {
        build.define("OCC_CONVERT_SIGNALS", "TRUE");
    }

    build.compile("dslcad_bend");

    println!("cargo:rerun-if-changed=src/bend.cc");
    println!("cargo:rerun-if-changed=src/bounds.cc");
}

/// Locate the installed OpenCASCADE headers.
///
/// OpenCASCADE is built by `opencascade-sys`. Cargo does not order the build
/// scripts of normal dependencies before ours, so `opencascade-sys` is also a
/// build dependency, which forces its (host) build to complete first. For
/// native builds that is the same OCCT this target links against; for
/// cross-compilation the target build may still be running, so fall back to the
/// host headers, which are identical across targets.
fn occt_include() -> PathBuf {
    let target = occt_sys::occt_path().join("include");
    if target.exists() {
        return target;
    }

    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("missing OUT_DIR"));
    let host = out_dir.join("../../../../../OCCT/include");
    if host.exists() {
        return host.canonicalize().unwrap_or(host);
    }

    target
}
