use std::{
    env, fs,
    path::{Path, PathBuf},
};

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

/// Locate the installed OpenCASCADE headers, reinstalling OpenCASCADE if the
/// installation is incomplete.
///
/// OpenCASCADE is built and installed by `opencascade-sys` into the Cargo
/// target directory (`<target>/OCCT`). Caching actions such as
/// `Swatinem/rust-cache` only keep Cargo's own artifacts under `target`, so the
/// installed `include` and `lib` directories are removed while
/// `opencascade-sys` is still considered up-to-date, which used to make the
/// build fail with `Bnd_Box.hxx: No such file or directory`.
///
/// Cargo does not order the build scripts of normal dependencies before ours,
/// so `opencascade-sys` (and `occt-sys`) are also build dependencies, which
/// forces their builds to complete first. This lets us re-run the OpenCASCADE
/// install from the preserved CMake build tree whenever the headers or
/// libraries are missing.
fn occt_include() -> PathBuf {
    let occt = occt_sys::occt_path();
    let include = occt.join("include");

    if is_occt_installed(&occt) {
        return include;
    }

    occt_sys::build_occt();

    if is_occt_installed(&occt) {
        return include;
    }

    // Cross-compilation may still be building the target OpenCASCADE. The host
    // headers are identical across targets, so fall back to them.
    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("missing OUT_DIR"));
    let host = out_dir.join("../../../../../OCCT/include");
    if host.join("Bnd_Box.hxx").exists() {
        return host.canonicalize().unwrap_or(host);
    }

    include
}

/// Whether the OpenCASCADE installation contains both the headers and the
/// libraries the linker needs.
fn is_occt_installed(occt: &Path) -> bool {
    if !occt.join("include").join("Bnd_Box.hxx").exists() {
        return false;
    }

    match fs::read_dir(occt.join("lib")) {
        Ok(mut entries) => entries.next().is_some(),
        Err(_) => false,
    }
}
