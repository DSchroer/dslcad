use std::{env, path::PathBuf, thread, time::Duration};

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

/// Locate the OpenCASCADE headers.
///
/// OpenCASCADE is built by `opencascade-sys`. Cargo does not order the build
/// scripts of normal dependencies before ours, so `opencascade-sys` is also a
/// build dependency, which forces its (host) build to complete first. For
/// native builds that is the same OCCT this target links against; for
/// cross-compilation the target build may still be running, so fall back to the
/// host headers, which are identical across targets. The CMake build tree
/// (`build/include`) is also considered because it is populated before the
/// installed `include` directory.
fn occt_include() -> PathBuf {
    let candidates = include_candidates();

    // A concurrent OCCT build can briefly leave a candidate incomplete, so
    // wait for one of them to settle instead of failing the build.
    for _ in 0..150 {
        for candidate in &candidates {
            if candidate.join("Bnd_Box.hxx").exists() {
                return candidate.clone();
            }
        }
        thread::sleep(Duration::from_secs(2));
    }

    candidates.into_iter().next().unwrap()
}

fn include_candidates() -> Vec<PathBuf> {
    let occt = occt_sys::occt_path();
    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("missing OUT_DIR"));
    let host = out_dir.join("../../../../../OCCT");

    let mut candidates = Vec::new();
    for root in [&occt, &host] {
        // Prefer the CMake build tree headers: they exist as soon as OCCT is
        // configured and are not rewritten by the install step, which can race
        // with this build script.
        for dir in ["build/include", "build/inc", "include"] {
            candidates.push(root.join(dir));
        }
    }
    candidates
}
