//! Peer session management

use crate::{
    peer::{FrameReceiver, FrameSender, Peer},
    proto::{self, Handshake},
    routes::Target,
    PeerType,
};
use async_std::{
    net::{SocketAddr, TcpStream},
    sync::Arc,
    task,
};
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
    Handshake(SessionData, io::Error),
}

/// Outgoing session manager
///
/// This type is responsible for creating a handshake with a remote
/// peer and then creating a [`Peer`](crate::peer::Peer) type to
/// handle data.  When a data transfer fails, control is given back to
/// the `SessionManager` to re-establish the connection (hopefully).
pub(crate) struct SessionManager;

/// A convenient data struct to represent a session attempt
#[derive(Copy, Clone, Debug)]
pub(crate) struct SessionData {
    pub(crate) id: Target,
    pub(crate) tt: PeerType,
    pub(crate) addr: SocketAddr,
    pub(crate) self_port: u16,
}



impl SessionManager {
    /// Attempt to start a session with a peer
    ///
    /// When starting a `Standard` peer this session will never time-out
    /// and re-try forever (but with connection back-off).
    ///
    /// For a `Cross` peer it will give up after `CROSS_SESSION_TIMEOUT`
    pub(crate) async fn connect(
        ctr: &mut u16,
        SessionData { tt, addr, .. }: &SessionData,
    ) -> Result<TcpStream, SessionError> {
        let mut holdoff = 2; // in seconds
        loop {
            match TcpStream::connect(addr).await {
                Ok(c) => {
                    info!("Successfully peered with {}", addr);
                    return Ok(c);
                }
                Err(_) => {
                    error!("Failed peering with {} [attempt {}]", addr, ctr);
                    task::sleep(Duration::from_secs(holdoff)).await;
                    *ctr += 1;
                }
            }

            match tt {
                // For cross-connections we eventually give up
                PeerType::Cross if *ctr >= SESSION_TIMEOUT => {
                    break Err(SessionError::Refused(*addr, *ctr))
                }
                // For standard connections we just slow down our attempts up to ~69 minutes
                PeerType::Standard if *ctr >= SESSION_TIMEOUT && holdoff < 4096 => holdoff *= 2,
                // Limited connections are not implemented yet
                PeerType::Limited(_) => {
                    error!("APOLOGIES this feature is not yet implemented, despite what the documentation tells you");
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
    pub(crate) async fn handshake(
        data: &SessionData,
        sender: FrameSender,
        mut stream: TcpStream,
    ) -> Result<Arc<Peer>, SessionError> {
        proto::write(
            &mut stream,
            &Handshake::Hello {
                tt: data.tt,
                self_port: 0,
            },
        )
        .await
        .map_err(|e| SessionError::Handshake(*data, e))?;

        let ack: Handshake = proto::read_blocking(&mut stream)
            .await
            .map_err(|e| SessionError::Handshake(*data, e))?;

        match (data.tt, ack) {
            (outgoing, Handshake::Ack { tt }) if outgoing == tt => {
                debug!("Received valid ACK session data");
            }
            _ => {
                error!("Received invalid ACK session data");
                drop(stream);
                return Err(SessionError::Dropped(data.addr));
            }
        }

        Ok(Peer::standard(*data, sender, stream))
    }
}
