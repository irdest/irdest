use crate::context::RatmanContext;
use libratman::{
    api::{
        socket_v2::RawSocketHandle,
        types::{AddrCreate, AddrDestroy, AddrDown, AddrUp, Handshake, ServerPing},
        version_str, versions_compatible,
    },
    frame::micro::{client_modes as cm, MicroframeHeader},
    tokio::{net::TcpStream, task::spawn_local, time::timeout},
    types::{ClientAuth, Id},
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

fn check_auth(header: &MicroframeHeader, expected_auth: Option<ClientAuth>) -> Result<ClientAuth> {
    header
        .auth
        .ok_or(RatmanError::ClientApi(ClientError::InvalidAuth))
        .and_then(|given_auth| match expected_auth {
            Some(expected_auth) if given_auth == expected_auth => Ok(given_auth),
            _ => Err(RatmanError::ClientApi(ClientError::InvalidAuth)),
        })
}

async fn reply_ok(raw_socket: &mut RawSocketHandle, auth: ClientAuth) -> Result<()> {
    raw_socket
        .write_microframe(MicroframeHeader::intrinsic_auth(auth), ServerPing::Ok)
        .await
}

pub(super) async fn single_session_exchange(
    ctx: &Arc<RatmanContext>,
    client_id: Id,
    expected_auth: &mut Option<ClientAuth>,
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
        // ^-^ Creating a new address key and client auth token
        m if m == cm::make(cm::ADDR, cm::CREATE) => {
            let _payload = raw_socket
                .read_payload::<AddrCreate>(header.payload_size)
                .await?;
            let (addr, client_auth) = ctx.meta_db.insert_addr_key(client_id)?;

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
