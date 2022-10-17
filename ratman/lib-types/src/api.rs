// SPDX-FileCopyrightText: 2019-2022 Katharina Fey <kookie@spacekookie.de>
//
// SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

//! Ratman API abstractions

pub use crate::proto::api::{
    ApiMessage, ApiMessage_oneof_inner as ApiMessageEnum, Peers, Peers_Type, Receive, Receive_Type,
    Send, Send_Type, Setup, Setup_Type,
};
use crate::proto::message::Message;
use crate::Address;

//////////// SEND type

fn send(msg: Message, t: Send_Type, scope: Option<Address>, mirror: bool) -> Send {
    let mut send = Send::new();
    send.set_field_type(t);
    send.set_msg(msg);
    send.set_mirror(mirror);
    if let Some(scope) = scope {
        send.set_scope(scope.as_bytes().to_vec());
    }

    send
}

/// Create a new default send message
pub fn send_default(msg: Message) -> Send {
    send(msg, Send_Type::DEFAULT, None, false)
}

/// Create a new flood send message
pub fn send_flood(msg: Message, scope: Address, mirror: bool) -> Send {
    send(msg, Send_Type::FLOOD, Some(scope), mirror)
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
pub fn online(id: Address, token: Vec<u8>) -> Setup {
    let mut setup = Setup::new();
    setup.set_field_type(Setup_Type::ONLINE);
    setup.set_id(id.as_bytes().to_vec());
    setup.set_token(token.into());
    setup
}

/// Create an offline message
pub fn offline(id: Address, token: Vec<u8>) -> Setup {
    let mut setup = Setup::new();
    setup.set_field_type(Setup_Type::OFFLINE);
    setup.set_id(id.as_bytes().to_vec());
    setup.set_token(token.into());
    setup
}

pub fn online_ack(id: Address) -> Setup {
    let mut setup = Setup::new();
    setup.set_field_type(Setup_Type::ACK);
    setup.set_id(id.as_bytes().to_vec());
    setup
}

pub fn anonymous() -> Setup {
    let mut setup = Setup::new();
    setup.set_field_type(Setup_Type::ANONYMOUS);
    setup
}

//////////// PEERS type

/// Create a new discovery message
pub fn discovery(id: Address) -> Peers {
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

/// Construct a response including "all peers"
pub fn all_peers(ids: Vec<Address>) -> Peers {
    let mut peers = Peers::new();
    peers.set_field_type(Peers_Type::RESP);
    peers.set_peers(
        ids.into_iter()
            .map(|id| id.as_bytes().to_vec())
            .collect::<Vec<_>>()
            .into(),
    );
    peers
}

pub fn peers_req() -> Peers {
    let mut peers = Peers::new();
    peers.set_field_type(Peers_Type::REQ);
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
