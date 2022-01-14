use async_std_resolver::{config, resolver, AsyncStdResolver};
use std::net::*;

pub struct Resolver {
    inner: Option<AsyncStdResolver>,
}

impl Resolver {
    pub(crate) async fn new() -> Self {
        Self {
            inner: resolver(
                config::ResolverConfig::default(),
                config::ResolverOpts::default(),
            )
            .await
            .ok(),
        }
    }

    /// Turn a peer line into a SocketAddr via magic
    pub(crate) async fn resolve_peer(&self, peer: &str) -> Option<SocketAddr> {
        match peer.parse().ok() {
            // First attempt to use it as a regular IP address string
            Some(s) => Some(s),
            // If we have a resolver, try to resolve this payload to
            // an IP address (splitting off the port)
            None if self.inner.is_some() => {
                let split: Vec<_> = peer.split(":").collect();
                let maybe_domain_string = split.get(0).map(|s| s.to_string())?;
                trace!("Looking up `{}`...", maybe_domain_string);
                self.inner
                    .as_ref()
                    .unwrap()
                    .lookup_ip(maybe_domain_string)
                    .await
                    .ok()?
                    .iter()
                    .fold(None, |acc, addr| match (acc, addr) {
                        (None, addr) => Some(addr),
                        (_, maybe_v6) if maybe_v6.is_ipv6() => Some(maybe_v6),
                        (addr, _) => addr,
                    })
                    .and_then(|ip| {
                        split
                            .get(1)
                            .and_then(|port| port.parse().ok())
                            .map(|port| SocketAddr::new(ip, port))
                    })
            }
            // Otherwise log something about it and return None
            None => {
                debug!("No resolving mechanisms configured");
                None
            }
        }
    }
}
