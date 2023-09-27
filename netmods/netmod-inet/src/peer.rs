// SPDX-FileCopyrightText: 2019-2022 Katharina Fey <kookie@spacekookie.de>
//
// SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

use crate::proto::ProtoError;
use crate::session::{SessionData, SessionError};
use crate::{proto, routes::Target};
use async_std::channel;
use async_std::task::JoinHandle;
use async_std::{
    channel::{Receiver, Sender},
    net::{SocketAddr, TcpStream},
    sync::{Arc, Mutex},
    task,
};
use libratman::netmod::InMemoryEnvelope;

pub(crate) type FrameReceiver = Receiver<(Target, InMemoryEnvelope)>;
pub(crate) type FrameSender = Sender<(Target, InMemoryEnvelope)>;

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
    restart: Option<Sender<SessionData>>,
}

impl Peer {
    /// Connect to a peer via "standard" connection
    pub(crate) fn standard(
        session: SessionData,
        receiver: FrameSender,
        restart: Option<Sender<SessionData>>,
        stream: TcpStream,
    ) -> Arc<Self> {
        Arc::new(Self {
            session,
            tx: Mutex::new(Some(stream.clone())),
            rx: Mutex::new(Some(stream)),
            receiver,
            restart,
        })
    }

    /// Return this Peer's ID
    #[inline]
    pub(crate) fn id(&self) -> Target {
        self.session.id
    }

    /// Send a frame to this peer
    ///
    /// If the sending fails for any reason, the underlying
    /// `SessionData` is returned so that a new session may be
    /// started.
    pub(crate) async fn send(self: &Arc<Self>, env: &InMemoryEnvelope) -> Result<(), SessionError> {
        let mut txg = self.tx.lock().await;

        // The TcpStream SHOULD never just disappear
        let tx = txg.as_mut().unwrap();

        trace!("Writing data to stream {}", self.id());
        match proto::write(&mut *tx, env).await {
            Ok(()) => Ok(()),
            Err(e) => {
                warn!("Failed to send data for peer {}", self.session.id);

                // If we are the outgoing side we signal to be restarted
                if let Some(ref tx) = self.restart {
                    tx.send(self.session).await;
                    debug!("Notify restart hook");
                    Ok(())
                }
                // Else we just inform the sending context that this
                // has failed.  On the server side we then remove this
                // peer from the routing table and insert a temp
                // buffer instead.
                else {
                    Err(SessionError::Dropped(self.session.addr))
                }
            }
        }
    }

    /// Repeatedly attempt to read from the reading socket
    pub(crate) async fn run(self: Arc<Self>) {
        loop {
            let mut rxg = self.rx.lock().await;
            let rx = match rxg.as_mut() {
                Some(rx) => rx,
                None => break,
            };

            let envelope = match proto::read(rx).await {
                Ok(f) => {
                    trace!("Received frame from stream {}", self.id());
                    f
                }
                Err(ProtoError::NoData) => {
                    drop(rxg);
                    task::yield_now();
                    continue;
                }
                Err(ProtoError::Io(io)) => {
                    error!(
                        "Peers {} encountered I/O error during receiving: {}",
                        self.id(),
                        io
                    );

                    // If we were the outgoing peer we signal to re-connect
                    if let Some(ref tx) = self.restart {
                        tx.send(self.session).await;
                    }

                    break;
                }
            };

            // If we received a correct frame we forward it to the receiver
            self.receiver.send((self.session.id, envelope)).await;
        }

        trace!("Exit receive loop for peer {}", self.id());
    }
}
