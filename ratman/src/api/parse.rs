// SPDX-FileCopyrightText: 2019-2022 Katharina Fey <kookie@spacekookie.de>
//
// SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

use crate::{
    api::{client::OnlineClient, io::Io},
    context::RatmanContext,
    util::transform,
};
use async_std::{
    io::{Read, Write},
    sync::Arc,
};
use libratman::types::{
    self,
    api::{
        self, all_peers, api_peers, api_setup, online_ack, ApiMessageEnum, Peers, Peers_Type,
        Receive, Send, Setup_Type,
    },
    encode_message, parse_message, write_with_length, Address, ClientError, Id, Message, Recipient,
    Result,
};

async fn handle_send(ctx: &Arc<RatmanContext>, _self: Address, send: Send) -> Result<()> {
    debug!("Queuing message to send");
    let mirror = send.mirror;
    for msg in transform::send_to_message(send) {
        let Message {
            ref id,
            ref sender,
            ref recipient,
            ref payload,
            ref time,
            ref signature,
        } = msg;

        match msg.recipient {
            Recipient::Flood(_) => {
                let recv = api::receive_default(Message::received(
                    *id,
                    *sender,
                    recipient.clone(),
                    payload.clone(),
                    format!("{:?}", time),
                    signature.clone(),
                ));

                for (
                    _client_id,
                    OnlineClient {
                        ref mut io,
                        ref base,
                    },
                ) in ctx.clients.online.lock().await.iter_mut()
                {
                    // Skip if recipient is self and mirror = false
                    if base.primary_address() == _self && !mirror && continue {}

                    // Otherwise try to forward the message to the given I/O socket
                    if let Err(e) = forward_recv(io.as_io(), recv.clone()).await {
                        error!("Failed to forward received message: {}", e);
                    }
                }
            }
            _ => {}
        }
        ctx.core.send(msg).await?;
    }
    Ok(())
}

async fn handle_peers(io: &mut Io, ctx: &Arc<RatmanContext>, peers: Peers) -> Result<()> {
    if peers.field_type != Peers_Type::REQ {
        return Ok(()); // Ignore all other messages
    }

    let all = ctx
        .core
        .all_known_addresses()
        .await
        .into_iter()
        .map(|(addr, _)| addr)
        .collect();
    let response = encode_message(api_peers(all_peers(all))).unwrap();
    write_with_length(io.as_io(), &response).await?;
    Ok(())
}

async fn send_online_ack<Io: Write + Unpin>(io: &mut Io, id: Address) -> Result<()> {
    let ack = encode_message(api_setup(online_ack(id)))?;
    write_with_length(io, &ack).await?;
    Ok(())
}

