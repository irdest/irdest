//! Ratman API abstractions

use crate::message::Message;
pub use crate::proto::api::{
    ApiMessage, ApiMessage_oneof_inner as ApiMessageEnum, Peers, Peers_Type, Receive, Receive_Type,
    Send, Send_Type, Setup, Setup_Type,
};
use ratman_identity::Identity;

//////////// SEND type

fn send(msg: Message, t: Send_Type) -> Send {
    let mut send = Send::new();
    send.set_field_type(t);
    send.set_msg(msg);
    send
}

/// Create a new default send message
pub fn send_default(msg: Message) -> Send {
    send(msg, Send_Type::DEFAULT)
}

/// Create a new flood send message
pub fn send_flood(msg: Message) -> Send {
    send(msg, Send_Type::FLOOD)
}

//////////// RECEIVE type

fn receive(msg: Message, t: Receive_Type) -> Receive {
    let mut receive = Receive::new();
    receive.set_field_type(t);
    receive.set_msg(msg);
    receive
}

/// Create a new default receive message
pub fn receive_default(msg: Message) -> Receive {
    receive(msg, Receive_Type::DEFAULT)
}

/// Create a new flood receive message
pub fn receive_flood(msg: Message) -> Receive {
    receive(msg, Receive_Type::FLOOD)
}

//////////// SETUP type

/// Create the initial Online request
pub fn online_init() -> Setup {
    let mut setup = Setup::new();
    setup.set_field_type(Setup_Type::ONLINE);
    setup
}

/// Create an online message with ID and token
pub fn online(id: Identity, token: Vec<u8>) -> Setup {
    let mut setup = Setup::new();
    setup.set_field_type(Setup_Type::ONLINE);
    setup.set_id(id.as_bytes().to_vec());
    setup.set_token(token.into());
    setup
}

/// Create an offline message
pub fn offline(id: Identity, token: Vec<u8>) -> Setup {
    let mut setup = Setup::new();
    setup.set_field_type(Setup_Type::OFFLINE);
    setup.set_id(id.as_bytes().to_vec());
    setup.set_token(token.into());
    setup
}

//////////// PEERS type

/// Create a new discovery message
pub fn discovery(id: Identity) -> Peers {
    let mut peers = Peers::new();
    peers.set_field_type(Peers_Type::DISCOVER);
    peers.set_peers(
        vec![id]
            .iter()
            .map(|id| id.as_bytes().to_vec())
            .collect::<Vec<_>>()
            .into(),
    );
    peers
}

//////////// APIMESAGE type

pub fn api_send(s: Send) -> ApiMessage {
    let mut msg = ApiMessage::new();
    msg.set_send(s);
    msg
}

pub fn api_recv(r: Receive) -> ApiMessage {
    let mut msg = ApiMessage::new();
    msg.set_recv(r);
    msg
}

pub fn api_setup(s: Setup) -> ApiMessage {
    let mut msg = ApiMessage::new();
    msg.set_setup(s);
    msg
}

pub fn api_peers(p: Peers) -> ApiMessage {
    let mut msg = ApiMessage::new();
    msg.set_peers(p);
    msg
}
