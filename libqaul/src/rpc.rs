//! libqaul RPC compatibility adapter
//!
//! By default `libqaul` is only meant to be used by local Rust
//! clients.  To allow third-party clients to also interact with a
//! running stack, you should use the qrpc bus.  This module exposes
//! some utilities to bind libqaul functions to an rpc server.
//!
//! To write a service to use libqaul, include the client-lib
//! (libqaul-rpc) for type and API configuration.

use crate::{
    types::rpc::{Capabilities, Reply, UserCapabilities, UserReply},
    Identity, QaulRef,
};
use async_std::{sync::Arc, task};
use qrpc_sdk::{default_socket_path, error::RpcResult, io::Message, RpcSocket, Service};
use std::str;

/// A pluggable RPC server that wraps around libqaul
///
/// Initialise this server with a fully initialised [`Qaul`] instance.
/// You will lose access to this type once you start the RPC server.
/// Currently there is no self-management interface available via
/// qrpc.
pub struct RpcServer {
    inner: QaulRef,
    socket: Arc<RpcSocket>,
    serv: Service,
    id: Identity,
}

impl RpcServer {
    /// Wrapper around `new` with `default_socket_path()`
    pub async fn start_default(inner: QaulRef) -> RpcResult<Arc<Self>> {
        let (addr, port) = default_socket_path();
        Self::new(inner, addr, port).await
    }

    pub async fn new(inner: QaulRef, addr: &str, port: u16) -> RpcResult<Arc<Self>> {
        let socket = RpcSocket::connect(addr, port).await?;

        let mut serv = Service::new(
            crate::types::rpc::ADDRESS,
            1,
            "Core component for qaul ecosystem",
        );
        let id = serv.register(Arc::clone(&socket)).await?;

        debug!("libqaul service ID: {}", id);

        let _self = Arc::new(Self {
            inner,
            serv,
            socket,
            id,
        });

        let _this = Arc::clone(&_self);
        task::spawn(async move { _this.run_listen().await });

        Ok(_self)
    }

    async fn run_listen(self: &Arc<Self>) {
        let this = Arc::clone(self);
        self.socket
            .listen(move |msg| {
                let req = str::from_utf8(msg.data.as_slice())
                    .ok()
                    .and_then(|json| Capabilities::from_json(json))
                    .unwrap();

                let _this = Arc::clone(&this);
                task::spawn(async move { _this.spawn_on_request(msg, req).await });
            })
            .await;
    }

    async fn spawn_on_request(self: &Arc<Self>, msg: Message, cap: Capabilities) {
        let reply = match cap {
            Capabilities::Users(UserCapabilities::Create { pw }) => self
                .inner
                .users()
                .create(pw.as_str())
                .await
                .map(|auth| Reply::Users(UserReply::Auth(auth)))
                .map_err(|e| Reply::Error(e)),
            _ => todo!(),
        }
        .unwrap();

        self.socket
            .reply(msg.reply("...".into(), reply.to_json().as_bytes().to_vec()))
            .await
            .unwrap();
    }
}
