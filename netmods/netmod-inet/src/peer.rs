use crate::proto::ProtoError;
use crate::session::SessionData;
use crate::{proto, routes::Target};
use async_std::channel;
use async_std::{
    channel::{Receiver, Sender},
    net::{SocketAddr, TcpStream},
    sync::{Arc, Mutex},
    task,
};
use netmod::Frame;

pub(crate) type FrameReceiver = Receiver<(Target, Frame)>;
pub(crate) type FrameSender = Sender<(Target, Frame)>;

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
    session: SessionData,
    tx: Mutex<Option<TcpStream>>,
    rx: Mutex<Option<TcpStream>>,
    receiver: FrameSender,
}

impl Peer {
    /// Connect to a peer via "standard" connection
    pub(crate) fn outgoing_standard(
        session: SessionData,
        receiver: FrameSender,
        stream: TcpStream,
    ) -> Arc<Self> {
        Arc::new(Self {
            session,
            tx: Mutex::new(Some(stream.clone())),
            rx: Mutex::new(Some(stream)),
            receiver,
        })
    }

    /// Send a frame to this peer
    ///
    /// If the sending fails for any reason, the underlying
    /// `SessionData` is returned so that a new session may be
    /// started.
    pub(crate) async fn send(self: &Arc<Self>, f: &Frame) -> Result<(), SessionData> {
        let mut txg = self.tx.lock().await;
        let tx = txg.as_mut().ok_or(self.session)?;
        proto::write(&mut *tx, f).await.map_err(|_| self.session)?;
        Ok(())
    }

    /// Repeatedly attempt to read from the reading socket
    pub(crate) async fn run(self: &Arc<Self>) {
        loop {
            let mut rxg = self.rx.lock().await;
            let rx = match rxg.as_mut() {
                Some(rx) => rx,
                None => break,
            };

            let f: Frame = match proto::read(rx).await {
                Ok(f) => f,
                Err(ProtoError::NoData) => {
                    drop(rxg);
                    task::yield_now();
                    continue;
                }
                Err(ProtoError::Io(io)) => {
                    error!("Encountered I/O error during receiving: {}", io);
                    break;
                }
            };

            // If we received a correct frame we forward it to the receiver
            self.receiver.send((self.session.id, f)).await;
        }
    }
}
