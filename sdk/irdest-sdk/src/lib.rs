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

pub use ircore_types::*;
pub use irpc_sdk::{
    default_socket_path,
    error::{RpcError, RpcResult},
    io::{self, Message},
    RpcSocket, Service,
};
pub use std::{str, sync::Arc};

use alexandria_tags::TagSet;
use messages::{IdType, Mode, MsgId};
use rpc::{Capabilities, MessageReply, Reply, UserCapabilities, UserReply, ADDRESS};
use services::Service as ServiceId;
use users::UserAuth;

/// A irpc wrapper for irdest-core
///
/// This component exposes a public API surface to mirror the irdest-core
/// crate.  This means that other clients on the irpc bus can include
/// this surface to get access to all irdest-core functions, thate are
/// transparently mapped to the underlying irdest-core instance
/// potentially running in a different process.
pub struct IrdestSdk {
    socket: Arc<RpcSocket>,
    addr: String,
    enc: u8,
}

impl IrdestSdk {
    pub fn connect(service: &Service) -> RpcResult<Self> {
        let socket = service.socket();
        let addr = service.name.clone();
        let enc = service.encoding();
        Ok(Self { socket, addr, enc })
    }

    pub fn users<'ir>(&'ir self) -> UserRpc<'ir> {
        UserRpc { rpc: self }
    }

    pub fn messages<'ir>(&'ir self) -> MessageRpc<'ir> {
        MessageRpc { rpc: self }
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

pub struct UserRpc<'ir> {
    rpc: &'ir IrdestSdk,
}

impl<'ir> UserRpc<'ir> {
    pub async fn create<S: Into<String>>(&'ir self, pw: S) -> RpcResult<UserAuth> {
        if let Reply::Users(UserReply::Auth(auth)) = self
            .rpc
            .send(Capabilities::Users(UserCapabilities::Create {
                pw: pw.into(),
            }))
            .await?
        {
            Ok(auth)
        } else {
            Err(RpcError::EncoderFault("Invalid reply payload!".into()))
        }
    }

    pub async fn is_authenticated(&'ir self, auth: UserAuth) -> RpcResult<()> {
        match self
            .rpc
            .send(Capabilities::Users(UserCapabilities::IsAuthenticated {
                auth,
            }))
            .await
        {
            Ok(Reply::Users(UserReply::Ok)) => Ok(()),
            Err(e) => Err(e),
            _ => Err(RpcError::EncoderFault("Invalid reply payload!".into())),
        }
    }
}

pub struct MessageRpc<'ir> {
    rpc: &'ir IrdestSdk,
}

impl<'ir> MessageRpc<'ir> {
    pub async fn send<S, T>(
        &'ir self,
        auth: UserAuth,
        mode: Mode,
        id_type: IdType,
        service: S,
        tags: T,
        payload: Vec<u8>,
    ) -> RpcResult<MsgId>
    where
        S: Into<ServiceId>,
        T: Into<TagSet>,
    {
        match self
            .rpc
            .send(Capabilities::Messages(rpc::MessageCapabilities::Send {
                auth,
                mode,
                id_type,
                service: service.into(),
                tags: tags.into(),
                payload,
            }))
            .await
        {
            Ok(Reply::Message(MessageReply::MsgId(id))) => Ok(id),
            Err(e) => Err(e),
            _ => Err(RpcError::EncoderFault("Invalid reply payload!".into())),
        }
    }
}
