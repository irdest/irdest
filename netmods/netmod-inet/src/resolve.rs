use async_std::net::*;

pub struct Resolver;

impl Resolver {
    /// Turn a peer line into a SocketAddr via magic
    pub(crate) async fn resolve(peer: &str) -> Option<SocketAddr> {
        let peer = if peer.ends_with("L") {
            &peer[0..peer.len()]
        } else {
            &peer[..]
        };

        match peer.parse().ok() {
            // First attempt to use it as a regular IP address string
            Some(s) => Some(s),
            // If we have a resolver, try to resolve this payload to
            // an IP address (splitting off the port)
            None => ToSocketAddrs::to_socket_addrs(peer)
                .await
                .ok()?
                .into_iter()
                .fold(None, |acc, addr| match (acc, addr) {
                    (None, addr) => Some(addr),
                    (_, maybe_v6) if maybe_v6.is_ipv6() => Some(maybe_v6),
                    (addr, _) => addr,
                }),
        }
    }
}