/// Handle the initial handshake with the daemon
///
/// It either authenticates an existing client, or registers a new
/// one.  In either case, the return value will be `Ok(Some(_))`,
/// containing the newly created address and associated client token
/// (FIXME: currently `client_id` and `address` are interchangable in
/// certain parts of the API, but not others.  This needs to become
/// more consistent).
///
/// If the client wishes to remain anynomous (for example simply for
/// querying the status interfaces, but never receiving a message),
/// the return value will be `Ok(None)`.
///
/// If any error occurs during authentication, `Err(_)` is returned.
pub(crate) async fn handle_auth<Io: Read + Write + Unpin>(
    io: &mut Io,
    ctx: &Arc<RatmanContext>,
) -> Result<Option<(Address, Vec<u8>)>> {
    debug!("Handle authentication request for new connection");

    // Wait for a message to come in.  Either it is
    //
    // 1. An `Online` message with attached identity
    //   - Authenticate token
    //   - Save stream for address
    // 2. An `Online` without attached identity
    //   - Assign an address
    //   - Return address and auth token
    // 3. Any other payload is invalid
    let one_of = parse_message(io)
        .await
        .map(|msg| msg.inner)?
        .ok_or(ClientError::InvalidAuth)?;

    match one_of {
        ApiMessageEnum::setup(setup) if setup.field_type == Setup_Type::ONLINE => {
            let address = Address::try_from_bytes(&setup.id).ok();
            let token = Id::try_from_bytes(&setup.token).ok();

            match (address, token) {
                // Both address and token were sent -> existing client
                (Some(address), Some(token)) => {
                    let client_id = ctx.clients.get_client_for_address(&address).await;
                    if ctx.clients.check_token(&client_id, &token).await {
                        // TODO: is this really the best place for this call ?
                        if ctx.load_existing_address(address, &[0]).await.is_ok() {
                            send_online_ack(io, address).await?;
                        }

                        // FIXME: what is the second argument here
                        // supposed to be doing anyway ?
                        Ok(Some((address, vec![])))
                    } else {
                        Err(ClientError::InvalidAuth.into())
                    }
                }

                // Neither an address nor token were sent -> new client
                (None, None) => {
                    todo!()
                }

                // address XOR token were sent -> invalid
                _ => Err(ClientError::InvalidAuth.into()),
            }
        }

        // If the client wants to remain anonymous we don't return an ID/token pair
        ApiMessageEnum::setup(setup) if setup.field_type == Setup_Type::ANONYMOUS => {
            debug!("Authorisation for anonymous client");
            Ok(None)
        }

        // Any other payload here is invalid and we return an error
        _ => Err(ClientError::InvalidAuth.into()),
    }
}

//         match (address, token) {
//             // We were sent both an address and a token
//             (address, token) if address.len() != 0 && token.len() != 0 => {
//                 // First we compare the token to the one on record
//                 let client_id = ctx.clients.get_client_for_address(address).await;

//                 // debug!("Authorisation for known client");
//                 // let id = Address::from_bytes(id.as_slice());
//                 // let _ = ctx.create_new_address().await;
//                 // ctx.online(id).await?;

//                 // send_online_ack(io, id).await?;
//                 //

//                 todo!()
//             }
//             (id, token) if id.len() == 0 && token.len() == 0 => {
//                 debug!("Authorisation for new client");
//                 // let id = Address::random();
//                 // r.add_existing_user(id).await?;
//                 // r.online(id).await?;

//                 // send_online_ack(io, id).await?;
//                 // Ok(Some((id, vec![])))

//                 todo!()
//             }
//             _ => {
//                 debug!("Failed to authenticate client");
//                 Err(ClientError::InvalidAuth.into())
//             }
//         }
//     }

/// Parse messages from a stream until it terminates
pub(crate) async fn parse_stream(ctx: Arc<RatmanContext>, _self: Address, mut io: Io) {
    loop {
        // Match on the msg type and call the appropriate handler
        match parse_message(io.as_io()).await.map(|msg| msg.inner) {
            Ok(Some(one_of)) => match one_of {
                ApiMessageEnum::send(send) => handle_send(&ctx, _self, send).await,
                ApiMessageEnum::peers(peers) => handle_peers(&mut io, &ctx, peers).await,
                ApiMessageEnum::setup(_) => continue, // Handled in state.rs
                ApiMessageEnum::recv(_) => continue,  // Ignore "Receive" messages
            },
            Ok(None) => {
                warn!("Received invalid message: empty payload");
                continue;
            }
            Err(e) => {
                trace!("Error: {:?}", e);
                info!("API stream was dropped by client");
                break;
            }
        }
        .unwrap_or_else(|e| error!("Failed to execute command: {:?}", e));
    }
}

pub(crate) async fn forward_recv<Io: Write + Unpin>(io: &mut Io, r: Receive) -> Result<()> {
    let api = api::api_recv(r);
    trace!("Encoding received message...");
    let msg = types::encode_message(api)?;
    trace!("Forwarding payload through stream");
    write_with_length(io, &msg).await?;
    Ok(())
}
