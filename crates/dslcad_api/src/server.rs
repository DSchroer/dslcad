use crate::{decode, encode, ClientFn};
use serde::{Deserialize, Serialize};
use serde_binary::Error;

#[macro_export]
macro_rules! server_fn {
    ($type: ident) => {
        #[no_mangle]
        pub extern "C" fn server(length: usize, message: *const u8, cb: dslcad_api::ClientFn) {
            $type::receive(length, message, cb)
        }
    };
}

pub trait Server<T: for<'a> Deserialize<'a> + Serialize> {
    fn on_message(message: T) -> T;

    fn receive(length: usize, message: *const u8, cb: ClientFn) {
        let message = decode(length, message).expect("failed to decode message");
        let response = Self::on_message(message);
        let response = encode(&response).expect("failed to encode response");
        cb(response.len(), response.as_ptr());
    }
}
