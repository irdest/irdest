//! A set of type builders for the basic qrpc-sdk
//!
//! It's recommended to write similar abstraction layers in your own
//! crate, so to make it easy for other third-party developers to use
//! your service's APIs and types as easily as possible.
//!
//! In your service you will likely not need to consume this API.  It
//! is included for debugging purposes.

use crate::{io::MsgReader, rpc::*, Service};
use byteorder::{BigEndian, ByteOrder};
use capnp::{message::Builder as Bld, serialize_packed};
use identity::Identity;
use socket2::Socket;

/// Generate an registry message for this service
pub fn register(service: &Service) -> (String, Vec<u8>) {
    todo!()
}

/// Generate an unregistry message for this service
pub fn unregister(hash_id: Identity) -> (String, Vec<u8>) {
    todo!()
}

pub fn upgrade(s: &Service, hash_id: Identity) -> (String, Vec<u8>) {
    todo!()
}

/// This function is only included for debugging reasons.  There's
/// basically no reason to call this function directly.
#[doc(hidden)]
pub fn _internal_to(target: String, data: Vec<u8>) -> Vec<u8> {
    let mut msg = Bld::new_default();
    let mut carrier = msg.init_root::<carrier::Builder>();
    carrier.set_target(&target);
    carrier.set_data(&data);

    let mut buffer = vec![];
    serialize_packed::write_message(&mut buffer, &msg).unwrap();

    let len = buffer.len();
    let mut message = vec![8];
    BigEndian::write_u64(&mut message, len as u64);

    message.append(&mut buffer);
    message
}

pub fn _internal_from(socket: &Socket) -> (String, Vec<u8>) {
    let mut len = vec![0; 8];
    loop {
        let (l, a) = socket.peek_from(&mut len).unwrap();
        if l == 8 {
            break;
        }
    }

    let (_, _) = socket.recv_from(&mut len).unwrap();
    let len = BigEndian::read_u64(&len);
    let mut buffer = vec![0; len as usize];
    socket.recv_from(&mut buffer).unwrap();

    let msg = MsgReader::new(buffer).unwrap();
    let carrier: carrier::Reader = msg.get_root().unwrap();
    let target = carrier.get_target().unwrap();
    let data = carrier.get_data().unwrap();

    (target.to_string(), data.to_vec())
}
