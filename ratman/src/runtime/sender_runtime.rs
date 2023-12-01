use libratman::{
    api::socket_v2::RawSocketHandle,
    frame::micro::MicroframeHeader,
    rt::{new_async_thread, AsyncSystem},
    tokio::{net::TcpStream, sync::mpsc::channel},
};
use std::sync::Arc;

pub struct SenderRuntime(AsyncSystem);

/// Spawn a new sender thread
pub async fn new_sender(mut s: TcpStream) -> Arc<SenderRuntime> {
    // let (tx, rx) = channel(8);
    let mut socket = RawSocketHandle::new(s);
    let header: MicroframeHeader = socket.read_header().await.unwrap();

    todo!()
}
