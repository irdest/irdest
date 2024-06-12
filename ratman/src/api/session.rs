use crate::{context::RatmanContext, crypto, procedures::handle_subscription_socket};
use libratman::{
    api::{
        socket_v2::RawSocketHandle,
        types::{
            AddrCreate, AddrDestroy, AddrDown, AddrUp, Handshake, ServerPing, SubsCreate,
            SubsDelete, SubsRestore,
        },
        version_str, versions_compatible,
    },
    frame::micro::{client_modes as cm, MicroframeHeader},
    tokio::{
        io::ErrorKind,
        net::{TcpListener, TcpStream},
        task::spawn_local,
        time::timeout,
    },
    types::{AddrAuth, Address, Ident32},
    ClientError, RatmanError, Result,
};
use std::{collections::BTreeMap, ffi::CString, sync::Arc, time::Duration};

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

fn check_auth(
    header: &MicroframeHeader,
    address: Address,
    expected_auth: &mut BTreeMap<AddrAuth, Address>,
) -> Result<AddrAuth> {
    header
        .auth
        .ok_or(RatmanError::ClientApi(ClientError::InvalidAuth))
        .and_then(|given_auth| match expected_auth.get(&given_auth) {
            Some(addr) if addr == &address => Ok(given_auth),
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
    expected_auth: &mut BTreeMap<AddrAuth, Address>,
    raw_socket: &mut RawSocketHandle,
) -> Result<SessionResult> {
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
            raw_socket
                .write_microframe(
                    MicroframeHeader::intrinsic_noauth(),
                    ServerPing::Error(ClientError::Internal(e.to_string())),
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
        //
        //
        // ;-; Client is breaking up with us, but at least it's not just a text
        m if m == cm::make(cm::INTRINSIC, cm::DOWN) => {
            debug!("Client {client_id} disconnecting gracefully");
            return Ok(SessionResult::Drop);
        }
        //
        //
        // ^-^ Creating a new address key and client auth token
        m if m == cm::make(cm::ADDR, cm::CREATE) => {
            let _payload = raw_socket
                .read_payload::<AddrCreate>(header.payload_size)
                .await??;
            let (addr, client_auth) = crypto::insert_addr_key(&ctx.meta_db)?;

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
        //
        //
        // ^-^ Destroy an existing address
        m if m == cm::make(cm::ADDR, cm::DESTROY) => {
            let AddrDestroy { addr, force: _ } = raw_socket
                .read_payload::<AddrDestroy>(header.payload_size)
                .await??;

            let auth = check_auth(&header, addr, expected_auth)?;

            crypto::destroy_addr_key(&ctx.meta_db, addr, auth, client_id)?;

            reply_ok(raw_socket, auth).await?;
        }
        //
        //
        // ^-^ Mark an existing address as "up" given the correct authentication
        m if m == cm::make(cm::ADDR, cm::UP) => {
            let addr_up = raw_socket
                .read_payload::<AddrUp>(header.payload_size)
                .await??;

            // Use the provided auth to open the stored address key.  If this
            // works then we store the provided authentication object in
            // "expected auth"
            crypto::open_addr_key(
                &ctx.meta_db,
                addr_up.addr,
                header
                    .auth
                    .ok_or(RatmanError::ClientApi(ClientError::InvalidAuth))?,
                client_id,
            )?;

            let auth = check_auth(&header, addr_up.addr, expected_auth)?;

            Arc::clone(&ctx.protocol)
                .online(addr_up.addr, auth, client_id, Arc::clone(&ctx))
                .await?;

            reply_ok(raw_socket, auth).await?;
        }
        //
        //
        // ^-^ Mark an address as "down" which is currently "up"
        m if m == cm::make(cm::ADDR, cm::DOWN) => {
            let addr_down = raw_socket
                .read_payload::<AddrDown>(header.payload_size)
                .await??;

            let auth = check_auth(&header, addr_down.addr, expected_auth)?;

            ctx.protocol.offline(addr_down.addr).await?;
            expected_auth.remove(&auth);

            reply_ok(raw_socket, auth).await?;
        }
        //
        //
        // ^-^ Create a new subscription
        m if m == cm::make(cm::SUB, cm::CREATE) => {
            let subs_create = raw_socket
                .read_payload::<SubsCreate>(header.payload_size)
                .await?;

            let auth = check_auth(&header, subs_create.addr, expected_auth)?;

            let (sub_id, rx) = ctx
                .subs
                .create_subscription(subs_create.addr, subs_create.recipient)
                .await?;

            let sub_listen = TcpListener::bind("127.0.0.1:0").await?;
            let bind = sub_listen.local_addr()?.to_string();

            let stream_ctx = Arc::clone(ctx);
            spawn_local(async move {
                if let Ok((stream, _)) = sub_listen.accept().await {
                    let raw_socket = RawSocketHandle::new(stream);
                    handle_subscription_socket(stream_ctx, rx, raw_socket, auth, sub_id).await;
                }
            });

            raw_socket
                .write_microframe(
                    MicroframeHeader::intrinsic_auth(auth),
                    ServerPing::Subscription {
                        sub_id,
                        sub_bind: CString::new(bind).unwrap(),
                    },
                )
                .await?;
        }
        //
        //
        // ^-^ Destroy an existing subscription
        m if m == cm::make(cm::SUB, cm::DELETE) => {
            let subs_delete = raw_socket
                .read_payload::<SubsDelete>(header.payload_size)
                .await?;

            let auth = check_auth(&header, subs_delete.addr, expected_auth)?;

            ctx.subs
                .delete_subscription(subs_delete.addr, subs_delete.sub_id)
                .await?;

            reply_ok(raw_socket, auth).await?;
        }
        //
        //
        // ^-^ List available subscriptions
        m if m == cm::make(cm::SUB, cm::LIST) => {}
        //
        //
        // ^-^ Restore an existing subscription
        m if m == cm::make(cm::SUB, cm::UP) => {
            let subs_restore = raw_socket
                .read_payload::<SubsRestore>(header.payload_size)
                .await?;

            let auth = check_auth(&header, subs_restore.addr, expected_auth)?;

            let rx = ctx
                .subs
                .restore_subscription(subs_restore.addr, subs_restore.sub_id)
                .await?;

            // todo: spawn receiver task/ socket
            // todo: return the correct type
        }
        //
        //
        // ^-^ Client wants to receive exactly one message
        m if m == cm::make(cm::RECV, cm::ONE) => {}
        //
        //
        // ^-^ Client wants to listen for messages in this session
        m if m == cm::make(cm::RECV, cm::MANY) => {}
        //
        //
        // ^-^ Client wants to send a message to one recipient
        m if m == cm::make(cm::SEND, cm::ONE) => {}
        //
        //
        // ^-^ Client wants to send a message to many recipients
        m if m == cm::make(cm::SEND, cm::MANY) => {}
        //
        //
        // u-u Don't know what to do with this
        mode => {
            raw_socket
                .write_microframe(
                    MicroframeHeader::intrinsic_noauth(),
                    ServerPing::Error(ClientError::Internal(format!("Invalid frame mode: {mode}"))),
                )
                .await?;

            return Ok(SessionResult::Next);
        }
    }

    Ok(SessionResult::Next)
}
