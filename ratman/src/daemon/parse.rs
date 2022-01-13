use crate::{
    daemon::{state::Io, transform},
    Result, Router,
};

use async_std::io::{Read, Write};
use identity::Identity;
use types::{
    api::{
        api_setup, online_ack, ApiMessageEnum, Peers, Receive, Send, Setup, Setup_Type,
        Setup_oneof__id,
    },
    encode_message, parse_message, write_with_length, Error as ParseError, Result as ParseResult,
};

async fn handle_send(r: &Router, send: Send) -> Result<()> {
    for msg in transform::send_to_message(send) {
        r.send(msg).await?;
    }
    Ok(())
}

async fn handle_setup(_r: &Router, s: Setup) -> Result<()> {
    trace!("Handle setup message: {:?}", s);
    Ok(())
}

async fn handle_peers(_: &Router, _: Peers) -> Result<()> {
    Ok(())
}

async fn send_online_ack<Io: Write + Unpin>(io: &mut Io, id: Identity) -> ParseResult<()> {
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
) -> ParseResult<(Identity, Vec<u8>)> {
    trace!("Handle authentication request for new connection");

    let one_of = parse_message(io)
        .await
        .map(|msg| msg.inner)?
        .ok_or(ParseError::InvalidAuth)?;

    match one_of {
        ApiMessageEnum::setup(setup) if setup.field_type == Setup_Type::ONLINE => {
            let id = setup._id;
            let token = setup._token;

            match (id, token) {
                // FIXME: validate token
                (Some(Setup_oneof__id::id(id)), Some(_)) => {
                    let id = Identity::from_bytes(id.as_slice());
                    r.online(id).await.unwrap();
                    Ok((id, vec![]))
                }
                (None, None) => {
                    let id = Identity::random();
                    r.add_user(id).await.unwrap();
                    send_online_ack(io, id).await?;
                    Ok((id, vec![]))
                }
                _ => Err(ParseError::InvalidAuth),
            }
        }
        _ => Err(ParseError::InvalidAuth),
    }
}

/// Parse messages from a stream until it terminates
pub(crate) async fn parse_stream(router: Router, mut io: Io) {
    loop {
        // Match on the msg type and call the appropriate handler
        match parse_message(io.as_io()).await.map(|msg| msg.inner) {
            Ok(Some(one_of)) => match one_of {
                ApiMessageEnum::send(send) => handle_send(&router, send).await,
                ApiMessageEnum::setup(setup) => handle_setup(&router, setup).await,
                ApiMessageEnum::peers(peers) => handle_peers(&router, peers).await,
                ApiMessageEnum::recv(_) => continue, // Ignore "Receive" messages
            },
            Ok(None) => {
                warn!("Received invalid message: empty payload");
                continue;
            }
            Err(e) => {
                info!("Parse stream terminated: `{}`", e);
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
