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

pub struct ServiceRpc<'ir> {
    pub(crate) rpc: &'ir IrdestSdk,
}

impl<'ir> ServiceRpc<'ir> {
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
}
