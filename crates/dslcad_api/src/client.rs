use crate::{decode_from_slice, encode, ServerFn};
use serde::{Deserialize, Serialize};
use std::future::Future;

use crate::busy_loop::busy_loop;
use std::marker::PhantomData;
use std::pin::Pin;
use std::ptr::slice_from_raw_parts;
use std::task::{Context, Poll};

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

    pub fn send(&self, message: T) -> PendingMessage<T> {
        let encoded = encode(&message).expect("failed to encode message");
        unsafe {
            (self.server)(encoded.len(), encoded.as_ptr(), client_handler);
        }
        PendingMessage(PhantomData::default())
    }
}

pub struct PendingMessage<T>(PhantomData<T>);

impl<T: for<'a> Deserialize<'a>> PendingMessage<T> {
    pub fn busy_loop(self) -> T {
        busy_loop(self)
    }
}

impl<T: for<'a> Deserialize<'a>> Future for PendingMessage<T> {
    type Output = T;

    fn poll(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Self::Output> {
        unsafe {
            match BUFFER.take() {
                None => Poll::Pending,
                Some(buff) => {
                    Poll::Ready(decode_from_slice(&buff).expect("failed to decode message"))
                }
            }
        }
    }
}
