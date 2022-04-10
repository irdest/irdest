use async_std::net::TcpListener;

 

/// Tcp connection listener taking on connections from peers,
/// configuring links, and spawning async peer handlers
pub struct Server {
    ipv4_listen: Option<TcpListener>,
    ipv6_listen: TcpListener,
}
