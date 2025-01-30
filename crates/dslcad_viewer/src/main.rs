use dslcad_storage::protocol::Render;
use dslcad_viewer::{Preview, PreviewHandle};
use std::error::Error;
use std::sync::OnceLock;
use std::{env, fs};

static HANDLE: OnceLock<PreviewHandle> = OnceLock::new();

#[no_mangle]
pub extern "C" fn allocate(len: usize) -> *mut u8 {
    let vec = Vec::with_capacity(len);
    let ptr = vec.leak();
    ptr as *const [u8] as *mut u8
}

#[no_mangle]
pub extern "C" fn show_rendering() {
    HANDLE.get().unwrap().show_rendering();
}

#[no_mangle]
/// # Safety
/// This is safe to call if ptr and len are from a Rust vec that has been forgotten.
/// Make sure that the original vec has been shrink_to_fit as len and cap are assumed equal.
pub unsafe extern "C" fn show_error(ptr: *mut u8, len: usize) {
    let data = Vec::from_raw_parts(ptr, len, len);
    let error = String::from_utf8(data).unwrap();
    HANDLE.get().unwrap().show_error(error);
}

#[no_mangle]
/// # Safety
/// This is safe to call if ptr and len are from a Rust vec that has been forgotten.
/// Make sure that the original vec has been shrink_to_fit as len and cap are assumed equal.
pub unsafe extern "C" fn show_render(ptr: *mut u8, len: usize) {
    let model = Vec::from_raw_parts(ptr, len, len);
    let render: Render = model.as_slice().try_into().unwrap();
    HANDLE.get().unwrap().show_render(render);
}

pub fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().skip(1).collect();

    let (preview, handle) = Preview::new();
    assert!(HANDLE.set(handle).is_ok());

    if !args.is_empty() {
        let mut model = fs::read(&args[0])?;
        model.shrink_to_fit();
        let len = model.len();
        let ptr = model.leak();

        // SAFETY: safe to call since we built the vec above
        unsafe {
            show_render(ptr as *const [u8] as *mut u8, len);
        }
    }

    preview.open(String::new());

    Ok(())
}
