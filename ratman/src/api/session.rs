use crate::context::RatmanContext;
use libratman::tokio::{net::TcpStream, time::timeout};
use libratman::{
    api::{
        socket_v2::RawSocketHandle,
        types::{Handshake, ServerPing},
        version_str, versions_compatible,
    },
    frame::micro::{client_modes as cm, MicroframeHeader},
    types::Id,
    ClientError, Result,
};
use std::ffi::CString;
use std::{sync::Arc, time::Duration};

/// Initiate a new client connection
pub(super) async fn handshake(stream: TcpStream) -> Result<RawSocketHandle> {
    // Wrap the TcpStream to bring its API into scope
    let mut raw_socket = RawSocketHandle::new(stream);

    // Read the client handshake to determine whether we are compatible
    let (_header, handshake) = raw_socket.read_microframe::<Handshake>().await?;
    let compatible = versions_compatible(libratman::api::VERSION, handshake.proto_version);

    // Reject connection and disconnect
    if !compatible {
        raw_socket
            .write_buffer(libratman::api::VERSION.to_vec())
            .await?;

        return Err(ClientError::IncompatibleVersion(format!(
            "self:{},client:{}",
            version_str(&libratman::api::VERSION),
            version_str(&handshake.proto_version)
        ))
        .into());
    }

    Ok(raw_socket)
}

pub(super) enum SessionResult {
    Next,
    Drop,
}

pub(super) async fn single_session_exchange(
    ctx: &Arc<RatmanContext>,
    raw_socket: &mut RawSocketHandle,
) -> Result<SessionResult> {
    // a) Send a ping to initiate a response
    // b) Wait for a command response
    // c) Timeout connection in case of no reply

    raw_socket
        .write_microframe(
            MicroframeHeader::intrinsic_noauth(),
            ServerPing::Update {
                available_subscriptions: vec![],
            },
        )
        .await?;

    let header = match timeout(Duration::from_secs(30), raw_socket.read_header()).await {
        Ok(Ok(header)) => header,
        Ok(Err(e)) => {
            warn!("Client sent invalid payload: {e:?}");
            let encoded = CString::new(e.to_string().as_bytes()).expect("failed to encode CString");
            raw_socket
                .write_microframe(
                    MicroframeHeader::intrinsic_noauth(),
                    ServerPing::Error(encoded),
                )
                .await?;

            // The client sent us rubbish but it won't phase us
            return Ok(SessionResult::Drop);
        }
        Err(Elapsed) => {
            info!("Connection X timed out!");
            // We ignore the error here in case the timeout send fails
            let _ = raw_socket
                .write_microframe(MicroframeHeader::intrinsic_noauth(), ServerPing::Timeout)
                .await;

            // The client stood us up but that doesn't mean we're in trouble
            return Ok(SessionResult::Drop);
        }
    };

    ////////////////////////////////////////////////

    match header.modes {
        // Creating a new
        m if m == cm::make(cm::ADDR, cm::CREATE) => {
            let (addr, client_auth) = ctx.meta_db.insert_addr_key(Id::random())?;
        }

        mode => {
            raw_socket
                .write_microframe(
                    MicroframeHeader::intrinsic_noauth(),
                    ServerPing::Error(
                        CString::new(format!("Invalid mode provided: {}", mode)).unwrap(),
                    ),
                )
                .await;

            return Ok(SessionResult::Next);
        }
    }

    Ok(SessionResult::Next)
}
