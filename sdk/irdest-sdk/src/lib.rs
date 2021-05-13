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

pub use ircore_types::*;
pub use irpc_sdk::{
    default_socket_path,
    error::{RpcError, RpcResult},
    io::{self, Message},
    RpcSocket, Service, SubSwitch, Subscription, ENCODING_JSON,
};
pub use std::{str, sync::Arc};

use alexandria_tags::TagSet;
use async_std::task;
use messages::{IdType, Message as IrdestMessage, Mode, MsgId};
use rpc::{
    Capabilities, MessageReply, Reply, SubscriptionReply, UserCapabilities, UserReply, ADDRESS,
};
use services::Service as ServiceId;
use users::{UserAuth, UserProfile, UserUpdate};

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
    /// Enumerate all users available
    ///
    /// No information about sessions or existing login state is
    /// stored or accessible via this API.
    pub async fn list(&self) -> Vec<UserProfile> {
        match self
            .rpc
            .send(Capabilities::Users(UserCapabilities::List))
            .await
        {
            Ok(Reply::Users(UserReply::List(list))) => list,
            _ => vec![],
        }
    }

    /// Enumerate remote stored users available
    pub async fn list_remote(&self) -> Vec<UserProfile> {
        match self
            .rpc
            .send(Capabilities::Users(UserCapabilities::ListRemote))
            .await
        {
            Ok(Reply::Users(UserReply::List(list))) => list,
            _ => vec![],
        }
    }

    /// Check if a user ID and token combination is valid
    pub async fn is_authenticated(&self, auth: UserAuth) -> RpcResult<()> {
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

    /// Create a new user and authenticated session
    ///
    /// The specified password `pw` is used to encrypt the user's
    /// private key and message stores and should be kept safe from
    /// potential attackers.
    ///
    /// It's mandatory to choose a password here, however it is
    /// possible for a frontend to choose a random sequence _for_ a
    /// user, instead of leaving files completely unencrypted. In this
    /// case, there's no real security, but a drive-by will still only
    /// grab encrypted files.
    pub async fn create(&self, pw: &str) -> RpcResult<UserAuth> {
        match self
            .rpc
            .send(Capabilities::Users(UserCapabilities::Create {
                pw: pw.into(),
            }))
            .await
        {
            Ok(Reply::Users(UserReply::Auth(auth))) => Ok(auth),
            Err(e) => Err(e),
            _ => Err(RpcError::EncoderFault("Invalid reply payload!".into())),
        }
    }

    /// Delete a local user from the auth store
    ///
    /// This function requires a valid login for the user that's being
    /// deleted.  This does not delete any data associated with this
    /// user, or messages from the node (or other device nodes).
    pub async fn delete(&self, auth: UserAuth) -> RpcResult<()> {
        match self
            .rpc
            .send(Capabilities::Users(UserCapabilities::Delete { auth }))
            .await
        {
            Ok(Reply::Users(UserReply::Ok)) => Ok(()),
            Err(e) => Err(e),
            _ => Err(RpcError::EncoderFault("Invalid reply payload!".into())),
        }
    }

    /// Change the passphrase for an authenticated user
    pub async fn change_pw(&self, auth: UserAuth, new_pw: &str) -> RpcResult<()> {
        let new_pw = new_pw.to_string();
        match self
            .rpc
            .send(Capabilities::Users(UserCapabilities::ChangePw {
                auth,
                new_pw,
            }))
            .await
        {
            Ok(Reply::Users(UserReply::Ok)) => Ok(()),
            Err(e) => Err(e),
            _ => Err(RpcError::EncoderFault("Invalid reply payload!".into())),
        }
    }

    /// Create a new session login for a local User
    pub async fn login<S: Into<String>>(&self, id: Identity, pw: S) -> RpcResult<UserAuth> {
        match self
            .rpc
            .send(Capabilities::Users(UserCapabilities::Login {
                id,
                pw: pw.into(),
            }))
            .await
        {
            Ok(Reply::Users(UserReply::Auth(auth))) => Ok(auth),
            Err(e) => Err(e),
            _ => Err(RpcError::EncoderFault("Invalid reply payload!".into())),
        }
    }

    /// Drop the current session Token, invalidating it
    pub async fn logout(&self, auth: UserAuth) -> RpcResult<()> {
        match self
            .rpc
            .send(Capabilities::Users(UserCapabilities::Logout { auth }))
            .await
        {
            Ok(Reply::Users(UserReply::Ok)) => Ok(()),
            Err(e) => Err(e),
            _ => Err(RpcError::EncoderFault("Invalid reply payload!".into())),
        }
    }

    /// Fetch the `UserProfile` for a known identity, remote or local
    ///
    /// No athentication is required for this endpoint, seeing as only
    /// public information is exposed via the `UserProfile`
    /// abstraction anyway.
    pub async fn get(&self, id: Identity) -> RpcResult<UserProfile> {
        match self
            .rpc
            .send(Capabilities::Users(UserCapabilities::Get { id }))
            .await
        {
            Ok(Reply::Users(UserReply::Profile(profile))) => Ok(profile),
            Err(e) => Err(e),
            _ => Err(RpcError::EncoderFault("Invalid reply payload!".into())),
        }
    }

    /// Update a `UserProfile` with a `UserUpdate` diff type
    pub async fn update(&self, auth: UserAuth, update: UserUpdate) -> RpcResult<()> {
        match self
            .rpc
            .send(Capabilities::Users(UserCapabilities::Update {
                auth,
                update,
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

    /// Subscribe to a stream of future message updates
    pub async fn subscribe<S, T>(
        &self,
        auth: UserAuth,
        service: S,
        tags: T,
    ) -> RpcResult<Subscription<IrdestMessage>>
    where
        S: Into<ServiceId>,
        T: Into<TagSet>,
    {
        let service = service.into();

        match self
            .rpc
            .send(Capabilities::Messages(
                rpc::MessageCapabilities::Subscribe {
                    auth,
                    service,
                    tags: tags.into(),
                },
            ))
            .await
        {
            // Create a Subscription object and a task that pushes
            // updates to it for incoming subscription events
            Ok(Reply::Subscription(SubscriptionReply::Ok(sub_id))) => {
                let s = self.rpc.subs.create(ENCODING_JSON, sub_id).await;

                // Listen for events for this task
                let rpc = Arc::clone(&self.rpc.socket.clone());
                let subs = Arc::clone(&self.rpc.subs);
                let enc = self.rpc.enc;
                task::spawn(async move {
                    let subs = Arc::clone(&subs);

                    rpc.wait_for(sub_id, |Message { data, .. }| {
                        let subs = Arc::clone(&subs);

                        async move {
                            match io::decode(enc, &data) {
                                Ok(Reply::Message(MessageReply::Message(ref msg))) => {
                                    subs.forward(sub_id, msg).await
                                }
                                _ => {
                                    warn!("Received invalid subscription payload; dropping!");
                                    Ok(())
                                }
                            }
                        }
                    })
                    .await
                    .unwrap();
                });

                Ok(s)
            }
            Err(e) => Err(e),
            _ => Err(RpcError::EncoderFault("Invalid reply payload!".into())),
        }
    }
}
