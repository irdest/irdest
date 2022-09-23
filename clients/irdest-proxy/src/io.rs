use crate::proto::Envelope;
use async_std::{
    io::{self, ReadExt, WriteExt},
    net::TcpStream,
};
use ratman_client::{Address, RatmanIpc, Receive_Type};

/// Connect to the router with a fixed address
pub async fn connect_with_address(bind: Option<&str>, addr: Address) -> io::Result<RatmanIpc> {
    Ok(match bind {
        Some(bind) => RatmanIpc::connect(bind, Some(addr)).await,
        None => RatmanIpc::default_with_addr(addr).await,
    }?)
}

pub async fn terminate_session(addr: Address, session: Address, ipc: &RatmanIpc) -> io::Result<()> {
    let env = Envelope::end(session).encode();
    ipc.send_to(addr, env).await?;
    Ok(())
}

pub async fn from_tcp_to_ratman(
    addr: Address,
    session: Address,
    tcp: &mut TcpStream,
    ipc: &RatmanIpc,
) -> io::Result<()> {
    let mut buffer = vec![0; 1024]; // TODO: get this size from IPC interface
    tcp.read(&mut buffer).await?;

    debug!("Read message from TCP, wrapping into Ratman envelope...");

    // Encode data into envelope
    let env = Envelope::with_session(session, buffer);

    ipc.send_to(addr, env.encode()).await?;
    Ok(())
}

/// Get a message for a session from Ratman
pub async fn from_ratman(ipc: &RatmanIpc) -> Option<(Address, Option<Vec<u8>>)> {
    ipc.next()
        .await
        .filter(|(t, _)| t == &Receive_Type::DEFAULT)
        .map(|(_, msg)| {
            let env = Envelope::decode(&msg.get_payload());
            (env.session, env.data)
        })
}

pub async fn to_tcp(tcp: &mut TcpStream, data: Vec<u8>) -> io::Result<()> {
    tcp.write_all(&data).await?;
    Ok(())
}
