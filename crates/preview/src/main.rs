use persistence::protocol::Render;
use preview::{Preview, PreviewHandle};
use std::error::Error;
use std::sync::OnceLock;
use std::{env, fs, slice};

static HANDLE: OnceLock<PreviewHandle> = OnceLock::new();

#[no_mangle]
pub extern "C" fn allocate(len: usize) -> *const u8 {
    let vec = Vec::with_capacity(len);
    let ptr = vec.leak();
    ptr as *const [u8] as *const u8
}

#[no_mangle]
/// # Safety
/// This is safe to call if ptr and len are from a Rust vec that has been forgotten.
/// Make sure that the original vec has been shrink_to_fit as len and cap are assumed equal.
pub unsafe extern "C" fn render_raw(ptr: *const u8, len: usize) {
    let model = slice::from_raw_parts(ptr, len);
    let render: Render = model.try_into().unwrap();
    HANDLE.get().unwrap().show_render(render).unwrap();
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
            render_raw(ptr as *const [u8] as *const u8, len);
        }
    }

    preview.open(String::new());

    Ok(())
}
