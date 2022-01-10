use crate::{
    daemon::{state::ShareIo, transform},
    Result, Router,
};
use async_std::{
    io::{Read, Write},
    prelude::*,
};
use byteorder::{BigEndian, ByteOrder};
use identity::Identity;
use protobuf::Message;
use std::{io, sync::Arc};
use types::api::{
    ApiMessage, ApiMessageEnum, Peers, Receive, Send, Setup, Setup_Type, Setup_oneof__id,
};

pub(crate) type ParseResult<T> = std::result::Result<T, ParseError>;

#[derive(Debug, thiserror::Error)]
pub(crate) enum ParseError {
    #[error("failed to perform system i/o operation: {}", 0)]
    Io(#[from] io::Error),
    #[error("failed to parse base encoding: {}", 0)]
    Proto(#[from] protobuf::ProtobufError),
    #[error("failed to provide correct authentication in handshake")]
    InvalidAuth,
}

/// First write the length as big-endian u64, then write the provided buffer
async fn write_with_length<T: Write + Unpin>(t: &mut T, buf: &Vec<u8>) -> ParseResult<usize> {
    let mut len = vec![0; 8];
    BigEndian::write_u64(&mut len, buf.len() as u64);
    t.write_all(len.as_slice()).await?;
    t.write_all(buf.as_slice()).await?;
    Ok(len.len() + buf.len())
}

/// First read a big-endian u64, then read the number of bytes
async fn read_with_length<T: Read + Unpin>(r: &mut T) -> ParseResult<Vec<u8>> {
    let mut len_buf = vec![0; 8];
    r.read_exact(&mut len_buf).await?;
    let len = BigEndian::read_u64(&len_buf);

    let mut vec = vec![0; len as usize]; // FIXME: this might break on 32bit systems
    r.read_exact(&mut vec).await?;
    Ok(vec)
}

/// Parse a single message from a reader stream
async fn parse_message<R: Read + Unpin>(r: &mut R) -> ParseResult<ApiMessage> {
    let vec = read_with_length(r).await?;
    Ok(ApiMessage::parse_from_bytes(&vec)?)
}

async fn handle_send(r: &Router, send: Send) -> Result<()> {
    for msg in transform::send_to_message(send) {
        r.send(msg).await?;
    }
    Ok(())
}

async fn handle_setup(r: &Router, setup: Setup) -> Result<()> {
    Ok(())
}
async fn handle_peers(r: &Router, peers: Peers) -> Result<()> {
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
) -> ParseResult<(Identity, Vec<u8>)> {
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
                    Ok((Identity::from_bytes(id.as_slice()), vec![]))
                }
                (None, None) => Ok((Identity::random(), vec![])),
                _ => Err(ParseError::InvalidAuth),
            }
        }
        _ => Err(ParseError::InvalidAuth),
    }
}

/// Parse messages from a stream until it terminates
pub(crate) async fn parse_stream(router: Router, io: ShareIo) {
    loop {
        let mut io = io.lock().await;

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

        drop(io);
        async_std::task::sleep(std::time::Duration::from_micros(100)).await;
    }
}

pub(crate) async fn forward_recv<Io: Write + Unpin>(io: &mut Io, r: Receive) -> ParseResult<()> {
    let mut buf = vec![];
    r.write_to_vec(&mut buf)?;
    write_with_length(io, &buf).await?;
    Ok(())
}
