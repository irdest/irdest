use crate::{proto, routes::Target};
use async_std::channel;
use async_std::{
    channel::{Receiver, Sender},
    net::{SocketAddr, TcpStream},
    sync::Arc,
};
use netmod::Frame;

/// Represent another node running netmod-inet
///
/// A peer is represented by a pair of socket addresses, and two
/// sockets.  A peer runs an incoming packet socket via `peer.run()`
/// and can send messages via `peer.send(...)`
///
/// There are two peering modes: `standard` and `cross`.  They specify
/// the way that connections are established, and how connection drops
/// are handled.
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
///    established by the remote.  In this model there is no "server"
///    and thus in the case of a connection drop, either side can
///    re-establish the connection without causing a race-condition.
///
/// The two inverse scenarios exist on the "server" side.
pub struct Peer {
    id: Target,
    src_addr: Option<SocketAddr>,
    dst_addr: Option<SocketAddr>,
    tx: Option<TcpStream>,
    rx: Option<TcpStream>,
    receiver: Sender<(Target, Frame)>,
}

impl Peer {
    /// Connect to a peer via "standard" connection
    pub fn connect_standard(
        id: Target,
        dst_addr: SocketAddr,
        stream: TcpStream,
    ) -> (Self, Receiver<(Target, Frame)>) {
        let (ftx, frx) = channel::bounded(64);
        (
            Self {
                id,
                src_addr: None, // irrelevant
                dst_addr: Some(dst_addr),
                tx: Some(stream.clone()),
                rx: Some(stream),
                receiver: ftx,
            },
            frx,
        )
    }

    /// Connect to a peer via "cross" connection
    pub fn connect_cross(
        id: Target,
        dst_addr: SocketAddr,
        tx: TcpStream,
    ) -> (Self, Receiver<(Target, Frame)>) {
        let (ftx, frx) = channel::bounded(64);
        (
            Self {
                id,
                src_addr: None, // will be filled in
                dst_addr: Some(dst_addr),
                tx: Some(tx),
                rx: None, // will be filled in
                receiver: ftx,
            },
            frx,
        )
    }

    /// Create a peer for an incoming standard connection
    pub fn incoming_standard(
        id: Target,
        src_addr: SocketAddr,
        stream: TcpStream,
    ) -> (Self, Receiver<(Target, Frame)>) {
        let (ftx, frx) = channel::bounded(64);
        (
            Self {
                id,
                src_addr: Some(src_addr),
                dst_addr: None,
                tx: Some(stream.clone()),
                rx: Some(stream),
                receiver: ftx,
            },
            frx,
        )
    }

    /// Create a peer for an incoming cross connection
    pub fn incoming_cross(
        id: Target,
        src_addr: SocketAddr,
        dst_addr: SocketAddr,
        tx: TcpStream,
        rx: TcpStream,
    ) -> (Self, Receiver<(Target, Frame)>) {
        let (ftx, frx) = channel::bounded(64);
        (
            Self {
                id,
                src_addr: Some(src_addr),
                dst_addr: Some(dst_addr),
                tx: Some(tx),
                rx: Some(rx),
                receiver: ftx,
            },
            frx,
        )
    }

    /// Spawn this function to receive messages from this peer
    pub async fn run(self: &Arc<Self>) {
        while let Some(ref rx) = self.rx {
            let _f: Frame = proto::read(rx).await.unwrap();
        }
    }
}
