use libratman::{
    api::socket_v2::RawSocketHandle, frame::micro::MicroframeHeader, rt::AsyncSystem,
    tokio::net::TcpStream,
};
use std::sync::Arc;

pub struct SenderRuntime(AsyncSystem);

/// Spawn a new sender thread
pub async fn new_sender(s: TcpStream) -> Arc<SenderRuntime> {
    // let (tx, rx) = channel(8);
    let mut socket = RawSocketHandle::new(s);
    let _header: MicroframeHeader = socket.read_header().await.unwrap();

    todo!()
}
