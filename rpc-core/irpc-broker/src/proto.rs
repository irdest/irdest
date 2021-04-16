use crate::map::ServiceMap;
use async_std::net::TcpStream;
use irpc_sdk::{
    error::{RpcError, RpcResult},
    io,
    io::Message,
    proto::{Registry, SdkCommand, SdkReply},
    DEFAULT_BROKER_ADDRESS as ADDRESS,
};
use serde::Serialize;
use std::sync::Arc;

/// Send a reply message back to whoever talked to us
async fn send_reply<S: Serialize>(
    stream: &mut TcpStream,
    msg: Message,
    enc: u8,
    data: S,
) -> RpcResult<()> {
    let msg = msg.reply(ADDRESS, io::encode(enc, &data)?);
    io::send(stream, enc, &msg).await
}

/// Handle a registry message from a new client
pub(crate) async fn register_service(
    map: &Arc<ServiceMap>,
    stream: &mut TcpStream,
) -> RpcResult<()> {
    let msg = io::recv(stream).await?;
    let Registry {
        name,
        version,
        description,
        caps,
    } = Registry::parse(&msg)?;
    let encoding = caps.encoding;

    // Check if a service of this name already exists
    let reply = if let Ok(_) = map.caps(&name).await {
        SdkReply::Error(RpcError::AlreadyRegistered)
    } else {
        // The error case is actually correct here
        let id = map
            .register(name, version, description, caps, &stream)
            .await;
        SdkReply::Identity(id)
    };

    send_reply(stream, msg, encoding, reply).await
}

/// Handle messages address to the broker
pub(crate) async fn handle_sdk_command(
    stream: &mut TcpStream,
    map: &Arc<ServiceMap>,
    msg: Message,
) -> RpcResult<()> {
    let caps = map.caps(&msg.from).await?;
    match SdkCommand::parse(caps.encoding, &msg) {
        // Successfully parsed a shutdown command
        Ok(SdkCommand::Shutdown {
            ref name,
            ref hash_id,
        }) => match map.match_id(name, hash_id).await {
            Ok(()) => {
                map.shutdown(name).await;
                send_reply(stream, msg, caps.encoding, SdkReply::Ok).await?
            }
            Err(e) => send_reply(stream, msg, caps.encoding, SdkReply::Error(e)).await?,
        },
        _ => todo!(),
    };

    Ok(())
}

/// Handle messages address to another service
pub(crate) async fn proxy_message(
    _return: &mut TcpStream,
    map: &Arc<ServiceMap>,
    msg: Message,
) -> RpcResult<()> {
    debug!("Proxying message from '{}' to '{}'", msg.from, msg.to);

    // Fetch the target service capabilities
    let caps = map.caps(&msg.to).await?;
    match map.stream(&msg.to).await {
        Ok(ref mut stream) => io::send(stream, caps.encoding, &msg).await,
        Err(_) => {
            warn!(
                "Swallowing message addressed to unknown service: '{}'",
                msg.to
            );

            Ok(())
        }
    }
}
