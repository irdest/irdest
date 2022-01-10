use crate::{daemon::transform, Result, Router};
use async_std::{
    io::{Read, Write},
    prelude::*,
};
use byteorder::{BigEndian, ByteOrder};
use protobuf::Message;
use std::{io, sync::Arc};
use types::api::{ApiMessage, ApiMessageEnum, Peers, Receive, Send, Setup};

pub(crate) type ParseResult<T> = std::result::Result<T, ParseError>;

#[derive(Debug, thiserror::Error)]
pub(crate) enum ParseError {
    #[error("failed to perform system i/o operation: {}", 0)]
    Io(#[from] io::Error),
    #[error("failed to parse base encoding: {}", 0)]
    Proto(#[from] protobuf::ProtobufError),
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

async fn handle_send(r: &Arc<Router>, send: Send) -> Result<()> {
    for msg in transform::send_to_message(send) {
        r.send(msg).await?;
    }
    Ok(())
}

async fn handle_recv(r: &Arc<Router>, recv: Receive) -> Result<()> {
    Ok(())
}
async fn handle_setup(r: &Arc<Router>, setup: Setup) -> Result<()> {
    Ok(())
}
async fn handle_peers(r: &Arc<Router>, peers: Peers) -> Result<()> {
    Ok(())
}

/// Parse messages from a stream until it terminates
pub(crate) async fn parse_stream<R: Read + Unpin>(router: Arc<Router>, mut r: R) {
    loop {
        // Match on the msg type and call the appropriate handler
        match parse_message(&mut r).await.map(|msg| msg.inner) {
            Ok(Some(one_of)) => match one_of {
                ApiMessageEnum::send(send) => handle_send(&router, send).await,
                ApiMessageEnum::recv(recv) => handle_recv(&router, recv).await,
                ApiMessageEnum::setup(setup) => handle_setup(&router, setup).await,
                ApiMessageEnum::peers(peers) => handle_peers(&router, peers).await,
            },
            Err(e) => {
                tracing::warn!("Parse stream terminated: `{}`", e);
                break;
            }
            _ => panic!(),
        }
        .unwrap_or_else(|e| error!("Failed to execute command: {:?}", e));
    }
}
