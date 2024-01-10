use std::fs;
use fs_extra::{copy_items, remove_items};
use fs_extra::dir::CopyOptions;
use occt_sys;

fn main() {
    std::env::set_var("REBUILD", format!("{:?}", std::time::Instant::now()));
    println!("cargo:rerun-if-env-changed=REBUILD");

    let built_occt= occt_sys::occt_path();
    let target = std::env::var("TARGET").unwrap();
    let out_path = format!("../../occt_prebuilt/{}/", target);

    let _ = remove_items(&[&out_path]);
    fs::create_dir_all(&out_path).expect("failed to make dir");

    copy_items(&[built_occt], &out_path, &CopyOptions::default()).expect("unable to copy OCCT");
}
