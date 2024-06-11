use crate::{context::RatmanContext, crypto};
use libratman::{
    api::{
        socket_v2::RawSocketHandle,
        types::{AddrCreate, AddrDestroy, AddrDown, AddrUp, Handshake, ServerPing},
        version_str, versions_compatible,
    },
    frame::micro::{client_modes as cm, MicroframeHeader},
    tokio::{io::ErrorKind, net::TcpStream, time::timeout},
    types::{AddrAuth, Ident32},
    ClientError, RatmanError, Result,
};
use std::ffi::CString;
use std::{sync::Arc, time::Duration};

/// Initiate a new client connection
pub(super) async fn handshake(stream: TcpStream) -> Result<RawSocketHandle> {
    // Wrap the TcpStream to bring its API into scope
    let mut raw_socket = RawSocketHandle::new(stream);

    // Read the client handshake to determine whether we are compatible
    let (_header, handshake) = raw_socket.read_microframe::<Handshake>().await?;
    let compatible = versions_compatible(libratman::api::VERSION, handshake.client_version);

    // Reject connection and disconnect
    if compatible {
        raw_socket
            .write_microframe(MicroframeHeader::intrinsic_noauth(), ServerPing::Ok)
            .await?;
    } else {
        let router = version_str(&libratman::api::VERSION);
        let client = version_str(&handshake.client_version);

        raw_socket
            .write_microframe(
                MicroframeHeader::intrinsic_noauth(),
                ServerPing::IncompatibleVersion {
                    router: CString::new(router).unwrap(),
                    client: CString::new(client).unwrap(),
                },
            )
            .await?;

        return Err(ClientError::IncompatibleVersion(
            version_str(&libratman::api::VERSION),
            version_str(&handshake.client_version),
        )
        .into());
    }

    Ok(raw_socket)
}

pub(super) enum SessionResult {
    Next,
    Drop,
}

fn check_auth(header: &MicroframeHeader, expected_auth: Option<AddrAuth>) -> Result<AddrAuth> {
    header
        .auth
        .ok_or(RatmanError::ClientApi(ClientError::InvalidAuth))
        .and_then(|given_auth| match expected_auth {
            Some(expected_auth) if given_auth == expected_auth => Ok(given_auth),
            _ => Err(RatmanError::ClientApi(ClientError::InvalidAuth)),
        })
}

async fn reply_ok(raw_socket: &mut RawSocketHandle, auth: AddrAuth) -> Result<()> {
    raw_socket
        .write_microframe(MicroframeHeader::intrinsic_auth(auth), ServerPing::Ok)
        .await
}

pub(super) async fn single_session_exchange(
    ctx: &Arc<RatmanContext>,
    client_id: Ident32,
    expected_auth: &mut Option<AddrAuth>,
    raw_socket: &mut RawSocketHandle,
) -> Result<SessionResult> {
    // a) Send a ping to initiate a response
    // b) Wait for a command response
    // c) Timeout connection in case of no reply

    // raw_socket
    //     .write_microframe(MicroframeHeader::intrinsic_noauth(), ServerPing::Poke)
    //     .await?;

    let header = match timeout(
        Duration::from_secs(/* 2 minute timeout */ 2 * 60),
        raw_socket.read_header(),
    )
    .await
    {
        Ok(Ok(header)) => header,
        // Handle EOF errors explicitly to orderly shut down this thing
        Ok(Err(RatmanError::TokioIo(err))) if err.kind() == ErrorKind::UnexpectedEof => {
            MicroframeHeader {
                modes: cm::make(cm::INTRINSIC, cm::DOWN),
                auth: None,
                payload_size: 0,
            }
        }
        // Every other error can be logged properly
        Ok(Err(e)) => {
            debug!("Failed to read from socket: {e:?}");
            let encoded = CString::new(e.to_string().as_bytes()).expect("failed to encode CString");
            raw_socket
                .write_microframe(
                    MicroframeHeader::intrinsic_noauth(),
                    ServerPing::Error(encoded),
                )
                .await?;

            // This session ended unexpectedly
            return Err(e);
        }
        Err(_) => {
            info!("Connection {client_id} timed out!");
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
        m if m == cm::make(cm::INTRINSIC, cm::DOWN) => {
            info!("Client {client_id} disconnecting");
            return Ok(SessionResult::Drop);
        }

        // ^-^ Creating a new address key and client auth token
        m if m == cm::make(cm::ADDR, cm::CREATE) => {
            let _payload = raw_socket
                .read_payload::<AddrCreate>(header.payload_size)
                .await?;
            let (addr, client_auth) = crypto::insert_addr_key(&ctx.meta_db, client_id)?;

            ctx.clients
                .lock()
                .await
                .get_mut(&client_id)
                .unwrap()
                .add_address(addr);

            raw_socket
                .write_microframe(MicroframeHeader::intrinsic_auth(client_auth), addr)
                .await?;
        }
        // ^-^ Destroy an existing address
        m if m == cm::make(cm::ADDR, cm::DESTROY) => {
            let _payload = raw_socket
                .read_payload::<AddrDestroy>(header.payload_size)
                .await?;
        }
        // ^-^ Mark an existing address as "up" given the correct authentication
        m if m == cm::make(cm::ADDR, cm::UP) => {
            let addr_up = raw_socket
                .read_payload::<AddrUp>(header.payload_size)
                .await??;

            // Check if the given auth is valid for this session
            let auth = check_auth(&header, *expected_auth)?;

            Arc::clone(&ctx.protocol)
                .online(addr_up.addr, auth, Arc::clone(&ctx))
                .await?;

            reply_ok(raw_socket, auth).await?;
        }
        // ^-^ Mark an address as "down" which is currently "up"
        m if m == cm::make(cm::ADDR, cm::DOWN) => {
            let addr_down = raw_socket
                .read_payload::<AddrDown>(header.payload_size)
                .await??;

            let auth = check_auth(&header, *expected_auth)?;
            ctx.protocol.offline(addr_down.addr).await?;
            reply_ok(raw_socket, auth).await?;
        }
        // u-u Don't know what to do with this
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
