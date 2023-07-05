// SPDX-FileCopyrightText: 2019-2022 Katharina Fey <kookie@spacekookie.de>
//
// SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

// use crate::{
//     daemon::{
//         state::{Io, OnlineMap},
//         transform,
//     },
//     Message, Result, Router,
// };

use crate::Router;
use async_std::io::{Read, Write};
use libratman::types::{
    api::{
        all_peers, api_peers, api_setup, online_ack, ApiMessageEnum, Peers, Peers_Type, Receive,
        Send, Setup, Setup_Type,
    },
    encode_message, parse_message, write_with_length, Address, Error as ParseError, Recipient,
    Result as ParseResult,
};

async fn handle_send(r: &Router, online: &OnlineMap, _self: Address, send: Send) -> Result<()> {
    debug!("Queuing message to send");
    let mirror = send.mirror;
    for msg in transform::send_to_message(send) {
        let Message {
            ref id,
            ref sender,
            ref recipient,
            ref payload,
            ref timesig,
            ref sign,
        } = msg;

        match msg.recipient {
            Recipient::Flood(_) => {
                let recv = types::api::receive_default(types::Message::received(
                    *id,
                    *sender,
                    recipient.clone(),
                    payload.clone(),
                    format!("{:?}", timesig),
                    sign.clone(),
                ));

                for (id, ref mut io) in online.lock().await.iter_mut() {
                    if io.is_none() && continue {} // skip if the endpoint is unavailable
                    if id == &_self && !mirror && continue {} // skip if recipient is self and mirror = false
                    if let Err(e) = forward_recv(io.as_mut().unwrap().as_io(), recv.clone()).await {
                        error!("Failed to forward received message: {}", e);
                    }
                }
            }
            _ => {}
        }
        r.send(msg).await?;
    }
    Ok(())
}

// TODO: why does this function exist?
async fn handle_setup(_io: &mut Io, _r: &Router, s: Setup) -> Result<()> {
    trace!("Handle setup message: {:?}", s);
    Ok(())
}

async fn handle_peers(io: &mut Io, r: &Router, peers: Peers) -> Result<()> {
    if peers.field_type != Peers_Type::REQ {
        return Ok(()); // Ignore all other messages
    }

    let all = r
        .known_addresses()
        .await
        .into_iter()
        .map(|(addr, _)| addr)
        .collect();
    let response = encode_message(api_peers(all_peers(all))).unwrap();
    write_with_length(io.as_io(), &response).await.unwrap(); // baaaaad
    Ok(())
}

async fn send_online_ack<Io: Write + Unpin>(io: &mut Io, id: Address) -> ParseResult<()> {
    let ack = encode_message(api_setup(online_ack(id)))?;
    write_with_length(io, &ack).await?;
    Ok(())
}

/// Handle the initial handshake with the daemon
///
/// Wait for a message to come in.  Either it is
///
/// 1. An `Online` message with attached identity
///   - Authenticate token
///   - Save stream for address
/// 2. An `Online` without attached identity
///   - Assign an address
///   - Return address and auth token
/// 3. Any other payload is invalid
pub(crate) async fn handle_auth<Io: Read + Write + Unpin>(
    io: &mut Io,
    r: &Router,
) -> ParseResult<Option<(Address, Vec<u8>)>> {
    debug!("Handle authentication request for new connection");

    let one_of = parse_message(io)
        .await
        .map(|msg| msg.inner)?
        .ok_or(ParseError::InvalidAuth)?;

    match one_of {
        ApiMessageEnum::setup(setup) if setup.field_type == Setup_Type::ONLINE => {
            let id = setup.id;
            let token = setup.token;

            match (id, token) {
                // FIXME: validate token
                (id, token) if id.len() != 0 && token.len() != 0 => {
                    debug!("Authorisation for known client");
                    let id = Address::from_bytes(id.as_slice());
                    let _ = r.add_user().await;
                    r.online(id).await?;

                    send_online_ack(io, id).await?;
                    Ok(Some((id, vec![])))
                }
                (id, token) if id.len() == 0 && token.len() == 0 => {
                    debug!("Authorisation for new client");
                    let id = Address::random();
                    r.add_existing_user(id).await?;
                    r.online(id).await?;

                    send_online_ack(io, id).await?;
                    Ok(Some((id, vec![])))
                }
                _ => {
                    debug!("Failed to authenticate client");
                    Err(ParseError::InvalidAuth)
                }
            }
        }
        // If the client wants to remain anonymous we don't return an ID/token pair
        ApiMessageEnum::setup(setup) if setup.field_type == Setup_Type::ANONYMOUS => {
            debug!("Authorisation for anonymous client");
            Ok(None)
        }
        _ => Err(ParseError::InvalidAuth),
    }
}

/// Parse messages from a stream until it terminates
pub(crate) async fn parse_stream(router: Router, online: OnlineMap, _self: Address, mut io: Io) {
    loop {
        // Match on the msg type and call the appropriate handler
        match parse_message(io.as_io()).await.map(|msg| msg.inner) {
            Ok(Some(one_of)) => match one_of {
                ApiMessageEnum::send(send) => handle_send(&router, &online, _self, send).await,
                ApiMessageEnum::setup(setup) => handle_setup(&mut io, &router, setup).await,
                ApiMessageEnum::peers(peers) => handle_peers(&mut io, &router, peers).await,
                ApiMessageEnum::recv(_) => continue, // Ignore "Receive" messages
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

pub(crate) async fn forward_recv<Io: Write + Unpin>(io: &mut Io, r: Receive) -> ParseResult<()> {
    let api = types::api::api_recv(r);
    trace!("Encoding received message...");
    let msg = types::encode_message(api)?;
    trace!("Forwarding payload through stream");
    write_with_length(io, &msg).await?;
    Ok(())
}
