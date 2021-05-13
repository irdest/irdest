//! Exposing the irdest-core messages API via the RPC interface

use crate::IrdestSdk;

use alexandria_tags::TagSet;
use async_std::task;
pub use irpc_sdk::{
    default_socket_path,
    error::{RpcError, RpcResult},
    io::{self, Message},
    RpcSocket, Service, SubSwitch, Subscription, ENCODING_JSON,
};
pub use std::{str, sync::Arc};

// irdest-core types used by this API
use ircore_types::messages::{IdType, Message as IrdestMessage, Mode, MsgId};
use ircore_types::rpc::{self, Capabilities, MessageReply, Reply, SubscriptionReply};
use ircore_types::services::Service as ServiceId;
use ircore_types::users::UserAuth;

pub struct MessageRpc<'ir> {
    pub(crate) rpc: &'ir IrdestSdk,
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
