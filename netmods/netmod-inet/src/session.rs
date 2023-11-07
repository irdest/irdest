//! Peer session management

use crate::{
    peer::{FrameReceiver, FrameSender, Peer},
    proto::{self, Handshake},
    routes::{Routes, Target},
    PeerType,
};
use async_std::{
    channel::{unbounded, Receiver, Sender},
    net::{SocketAddr, TcpStream},
    sync::Arc,
    task,
};
use libratman::{netmod::InMemoryEnvelope, RatmanError};
use std::{io, time::Duration};

/// The number of attempts a session maskes to a peer before giving up
pub const SESSION_TIMEOUT: u16 = 6;

#[derive(Debug, thiserror::Error)]
pub(crate) enum SessionError {
    #[error("connection to {} refused (after {} tries)", 0, 1)]
    Refused(SocketAddr, u16),
    #[error("existing connection to {} was dropped by peer", 0)]
    Dropped(SocketAddr),
    #[error("mismatched peering expectations with {:?}: {}", 0, 1)]
    Handshake(SessionData, String),
}

/// Create a new session manager for a single peer
///
/// It will re-attempt to establish a connection until one is found.
/// It then adds the newly created peer to the routing table.
///
/// The running task then shuts down.  In case of connection drop,
/// call `cleanup_connection`
pub(crate) async fn start_connection(
    session_data: SessionData,
    routes: Arc<Routes>,
    sender: FrameSender,
) -> Result<Receiver<SessionData>, SessionError> {
    let (tx, rx) = unbounded();

    let routes2 = Arc::clone(&routes);
    let sender2 = sender.clone();
    task::spawn(async move {
        let tcp_stream = match connect(&session_data).await {
            Ok(tcp) => tcp,
            Err(e) => {
                error!("failed to establish session: {}", e);
                todo!()
                //            return;
            }
        };

        let peer = match handshake(&session_data, sender2, tx, tcp_stream).await {
            Ok(peer) => peer,
            Err(e) => {
                error!("peering handshake failed: {}", e);
                todo!()
                //            return;
            }
        };

        task::spawn(Arc::clone(&peer).run());
        routes2.add_peer(peer.id(), Arc::clone(&peer)).await;
    });

    Ok(rx)
}

pub(crate) async fn setup_cleanuptask(
    rx: Receiver<SessionData>,
    sender: FrameSender,
    routes: &Arc<Routes>,
) {
    // We wait to be notified either by the peer itself, or the
    // sending context that the peer has died and needs to be
    // restarted.  Thus we call `cleanup_connection` to restart
    // the whole thing.
    //
    // For peers of incoming connections this sender is None, and
    // thus this code will never be run on a server.  Woops :)
    let routes = Arc::clone(&routes);
    task::spawn(async move {
        debug!("setup_cleanuptask spawned");
        match rx.recv().await {
            Ok(session_data) => {
                debug!("Restart hook notified!");

                if let Err(e) = cleanup_connection(session_data, &routes, sender).await {
                    error!(
                        "Failed to re-establish connection for peer {}, cause: {}",
                        session_data.id, e
                    )
                }
            }
            _ => {}
        }
    });
}

pub(crate) async fn cleanup_connection(
    session_data: SessionData,
    routes: &Arc<Routes>,
    sender: FrameSender,
) -> Result<(), SessionError> {
    let peer = routes.remove_peer(session_data.id).await;
    debug!("References to PEER left: {}", Arc::strong_count(&peer));

    start_connection(session_data, Arc::clone(&routes), sender).await?;
    Ok(())
}

/// A convenient data struct to represent a session attempt
#[derive(Copy, Clone, Debug)]
pub(crate) struct SessionData {
    pub(crate) id: Target,
    pub(crate) tt: PeerType,
    pub(crate) addr: SocketAddr,
    pub(crate) self_port: u16,
}

/// Attempt to start a session with a peer
///
/// When starting a `Standard` peer this session will never time-out
/// and re-try forever (but with connection back-off).
///
/// For a `Cross` peer it will give up after `CROSS_SESSION_TIMEOUT`
pub(crate) async fn connect(
    SessionData { tt, addr, .. }: &SessionData,
) -> Result<TcpStream, SessionError> {
    let mut holdoff = 2; // in seconds
    let mut ctr = 0;
    loop {
        match TcpStream::connect(addr).await {
            Ok(c) => {
                info!("Successfully connected to {}", addr);
                return Ok(c);
            }
            Err(_) => {
                error!("Failed connecting to {} [attempt {}]", addr, ctr);
                task::sleep(Duration::from_secs(holdoff)).await;
                ctr += 1;
            }
        }

        match tt {
            // For cross-connections we eventually give up
            PeerType::Cross if ctr >= SESSION_TIMEOUT => {
                break Err(SessionError::Refused(*addr, ctr));
            }
            // For standard connections we just slow down our attempts up to ~69 minutes
            PeerType::Standard if ctr >= SESSION_TIMEOUT && holdoff < 4096 => holdoff *= 2,
            // Limited connections are not implemented yet
            PeerType::Limited(_) => {
                error!("APOLOGIES this feature is not yet implemented, despite what the documentation tells you >:(");
                todo!()
            }
            // The match block does nothing
            _ => {}
        }
    }
}

/// Establish the correct type of connection with the peer
///
/// ## Handshake procedure
///
/// To avoid spreading the documentation for this too thin (TODO:
/// write a manual or something), here is an outline of what needs
/// to happen.
///
/// We have just created a connection to a peer.  Now we need to
/// send a HANDSHAKE packet, letting the peer know who we are and what
/// we want.  This includes the PeerType, our own listening port,
/// and whether we are into dynimac peering or not (not used in
/// this version yet).
///
/// If anything goes wrong during the handshake we close the
/// connection again, and re-try to connect from the beginning.
async fn handshake(
    data: &SessionData,
    sender: FrameSender,
    restart: Sender<SessionData>,
    mut stream: TcpStream,
) -> Result<Arc<Peer>, SessionError> {
    let hello = Handshake::Hello {
        tt: data.tt,
        self_port: 0,
    }
    .to_carrier()
    .unwrap();

    proto::write(&mut stream, &hello)
        .await
        .map_err(|e| SessionError::Handshake(*data, e.to_string()))
        .unwrap();

    let ack_env: InMemoryEnvelope = proto::read_blocking(&mut stream)
        .await
        .map_err(|e| SessionError::Handshake(*data, e.to_string()))
        .unwrap();

    let ack = match Handshake::from_carrier(&ack_env) {
        Err(RatmanError::Nonfatal(nf)) => {
            warn!("Expected to receive a Handshake::Ack but received something different!");
            unreachable!();
        }
        Ok(ack) => ack,
        Err(e) => return Err(SessionError::Handshake(*data, e.to_string())),
    };

    // ??? what does this match block actually do
    match (data.tt, ack) {
        (outgoing, Handshake::Ack { tt }) if outgoing == tt => {
            debug!("Handshake with {:?} was successful!", stream.peer_addr());
        }
        _ => {
            error!("Handshake with {:?} was unsuccessful", stream.peer_addr());
            drop(stream);
            return Err(SessionError::Dropped(data.addr));
        }
    }

    Ok(Peer::standard(*data, sender, Some(restart), stream))
}
