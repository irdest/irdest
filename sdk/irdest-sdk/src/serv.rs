use crate::IrdestSdk;

pub use crate::services::ServiceEvent;
use async_std::task;
pub use irpc_sdk::{
    default_socket_path,
    error::{RpcError, RpcResult},
    io::{self, Message},
    RpcSocket, Service, SubSwitch, Subscription, ENCODING_JSON,
};
pub use std::{str, sync::Arc};

use ircore_types::rpc::{Capabilities, Reply, ServiceCapabilities, ServiceReply};
use ircore_types::{services::StoreKey, users::UserAuth};

pub struct ServiceRpc<'ir> {
    pub(crate) rpc: &'ir IrdestSdk,
}

impl<'ir> ServiceRpc<'ir> {
    /// Register a unique service with the irdest core
    // TODO: replace this callback with a Future generator?
    pub async fn register<S: Into<String>>(
        &self,
        name: S,
    ) -> RpcResult<Subscription<ServiceEvent>> {
        match self
            .rpc
            .send(Capabilities::Services(ServiceCapabilities::Register {
                name: name.into(),
            }))
            .await
        {
            Ok(Reply::Service(ServiceReply::Register { sub })) => {
                // We create a subscription handler for this ID and
                // map it to the user call-back

                let s: Subscription<ServiceEvent> = self.rpc.subs.create(ENCODING_JSON, sub).await;
                let rpc = Arc::clone(&self.rpc.socket.clone());
                let subs = Arc::clone(&self.rpc.subs);
                let enc = self.rpc.enc;

                // Spawn another task that waits for incoming service
                // events and forwards them to the subscription
                task::spawn(async move {
                    let subs = Arc::clone(&subs);

                    rpc.wait_for(sub, |Message { data, .. }| {
                        let subs = Arc::clone(&subs);

                        async move {
                            match io::decode(enc, &data) {
                                Ok(Reply::Service(ServiceReply::Event { ref event, sub })) => {
                                    debug!("Received event: {}", event.tt());
                                    subs.forward(sub, event).await
                                }
                                _ => {
                                    warn!("Received invalid ServiceEvent payload; dropping!");
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
            _ => Err(RpcError::EncoderFault("invalid reply payload!".into())),
        }
    }

    /// Store some service data in irdest-core
    ///
    /// A service can store metadata in the same encrypted database
    /// that irdest-core uses.  Each service has access to a single
    /// map [`MetadataMap`](ir_types::service::MetadataMap), which
    /// maps `StoreKey`s to arbitrary types that can be serialised
    /// into byte-arrays.
    ///
    /// A [`StoreKey`](ir_types::service::StoreKey) is a 2-String
    /// tuple that can be used by a service to construct meaningful
    /// namesspaces, without requiring nested datastructures.
    pub async fn insert<S, K>(
        &self,
        auth: UserAuth,
        service: S,
        key: K,
        value: Vec<u8>,
    ) -> RpcResult<()>
    where
        S: Into<String>,
        K: Into<StoreKey>,
    {
        let service = service.into();
        let key = key.into();

        match self
            .rpc
            .send(Capabilities::Services(ServiceCapabilities::Insert {
                auth,
                service,
                key,
                value,
            }))
            .await
        {
            Ok(Reply::Service(ServiceReply::Ok)) => Ok(()),
            Err(e) => Err(e),
            _ => Err(RpcError::EncoderFault("invalid reply payload!".into())),
        }
    }

    /// Query for a key in the service map
    pub async fn query<S, K>(&self, auth: UserAuth, service: S, key: K) -> RpcResult<Vec<u8>>
    where
        S: Into<String>,
        K: Into<StoreKey>,
    {
        let service = service.into();
        let key = key.into();

        match self
            .rpc
            .send(Capabilities::Services(ServiceCapabilities::Query {
                auth,
                service,
                key,
            }))
            .await
        {
            Ok(Reply::Service(ServiceReply::Query { val, .. })) => Ok(val),
            Err(e) => Err(e),
            _ => Err(RpcError::EncoderFault("invalid reply payload!".into())),
        }
    }

    /// Delete a key from the service metadata map
    pub async fn delete<S, K>(&self, auth: UserAuth, service: S, key: K) -> RpcResult<()>
    where
        S: Into<String>,
        K: Into<StoreKey>,
    {
        let service = service.into();
        let key = key.into();

        match self
            .rpc
            .send(Capabilities::Services(ServiceCapabilities::Delete {
                auth,
                service,
                key,
            }))
            .await
        {
            Ok(Reply::Service(ServiceReply::Ok)) => Ok(()),
            Err(e) => Err(e),
            _ => Err(RpcError::EncoderFault("invalid reply payload!".into())),
        }
    }
}
