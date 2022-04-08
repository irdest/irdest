use async_std::net::{SocketAddr, TcpStream};

/// Represent another node running netmod-inet
///
/// A peer is represented by a pair of socket addresses, and two
/// sockets.  A peer runs an incoming packet socket via `peer.run()`
/// and can send messages via `peer.send(...)`
///
/// When the connection drops the peer will automatically try to
/// reconnect, or wait for an incoming connection.  After a specified
/// timeout the peer is dropped.
///
/// ## Types of guys
///
/// 1. Peer is set to connect to a remote via standard connection
///
///    In this mode the peer creates a single outgoing connection, and
///    uses the same stream for sending and receiving messages.  When
///    the peer disconnects, it is responsible for re-connecting.  The
///    "server" will drop the peer and not re-connect (because it
///    doesn't know how).
///
///
/// 2. Peer is set to connect to a remote via cross connection
///
///    In this mode the peer creates a single outgoing connection, and
///    is upgraded with an incoming connection for receiving, which is
///    established by the remote.  In this model there is no "server".
///
/// 3. 
pub struct Peer {
    src_addr: SocketAddr,
    dst_addr: Option<SocketAddr>,
    tx: TcpStream,
    rx: TcpStream,
}

impl Peer {
    pub fn outgoing() -> Self {
        todo!()
    }
}
