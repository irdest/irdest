use crate::core::GenericEndpoint;
use libratman::{
    futures::stream::{FuturesUnordered, StreamExt},
    tokio,
    types::{InMemoryEnvelope, Neighbour},
    NetmodError, RatmanError, Result,
};
use std::{collections::BTreeMap, sync::Arc};

/// A full runtime system thread handling netmod traffic
///
/// This type can be initialised either for a single netmod, two, or
/// all of them.
pub struct NetmodRuntime {
    state: NetmodState,
}

type EndpointWithName = (String, Arc<GenericEndpoint>);

enum NetmodState {
    Single(EndpointWithName),
    Binary(EndpointWithName, EndpointWithName),
    Complete(BTreeMap<String, Arc<GenericEndpoint>>),
}

impl NetmodState {
    pub async fn send_to(
        &self,
        nmodid: &str,
        target: Neighbour,
        envelope: InMemoryEnvelope,
        exclude: Option<u16>,
    ) -> Result<()> {
        match self {
            Self::Single((name, ep)) if name == nmodid => ep.send(envelope, target, exclude).await,
            Self::Binary((n1, ep1), (n2, ep2)) => match (n1, n2) {
                (n1, _) if nmodid == n1 => ep1.send(envelope, target, exclude).await,
                (_, n2) if nmodid == n2 => ep2.send(envelope, target, exclude).await,
                _ => Err(RatmanError::Netmod(NetmodError::NotSupported)),
            },
            Self::Complete(map) => match map.get(nmodid) {
                Some(ep) => ep.send(envelope, target, exclude).await,
                None => Err(RatmanError::Netmod(NetmodError::NotSupported)),
            },
            _ => Err(RatmanError::Netmod(NetmodError::NotSupported)),
        }
    }

    ///
    pub async fn next(&self) -> Result<(InMemoryEnvelope, Neighbour)> {
        match self {
            Self::Single((_, ep)) => ep.next().await,
            Self::Binary((_, ep1), (_, ep2)) => tokio::select! {
                val = ep1.next() => val,
                val = ep2.next() => val,
            },
            Self::Complete(map) => map
                .values()
                .map(|ep| ep.next())
                .fold(FuturesUnordered::new(), |mut acc, f| {
                    acc.push(f);
                    acc
                })
                .next()
                .await
                .expect("Netmod stream has ended unexpectedly!"),
            _ => todo!(),
        }
    }
}
