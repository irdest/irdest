//! qaul development SDK.
//!
//!
//!
//! The API surface is exposed via the `QaulRpc` type, while data
//! types are exposed via the `libqaul-types` crate (re-exported from
//! this crate via [`types`]).
//!
//! Check the qrpc-sdk documentation to learn how to use this crate.

pub use ircore_types::*;
pub use irpc_sdk::{
    default_socket_path,
    error::{RpcError, RpcResult},
    io::Message,
    RpcSocket, Service,
};
pub use std::{str, sync::Arc};

use rpc::{Capabilities, Reply, UserCapabilities, UserReply, ADDRESS};
use users::UserAuth;

/// A qrpc wrapper for libqaul
///
/// This component exposes a public API surface to mirror the libqaul
/// crate.  This means that other clients on the qrpc bus can include
/// this surface to get access to all libqaul functions, thate are
/// transparently mapped to the underlying libqaul instance
/// potentially running in a different process.
pub struct QaulRpc {
    socket: Arc<RpcSocket>,
    addr: String,
}

impl QaulRpc {
    pub fn connect(service: &Service) -> RpcResult<Self> {
        let socket = service.socket();
        let addr = service.name.clone();
        Ok(Self { socket, addr })
    }

    pub fn users<'q>(&'q self) -> UserRpc<'q> {
        UserRpc { rpc: self }
    }

    async fn send(&self, cap: Capabilities) -> RpcResult<Reply> {
        let json = cap.to_json();
        let msg = Message::to_addr(ADDRESS, &self.addr, json.as_bytes().to_vec());

        self.socket
            .send(msg, |Message { data, .. }| {
                match str::from_utf8(data.as_slice())
                    .ok()
                    .and_then(|json| Reply::from_json(json))
                {
                    // Map the Reply::Error field to a Rust error
                    Some(Reply::Error(e)) => Err(RpcError::Other(e.to_string())),
                    None => Err(RpcError::EncoderFault("Invalid json payload!".into())),
                    Some(r) => Ok(r),
                }
            })
            .await
    }
}

pub struct UserRpc<'q> {
    rpc: &'q QaulRpc,
}

impl<'q> UserRpc<'q> {
    pub async fn create<S: Into<String>>(&'q self, pw: S) -> RpcResult<UserAuth> {
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
}
