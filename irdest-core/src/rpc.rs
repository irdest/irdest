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
    helpers::TagSet,
    types::rpc::{
        Capabilities, MessageCapabilities, MessageReply, Reply, UserCapabilities, UserReply,
        ADDRESS,
    },
    types::services::Service,
    users::{UserAuth, UserProfile, UserUpdate},
    Identity, IrdestRef,
};
use async_std::{sync::Arc, task};
use irpc_sdk::{
    default_socket_path,
    error::RpcResult,
    io::{self, Message},
    proto::{SdkCommand, SdkReply},
    Capabilities as ServiceCapabilities, RpcSocket, Service as SdkService, SubManager,
};
use std::{str, sync::atomic::Ordering};

/// A pluggable RPC server that wraps around libqaul
///
/// Initialise this server with a fully initialised [`Irdest`] instance.
/// You will lose access to this type once you start the RPC server.
/// Currently there is no self-management interface available via
/// qrpc.
pub struct RpcServer {
    inner: IrdestRef,
    socket: Arc<RpcSocket>,
    serv: SdkService,
    id: Identity,
    subs: SubManager,
}

impl RpcServer {
    /// Wrapper around `new` with `default_socket_path()`
    pub async fn start_default(inner: IrdestRef) -> RpcResult<Arc<Self>> {
        let (addr, port) = default_socket_path();
        Self::new(inner, addr, port).await
    }

    pub async fn new(inner: IrdestRef, addr: &str, port: u16) -> RpcResult<Arc<Self>> {
        let socket = RpcSocket::connect(addr, port).await?;

        let mut serv = SdkService::new(
            crate::types::rpc::ADDRESS,
            1,
            "Core component for irdest ecosystem",
        );
        let id = serv
            .register(&socket, ServiceCapabilities::basic_json())
            .await?;
        debug!("irdest-core service ID: {}", id);

        let _self = Arc::new(Self {
            inner,
            serv,
            socket,
            id,
            subs: SubManager::new(),
        });

        let _this = Arc::clone(&_self);
        task::spawn(async move { _this.run_listen().await });

        Ok(_self)
    }

    async fn run_listen(self: &Arc<Self>) {
        let this = Arc::clone(self);
        self.socket
            .listen(move |msg| {
                let enc = this.serv.encoding();

                let req = io::decode::<String>(enc, &msg.data)
                    .ok()
                    .and_then(|json| Capabilities::from_json(&json))
                    .unwrap();

                let _this = Arc::clone(&this);
                task::spawn(async move { _this.spawn_on_request(msg, req).await });
            })
            .await;
    }

    async fn spawn_on_request(self: &Arc<Self>, msg: Message, cap: Capabilities) {
        use Capabilities::*;
        use MessageCapabilities as MsgCap;
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

            // =^-^= Message API functions =^-^=
            Messages(MsgCap::Subscribe {
                auth,
                service,
                tags,
            }) => self.message_subscribe(&msg, auth, service, tags).await,

            //     // Subscribe the user and map the output to an encoded buffer
            //     let reply = match self
            //         .message_subscribe(&msg, &self.socket, auth, service, tags)
            //         .await
            //     {
            //         Ok(ref sdk) => io::encode(enc, sdk),
            //         Err(ref ir) => io::encode(enc, ir),
            //     }
            //     .unwrap();

            //     // Send the reply and exit the function early
            //     self.socket.reply(msg.reply(ADDRESS, reply)).await.unwrap();
            //     return;
            // }
            // Subscriptions(SubCap::Unregister(id)) => self.subscription_unregister(id).await,

            // =^-^= Everything else is todo! =^-^=
            _ => todo!(),
        };

        self.socket
            .reply(msg.reply(ADDRESS, reply.to_json().as_bytes().to_vec()))
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

    async fn message_subscribe(
        self: &Arc<Self>,
        msg: &Message,
        auth: UserAuth,
        service: Service,
        tags: TagSet,
    ) -> Reply {
        match self.inner.messages().subscribe(auth, service, tags).await {
            Ok(sub) => {
                let to = msg.from.clone();
                let socket = Arc::clone(&self.socket);
                let _msg = msg.clone();

                let b = self.subs.insert(msg.id).await;

                // Spawn a talk to poll the subscription and then send
                // out a message to the subscription client
                //
                // TODO: this needs a better utility in irpc-sdk
                task::spawn(async move {
                    while b.load(Ordering::Relaxed) {
                        let new_msg = sub.next().await;

                        // Special check here because a subscription
                        // might be idle for ages and the run
                        // condition changed.
                        //
                        // FIXME: wrap ArcBool into a Future that you
                        // can select on
                        if !b.load(Ordering::Relaxed) {
                            break;
                        }

                        // Push message to socket
                        socket
                            .reply(
                                _msg.clone().reply(
                                    ADDRESS,
                                    // Create a reply message
                                    Reply::Message(MessageReply::Message(new_msg))
                                        .to_json()
                                        .as_bytes()
                                        .to_vec(),
                                ),
                            )
                            .await
                            .unwrap();
                    }
                });

                Reply::Subscription(msg.id)
            }
            Err(e) => Reply::Error(e),
        }
    }
}

/// Keep polling a subscription until it is deallocated
pub struct RpcSubscription {
    socket: Arc<RpcSocket>,
}
