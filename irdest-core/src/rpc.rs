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
    error::Error,
    types::rpc::{Capabilities, Reply, UserCapabilities, UserReply},
    users::{UserAuth, UserProfile, UserUpdate},
    Identity, IrdestRef,
};
use async_std::{sync::Arc, task};
use irpc_sdk::{
    default_socket_path, error::RpcResult, io::Message, Capabilities as ServiceCapabilities,
    RpcSocket, Service,
};
use std::str;

/// A pluggable RPC server that wraps around libqaul
///
/// Initialise this server with a fully initialised [`Irdest`] instance.
/// You will lose access to this type once you start the RPC server.
/// Currently there is no self-management interface available via
/// qrpc.
pub struct RpcServer {
    inner: IrdestRef,
    socket: Arc<RpcSocket>,
    serv: Service,
    id: Identity,
}

impl RpcServer {
    /// Wrapper around `new` with `default_socket_path()`
    pub async fn start_default(inner: IrdestRef) -> RpcResult<Arc<Self>> {
        let (addr, port) = default_socket_path();
        Self::new(inner, addr, port).await
    }

    pub async fn new(inner: IrdestRef, addr: &str, port: u16) -> RpcResult<Arc<Self>> {
        let socket = RpcSocket::connect(addr, port).await?;

        let mut serv = Service::new(
            crate::types::rpc::ADDRESS,
            1,
            "Core component for qaul ecosystem",
        );
        serv.register(&socket, ServiceCapabilities::basic_json())
            .await?;
        let id = serv.id().unwrap();

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
                    .and_then(|json| Capabilities::from_json(dbg!(json)))
                    .unwrap();

                let _this = Arc::clone(&this);
                task::spawn(async move { _this.spawn_on_request(msg, req).await });
            })
            .await;
    }

    async fn spawn_on_request(self: &Arc<Self>, msg: Message, cap: Capabilities) {
        use Capabilities::*;
        use UserCapabilities as UserCap;

        let reply = match cap {
            // =^-^= User API functions =^-^=
            Users(UserCap::List) => self.user_list().await,
            Users(UserCap::ListRemote) => self.user_list_remote().await,
            Users(UserCap::IsAuthenticated { auth }) => self.user_is_authenticated(auth).await,
            Users(UserCap::Create { pw }) => self.user_create(pw.as_str()).await,
            Users(UserCap::Delete { auth }) => self.user_delete(auth).await,
            Users(UserCap::ChangePw { auth, new_pw }) => self.user_change_pw(auth, new_pw),
            Users(UserCap::Login { id, pw }) => self.user_login(id, pw).await,
            Users(UserCap::Logout { auth }) => self.user_logout(auth).await,
            Users(UserCap::Get { id }) => self.user_get(id).await,
            Users(UserCap::Update { auth, update }) => self.user_update(auth, update).await,

            // =^-^= Everything else is todo! =^-^=
            _ => todo!(),
        };

        self.socket
            .reply(msg.reply("...".into(), reply.to_json().as_bytes().to_vec()))
            .await
            .unwrap();
    }

    /////// Internal command wrappers

    async fn user_list(self: &Arc<Self>) -> Reply {
        Reply::Users(UserReply::List(self.inner.users().list().await))
    }

    async fn user_list_remote(self: &Arc<Self>) -> Reply {
        Reply::Users(UserReply::List(self.inner.users().list().await))
    }

    async fn user_is_authenticated(self: &Arc<Self>, auth: UserAuth) -> Reply {
        match self.inner.users().is_authenticated(auth).await {
            Ok(()) => Reply::Users(UserReply::Ok),
            Err(e) => Reply::Error(e),
        }
    }

    async fn user_create(self: &Arc<Self>, pw: &str) -> Reply {
        match self.inner.users().create(pw).await {
            Ok(auth) => Reply::Users(UserReply::Auth(auth)),
            Err(e) => Reply::Error(e),
        }
    }

    async fn user_delete(self: &Arc<Self>, auth: UserAuth) -> Reply {
        match self.inner.users().delete(auth).await {
            Ok(()) => Reply::Users(UserReply::Ok),
            Err(e) => Reply::Error(e),
        }
    }

    fn user_change_pw(self: &Arc<Self>, auth: UserAuth, pw: String) -> Reply {
        match self.inner.users().change_pw(auth, pw.as_str()) {
            Ok(()) => Reply::Users(UserReply::Ok),
            Err(e) => Reply::Error(e),
        }
    }

    async fn user_login(self: &Arc<Self>, id: Identity, pw: String) -> Reply {
        match self.inner.users().login(id, pw.as_str()).await {
            Ok(auth) => Reply::Users(UserReply::Auth(auth)),
            Err(e) => Reply::Error(e),
        }
    }

    async fn user_logout(self: &Arc<Self>, auth: UserAuth) -> Reply {
        match self.inner.users().logout(auth).await {
            Ok(()) => Reply::Users(UserReply::Ok),
            Err(e) => Reply::Error(e),
        }
    }

    async fn user_get(self: &Arc<Self>, id: Identity) -> Reply {
        match self.inner.users().get(id).await {
            Ok(profile) => Reply::Users(UserReply::Profile(profile)),
            Err(e) => Reply::Error(e),
        }
    }

    async fn user_update(self: &Arc<Self>, auth: UserAuth, update: UserUpdate) -> Reply {
        match self.inner.users().update(auth, update).await {
            Ok(()) => Reply::Users(UserReply::Ok),
            Err(e) => Reply::Error(e),
        }
    }
}
