// SPDX-FileCopyrightText: 2023-2024 Katharina Fey <kookie@spacekookie.de>
//
// SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

use crate::{
    api::send_util::exec_send_many_socket,
    context::RatmanContext,
    crypto,
    procedures::{handle_subscription_socket, SenderSystem},
};
use libratman::{
    api::{
        socket_v2::RawSocketHandle,
        types::{
            AddrCreate, AddrDestroy, AddrDown, AddrList, AddrUp, AnycastProbe, Handshake,
            NamespaceDown, NamespaceRegister, NamespaceUp, PeerList, RecvMany, RecvOne, SendMany,
            SendOne, ServerPing, SubsCreate, SubsDelete, SubsRestore,
        },
        version_str, versions_compatible,
    },
    frame::micro::{client_modes as cm, MicroframeHeader},
    tokio::{
        io::ErrorKind,
        net::{TcpListener, TcpStream},
        sync::broadcast::channel as bcast_channel,
        task::spawn,
        time::timeout,
    },
    types::{error::UserError, AddrAuth, Address, Ident32},
    ClientError, RatmanError, Result,
};
use std::{ffi::CString, sync::Arc, time::Duration};

use super::clients::AuthGuard;

/// Initiate a new client connection
pub(super) async fn handshake(stream: TcpStream) -> Result<RawSocketHandle> {
    // Wrap the TcpStream to bring its API into scope
    let mut raw_socket = RawSocketHandle::new(stream);

    // Read the client handshake to determine whether we are compatible
    let (_header, handshake) = raw_socket.read_microframe::<Handshake>().await?;
    let compatible = versions_compatible(libratman::api::VERSION, handshake.client_version);

    // Reject connection and disconnect
    if compatible {
        info!("Sending OK message to client");
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

async fn check_auth<'a>(
    header: &MicroframeHeader,
    address: Address,
    expected_auth: &AuthGuard,
) -> Result<AddrAuth> {
    let auth = expected_auth.lock().await;
    header
        .auth
        .ok_or_else(|| RatmanError::ClientApi(ClientError::InvalidAuth))
        .and_then(|given_auth| match auth.get(&given_auth) {
            Some(addr) if addr == &address => Ok(given_auth),
            _ => Err(RatmanError::ClientApi(ClientError::InvalidAuth)),
        })
}

async fn reply_ok(raw_socket: &mut RawSocketHandle, auth: AddrAuth) -> Result<()> {
    raw_socket
        .write_microframe(MicroframeHeader::intrinsic_auth(auth), ServerPing::Ok)
        .await
}

pub(super) async fn single_session_exchange<'a>(
    ctx: &Arc<RatmanContext>,
    client_id: Ident32,
    auth_guard: &AuthGuard,
    raw_socket: &mut RawSocketHandle,
    senders: &Arc<SenderSystem>,
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
            info!("Connection {} timed out!", client_id.pretty_string());
            // We ignore the error here in case the timeout send fails
            let _ = raw_socket
                .write_microframe(MicroframeHeader::intrinsic_noauth(), ServerPing::Timeout)
                .await;

            // The client stood us up but that doesn't mean we're in trouble
            return Ok(SessionResult::Drop);
        }
    };

    ////////////////////////////////////////////////

    trace!(
        "given_auth: {} // auth_guard={:?}",
        header
            .auth
            .map(|auth| auth.token.pretty_string())
            .unwrap_or_else(|| "None".to_string()),
        auth_guard
            .lock()
            .await
            .iter()
            .map(|(k, v)| (k.token.pretty_string(), v.pretty_string()))
            .collect::<Vec<_>>()
    );

    match header.modes {
        //
        //
        // ;-; Client is breaking up with us, but at least it's not just a text
        m if m == cm::make(cm::INTRINSIC, cm::DOWN) => {
            return Ok(SessionResult::Drop);
        }

        //
        //
        // ADDRESS COMMANDS
        //
        //

        //
        //
        // ^-^ List locally available addresses
        m if m == cm::make(cm::ADDR, cm::LIST) => {
            let local_addrs = crypto::list_addr_keys(&ctx.meta_db);
            raw_socket
                .write_microframe(MicroframeHeader::intrinsic_noauth(), local_addrs)
                .await?;
        }
        //
        //
        // ^-^ Creating a new address key and client auth token
        m if m == cm::make(cm::ADDR, cm::CREATE) => {
            let addr_create = raw_socket
                .read_payload::<AddrCreate>(header.payload_size)
                .await
                .unwrap();
            let (addr, client_auth) =
                crypto::create_addr_key(&ctx.meta_db, addr_create.name).await?;
            ctx.routes.register_local_route(addr).await?;

            ctx.clients
                .lock_inner()
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

            let auth = check_auth(&header, addr, auth_guard).await?;

            crypto::destroy_addr_key(&ctx.meta_db, addr).await?;

            ctx.routes.scrub_local(addr).await?;

            reply_ok(raw_socket, auth).await?;
        }
        //
        //
        // ^-^ Mark an existing address as "up" given the correct authentication
        m if m == cm::make(cm::ADDR, cm::UP) => {
            let addr_up = raw_socket
                .read_payload::<AddrUp>(header.payload_size)
                .await??;

            let auth = header
                .auth
                .ok_or(RatmanError::ClientApi(ClientError::InvalidAuth))?;

            // If we can decrypt the adress key the token passed authentication
            let _ = crypto::get_addr_key(&ctx.meta_db, addr_up.addr, auth).await?;

            debug!(
                "Client {} provided valid authentication for address '{}'",
                client_id.pretty_string(),
                addr_up.addr.pretty_string()
            );

            // Use the provided auth to open the stored address key.  If this
            // works then we store the provided authentication object in
            // "expected auth"
            auth_guard.lock().await.insert(auth, addr_up.addr);

            let ctx2 = Arc::clone(&ctx);
            spawn(async move {
                if let Err(e) = Arc::clone(&ctx2.protocol)
                    .online(addr_up.addr, auth, ctx2)
                    .await
                {
                    error!("failed to spawn address announcer: {e}");
                }
            });

            reply_ok(raw_socket, auth).await?;
        }
        //
        //
        // ^-^ Mark an address as "down" which is currently "up"
        m if m == cm::make(cm::ADDR, cm::DOWN) => {
            let addr_down = raw_socket
                .read_payload::<AddrDown>(header.payload_size)
                .await??;

            let auth = check_auth(&header, addr_down.addr, auth_guard).await?;

            ctx.protocol.offline(addr_down.addr).await?;
            auth_guard.lock().await.remove(&auth);

            reply_ok(raw_socket, auth).await?;
        }
        //
        //
        // ^-^ List all available local addresses
        m if m == cm::make(cm::ADDR, cm::LIST) => {
            let available_addrs = ctx
                .meta_db
                .addrs
                .iter()
                .into_iter()
                .map(|(ref addr, _)| Address::from_string(addr))
                .collect::<Vec<Address>>();

            raw_socket
                .write_microframe(
                    MicroframeHeader::intrinsic_noauth(),
                    AddrList {
                        list: available_addrs,
                    },
                )
                .await?;
        }

        //
        //
        // NAMESPACE COMMANDS
        //
        //
        m if m == cm::make(cm::SPACE, cm::CREATE) => {
            let NamespaceRegister { pubkey, privkey } = raw_socket
                .read_payload::<NamespaceRegister>(header.payload_size)
                .await?;

            crypto::create_namespace(&ctx.meta_db, None, pubkey, privkey).await?;
            ctx.routes.register_local_route(pubkey).await?;

            raw_socket
                .write_microframe(MicroframeHeader::intrinsic_noauth(), ServerPing::Ok)
                .await?;
        }
        m if m == cm::make(cm::SPACE, cm::UP) => {
            let NamespaceUp {
                client_addr,
                namespace_addr,
            } = raw_socket
                .read_payload::<NamespaceUp>(header.payload_size)
                .await?;
            let _ = crypto::get_namespace_key(&ctx.meta_db, namespace_addr).await?;

            ctx.protocol
                .clone()
                .online_namespace(client_addr, namespace_addr)
                .await?;

            raw_socket
                .write_microframe(MicroframeHeader::intrinsic_noauth(), ServerPing::Ok)
                .await?;
        }
        m if m == cm::make(cm::SPACE, cm::DOWN) => {
            let NamespaceDown {
                client_addr,
                namespace_addr,
            } = raw_socket
                .read_payload::<NamespaceDown>(header.payload_size)
                .await?;
            ctx.protocol
                .clone()
                .offline_namespace(client_addr, namespace_addr)
                .await?;

            raw_socket
                .write_microframe(MicroframeHeader::intrinsic_noauth(), ServerPing::Ok)
                .await?;
        }
        m if m == cm::make(cm::SPACE, cm::ANYCAST) => {
            let anycast_probe = raw_socket
                .read_payload::<AnycastProbe>(header.payload_size)
                .await?;

            let auth = check_auth(&header, anycast_probe.self_addr, auth_guard).await?;

            let peers = ctx
                .protocol
                .run_anycast_probe(
                    &ctx,
                    anycast_probe.self_addr,
                    anycast_probe.namespace_addr,
                    Duration::from_millis(anycast_probe.timeout_ms),
                )
                .await?;

            raw_socket
                .write_microframe(
                    MicroframeHeader::intrinsic_auth(auth),
                    ServerPing::Anycast(
                        peers
                            .into_iter()
                            // FIXME: why is one a u64 and one u128??
                            .map(|(addr, timeout)| (addr, timeout.as_millis() as u64))
                            .collect(),
                    ),
                )
                .await?;
        }

        //
        //
        // STREAM COMMANDS
        //
        //

        //
        //
        // ^-^ Create a new stream subscription
        m if m == cm::make(cm::STREAM, cm::SUB) => {
            let subs_create = raw_socket
                .read_payload::<SubsCreate>(header.payload_size)
                .await?;

            let auth = check_auth(&header, subs_create.addr, auth_guard).await?;

            let (sub_id, rx) = ctx
                .subs
                .create_subscription(subs_create.addr, subs_create.recipient)
                .await?;

            let sub_listen = TcpListener::bind("127.0.0.1:0").await?;
            let bind = sub_listen.local_addr()?.to_string();
            info!(
                "Starting new subscription {} on socket {}",
                sub_id.pretty_string(),
                bind
            );

            let stream_ctx = Arc::clone(ctx);
            spawn(async move {
                debug!("Starting subscription one-shot socket");
                if let Ok((stream, _)) = sub_listen.accept().await {
                    let raw_socket = RawSocketHandle::new(stream);
                    handle_subscription_socket(stream_ctx, rx, raw_socket, auth, sub_id).await;
                }
                debug!("Subscription one-shot has completed");
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
        // ^-^ Destroy an existing stream subscription
        m if m == cm::make(cm::STREAM, cm::UNSUB) => {
            let subs_delete = raw_socket
                .read_payload::<SubsDelete>(header.payload_size)
                .await?;

            let auth = check_auth(&header, subs_delete.addr, auth_guard).await?;

            ctx.subs
                .delete_subscription(subs_delete.addr, subs_delete.sub_id)
                .await?;

            reply_ok(raw_socket, auth).await?;
        }
        //
        //
        // ^-^ List all available subscriptions
        m if m == cm::make(cm::STREAM, cm::LIST) => {
            let addr = raw_socket
                .read_payload::<Address>(header.payload_size)
                .await?;
            let available_subscriptions = ctx
                .subs
                .available_subscriptions(libratman::types::Recipient::Address(addr));
            debug!("ctx.subs() says there are {:?}", available_subscriptions);
            raw_socket
                .write_microframe(
                    match header.auth {
                        Some(auth) => MicroframeHeader::intrinsic_auth(auth),
                        None => MicroframeHeader::intrinsic_noauth(),
                    },
                    ServerPing::Update {
                        available_subscriptions,
                    },
                )
                .await?;
        }
        //
        //
        // ^-^ Subscribe to new address events
        m if m == cm::make(cm::ADDR, cm::SUB) => {
            todo!()
        }
        //
        //
        // ^-^ Unsubscribe from new address events
        m if m == cm::make(cm::ADDR, cm::UNSUB) => {
            todo!()
        }
        //
        //
        // ^-^ Resubscribe from new address events
        m if m == cm::make(cm::ADDR, cm::RESUB) => {
            todo!()
        }
        //
        //
        // ^-^ List all available peers with some metadata
        m if m == cm::make(cm::PEER, cm::LIST) => {
            let peer_list = ctx.routes.list_remote().await?;
            raw_socket
                .write_microframe(
                    MicroframeHeader::intrinsic_noauth(),
                    ServerPing::PeerList(PeerList { list: peer_list }),
                )
                .await?;
        }
        //
        //
        // ^-^ Get some diagnostics about the current status of the router
        m if m == cm::make(cm::INTRINSIC, cm::STATUS) => {
            let num_peers = ctx.routes.list_remote().await?.len() as u64;
            let num_local = crypto::list_addr_keys(&ctx.meta_db).len() as u64;
            let num_auth = ctx.clients.active_auth().lock().await.len() as u64;
            let num_collector_workers = ctx.collector.num_workers().await;

            raw_socket
                .write_microframe(
                    MicroframeHeader::intrinsic_noauth(),
                    ServerPing::Status {
                        num_peers,
                        num_local,
                        num_auth,
                        num_collector_workers,
                    },
                )
                .await?;
        }
        //
        //
        // ^-^ Restore an existing subscription
        m if m == cm::make(cm::STREAM, cm::RESUB) => {
            let subs_restore = raw_socket
                .read_payload::<SubsRestore>(header.payload_size)
                .await?;

            let auth = check_auth(&header, subs_restore.addr, auth_guard).await?;

            let rx = ctx
                .subs
                .restore_subscription(subs_restore.addr, subs_restore.sub_id)
                .await?;

            // crypto::open_space_key(subs_restore.addr, auth);

            let sub_listen = TcpListener::bind("127.0.0.1:0").await?;
            let bind = sub_listen.local_addr()?.to_string();
            let sub_id = subs_restore.sub_id;

            let stream_ctx = Arc::clone(ctx);
            spawn(async move {
                if let Ok((stream, _)) = sub_listen.accept().await {
                    let raw_socket = RawSocketHandle::new(stream);
                    handle_subscription_socket(
                        stream_ctx,
                        rx,
                        raw_socket,
                        auth,
                        subs_restore.sub_id,
                    )
                    .await;
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
        // ^-^ Client wants to receive exactly one message
        m if m == cm::make(cm::RECV, cm::ONE) => {
            let recv_one = raw_socket
                .read_payload::<RecvOne>(header.payload_size)
                .await?;

            debug!("Decode RecvOne {}", recv_one.addr);
            let auth = check_auth(&header, recv_one.addr, auth_guard).await?;

            raw_socket
                .write_microframe(MicroframeHeader::intrinsic_auth(auth), ServerPing::Ok)
                .await?;

            let (tx, mut rx) = bcast_channel(1);
            ctx.clients.insert_sync_listener(recv_one.to, tx).await;
            debug!("Blocking task on synchronous stream receiver");

            match rx.recv().await {
                Ok((letterhead, read_cap)) => {
                    debug!("Relaying stream to sync receiver: {letterhead:?}");
                    raw_socket
                        .write_microframe(
                            MicroframeHeader {
                                modes: cm::make(cm::RECV, cm::ONE),
                                auth: Some(auth),
                                ..Default::default()
                            },
                            letterhead.clone(),
                        )
                        .await?;

                    let mut compat_socket = raw_socket.to_compat();

                    let res =
                        async_eris::decode(&mut compat_socket, &read_cap, &ctx.journal.blocks)
                            .await;

                    raw_socket.from_compat(compat_socket);
                    match res {
                        Ok(()) => reply_ok(raw_socket, auth).await?,
                        Err(e) => {
                            raw_socket
                                .write_microframe(
                                    MicroframeHeader::intrinsic_auth(auth),
                                    ServerPing::Error(ClientError::Internal(e.to_string())),
                                )
                                .await?
                        }
                    }
                }
                Err(e) => {
                    raw_socket
                        .write_microframe(
                            MicroframeHeader::intrinsic_auth(auth),
                            ServerPing::Error(ClientError::Internal(e.to_string())),
                        )
                        .await?
                }
            }

            ctx.clients.remove_sync_listener(recv_one.to).await;
        }
        //
        //
        // ^-^ Client wants to listen for messages in this session
        m if m == cm::make(cm::RECV, cm::MANY) => {
            let RecvMany {
                addr,
                to,
                mut limit,
            } = raw_socket
                .read_payload::<RecvMany>(header.payload_size)
                .await?;

            let auth = check_auth(&header, addr, auth_guard).await?;

            raw_socket
                .write_microframe(MicroframeHeader::intrinsic_auth(auth), ServerPing::Ok)
                .await?;

            let (tx, mut rx) = bcast_channel(8);
            ctx.clients.insert_sync_listener(to, tx).await;

            loop {
                match rx.recv().await {
                    Ok((letterhead, read_cap)) => {
                        raw_socket
                            .write_microframe(
                                MicroframeHeader {
                                    modes: cm::make(cm::RECV, cm::MANY),
                                    auth: Some(auth),
                                    ..Default::default()
                                },
                                letterhead.clone(),
                            )
                            .await?;

                        let mut compat_socket = raw_socket.to_compat();

                        let res =
                            async_eris::decode(&mut compat_socket, &read_cap, &ctx.journal.blocks)
                                .await;

                        raw_socket.from_compat(compat_socket);
                        match res {
                            Ok(()) => {}
                            Err(e) => {
                                raw_socket
                                    .write_microframe(
                                        MicroframeHeader::intrinsic_auth(auth),
                                        ServerPing::Error(ClientError::Internal(e.to_string())),
                                    )
                                    .await?
                            }
                        }

                        // If a limit was set, decrement it
                        if let Some(ref mut limit) = limit {
                            *limit -= 1;
                        }
                    }
                    Err(e) => {
                        raw_socket
                            .write_microframe(
                                MicroframeHeader::intrinsic_auth(auth),
                                ServerPing::Error(ClientError::Internal(e.to_string())),
                            )
                            .await?
                    }
                }

                // If the remaining limit is "1" we break here
                if let Some(1) = limit {
                    ctx.clients.remove_sync_listener(to).await;
                    break;
                }
            }
        }
        //
        //
        // ^-^ Client wants to send a message to one recipient
        m if m == cm::make(cm::SEND, cm::ONE) => {
            debug!("Handle send::one request payload: {}", header.payload_size);

            let SendOne { letterhead } = raw_socket
                .read_payload::<SendOne>(header.payload_size)
                .await??;

            trace!("Receiving {letterhead:?}");

            let auth = check_auth(&header, letterhead.from, auth_guard).await?;
            debug!("{client_id} Passed authentication on [send : one]");

            let ctx = Arc::clone(&ctx);
            let senders = Arc::clone(senders);
            let send_sock_l = TcpListener::bind("127.0.0.1:0").await?;
            let bind = send_sock_l.local_addr()?.to_string();
            let join = spawn(async move {
                let (stream, _) = send_sock_l.accept().await?;
                trace!("Spawn sender task for {client_id}");
                exec_send_many_socket(
                    &ctx,
                    client_id,
                    stream,
                    letterhead.from,
                    auth,
                    vec![letterhead],
                    &senders,
                )
                .await
            });

            raw_socket
                .write_microframe(
                    MicroframeHeader::intrinsic_auth(auth),
                    ServerPing::SendSocket {
                        socket_bind: CString::new(bind).unwrap(),
                    },
                )
                .await?;

            // wait for the sender thread to shut down and return its
            // return status
            let send_sys_res = join.await?;
            match send_sys_res {
                Ok(_) => reply_ok(raw_socket, auth).await?,
                Err(e) => {
                    raw_socket
                        .write_microframe(
                            MicroframeHeader::intrinsic_auth(auth),
                            ServerPing::Error(ClientError::Internal(e.to_string())),
                        )
                        .await?
                }
            }
        }
        //
        //
        // ^-^ Client wants to send a message to many recipients
        m if m == cm::make(cm::SEND, cm::MANY) => {
            debug!("Handle send::many request payload: {}", header.payload_size);

            let SendMany { letterheads } = raw_socket
                .read_payload::<SendMany>(header.payload_size)
                .await??;

            trace!("Receiving {letterheads:?}");

            let this_addr =
                letterheads
                    .iter()
                    .map(|lh| lh.from)
                    .next()
                    .ok_or(RatmanError::ClientApi(ClientError::User(
                        UserError::MissingInput("No letterheads provided!".into()),
                    )))?;

            let auth = check_auth(&header, this_addr, auth_guard).await?;
            debug!("{client_id} Passed authentication on [send : many]");

            let ctx = Arc::clone(&ctx);
            let senders = Arc::clone(senders);
            let send_sock_l = TcpListener::bind("127.0.0.1:0").await?;
            let bind = send_sock_l.local_addr()?.to_string();
            let join = spawn(async move {
                let (stream, _) = send_sock_l.accept().await?;
                trace!("Spawn sender task");
                exec_send_many_socket(
                    &ctx,
                    client_id,
                    stream,
                    this_addr,
                    auth,
                    letterheads,
                    &senders,
                )
                .await
            });

            raw_socket
                .write_microframe(
                    MicroframeHeader::intrinsic_auth(auth),
                    ServerPing::SendSocket {
                        socket_bind: CString::new(bind).unwrap(),
                    },
                )
                .await?;

            // wait for the sender thread to shut down and return its
            // return status
            let send_sys_res = join.await?;
            match send_sys_res {
                Ok(_) => reply_ok(raw_socket, auth).await?,
                Err(e) => {
                    raw_socket
                        .write_microframe(
                            MicroframeHeader::intrinsic_auth(auth),
                            ServerPing::Error(ClientError::Internal(e.to_string())),
                        )
                        .await?
                }
            }
        }
        //
        //
        // u-u Don't know what to do with this
        mode => {
            raw_socket
                .write_microframe(
                    MicroframeHeader::intrinsic_noauth(),
                    ServerPing::Error(ClientError::Internal(format!(
                        "Unsupported frame mode: {mode:b}"
                    ))),
                )
                .await?;

            return Ok(SessionResult::Next);
        }
    }

    Ok(SessionResult::Next)
}
