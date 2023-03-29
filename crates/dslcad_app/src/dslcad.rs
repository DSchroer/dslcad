
#[cfg(target_arch="wasm32")]
#[no_mangle]
extern "C" {
    #[no_mangle]
    pub fn server(
        length: usize,
        message: *const u8,
        cb: unsafe extern "C" fn(arg1: usize, arg2: *const u8),
    );
}

#[cfg(not(target_arch="wasm32"))]
pub use dslcad::server;
