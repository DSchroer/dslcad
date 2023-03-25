use crate::{decode_from_slice, encode, ServerFn};
use serde::{Deserialize, Serialize};
use serde_binary::Error;
use std::marker::PhantomData;
use std::ptr::slice_from_raw_parts;

static mut BUFFER: Option<Vec<u8>> = None;

pub struct Client<T> {
    server: ServerFn,
    _phantom: PhantomData<T>,
}

extern "C" fn client_handler(length: usize, message: *const u8) {
    let message = unsafe { &*slice_from_raw_parts(message, length) };
    unsafe { BUFFER = Some(message.to_vec()) }
}

impl<T: Serialize + for<'a> Deserialize<'a>> Client<T> {
    pub fn new(server: ServerFn) -> Self {
        Self {
            server,
            _phantom: Default::default(),
        }
    }

    pub fn send(&self, message: T) -> T {
        let encoded = encode(&message).expect("failed to encode message");
        (self.server)(encoded.len(), encoded.as_ptr(), client_handler);

        let response = unsafe { BUFFER.take() }.expect("server did not respond");

        decode_from_slice(&response).expect("failed to decode message")
    }
}
