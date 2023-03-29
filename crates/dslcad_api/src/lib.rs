mod client;
mod server;

mod busy_loop;
pub mod constants;
mod memory;
pub mod protocol;

use serde::{Deserialize, Serialize};
use serde_binary::binary_stream::Endian;

use std::ptr::slice_from_raw_parts;

pub use client::Client;
pub use serde_binary::Error;
pub use server::Server;

const ENDIAN: Endian = Endian::Big;

pub type ClientFn = unsafe extern "C" fn(usize, *const u8);
pub type ServerFn = unsafe extern "C" fn(usize, *const u8, ClientFn);

fn encode<T: Serialize>(data: &T) -> Result<Vec<u8>, Error> {
    serde_binary::to_vec(data, ENDIAN)
}

fn decode<T: for<'a> Deserialize<'a>>(length: usize, message: *const u8) -> Result<T, Error> {
    let message = unsafe { &*slice_from_raw_parts(message, length) };
    decode_from_slice(message)
}

fn decode_from_slice<T: for<'a> Deserialize<'a>>(message: &[u8]) -> Result<T, Error> {
    serde_binary::from_slice(message, ENDIAN)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, Debug)]
    struct Message {
        secret: u8,
    }

    struct PrintServer;
    impl Server<Message> for PrintServer {
        fn on_message(message: Message) -> Message {
            assert_eq!(42, message.secret);
            Message { secret: 1 }
        }
    }

    extern "C" fn printer(length: usize, message: *const u8, cb: ClientFn) {
        PrintServer::receive(length, message, cb);
    }

    #[test]
    fn it_can_send_and_receive_messages() {
        let client = Client::new(printer);
        let res = client.send(Message { secret: 42 }).busy_loop();
        assert_eq!(1, res.secret)
    }
}
