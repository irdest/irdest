//! Irdest development SDK.
//!
//! This SDK provides you with asynchronous access to all of the core
//! irdest function interfaces, while being connected to a remote
//! server instance via the irdest-rpc (irpc) system.
//!
//! To interact with the irpc system you need to create a
//! [`Service`](irpc_sdk::Service), which is registered with the
//! remote RPC broker.
//!
//! ```rust
//! use irpc_sdk::Service;
//! use irdest_sdk::IrdestSdk;
//! ```

#[macro_use]
extern crate tracing;

mod msgs;
pub use msgs::MessageRpc;

mod usrs;
pub use usrs::UserRpc;

mod serv;
pub use serv::ServiceRpc;

pub use ircore_types::*;
pub use irpc_sdk::{
    default_socket_path,
    error::{RpcError, RpcResult},
    io::{self, Message},
    RpcSocket, Service, SubSwitch, Subscription, ENCODING_JSON,
};
pub use std::{str, sync::Arc};

use rpc::{Capabilities, Reply, ADDRESS};

/// A irpc wrapper for irdest-core
///
/// This component exposes a public API surface to mirror the irdest-core
/// crate.  This means that other clients on the irpc bus can include
/// this surface to get access to all irdest-core functions, thate are
/// transparently mapped to the underlying irdest-core instance
/// potentially running in a different process.
#[derive(Clone)]
pub struct IrdestSdk {
    socket: Arc<RpcSocket>,
    subs: Arc<SubSwitch>,
    addr: String,
    enc: u8,
}

impl IrdestSdk {
    pub fn connect(service: &Service) -> RpcResult<Self> {
        let socket = service.socket();
        let addr = service.name.clone();
        let enc = service.encoding();
        let subs = SubSwitch::new(enc);
        Ok(Self {
            socket,
            subs,
            addr,
            enc,
        })
    }

    pub fn users<'ir>(&'ir self) -> UserRpc<'ir> {
        UserRpc { rpc: self }
    }

    pub fn messages<'ir>(&'ir self) -> MessageRpc<'ir> {
        MessageRpc { rpc: self }
    }

    pub fn services<'ir>(&'ir self) -> ServiceRpc<'ir> {
        ServiceRpc { rpc: self }
    }

    
    async fn send(&self, cap: Capabilities) -> RpcResult<Reply> {
        let json = cap.to_json();
        let msg = Message::to_addr(ADDRESS, &self.addr, json.as_bytes().to_vec());

        self.socket
            .send(msg, |Message { data, .. }| {
                match io::decode::<Reply>(self.enc, &data).ok() {
                    // Map the Reply::Error field to a Rust error
                    Some(Reply::Error(e)) => Err(RpcError::Other(e.to_string())),
                    None => Err(RpcError::EncoderFault(
                        "Received invalid json payload!".into(),
                    )),
                    Some(r) => Ok(r),
                }
            })
            .await
    }
}
