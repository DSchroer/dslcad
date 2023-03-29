use std::mem;

#[no_mangle]
pub unsafe extern "C" fn new_buffer(length: usize) -> *mut u8 {
    let mut buffer = vec![0; length];
    assert_eq!(buffer.len(), buffer.capacity());
    let ptr = buffer.as_mut_ptr();
    mem::forget(buffer);
    ptr
}

#[no_mangle]
pub unsafe extern "C" fn drop_buffer(length: usize, pointer: *mut u8) {
    drop(Vec::from_raw_parts(pointer, length, length));
}
