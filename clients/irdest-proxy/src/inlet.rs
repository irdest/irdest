use crate::{
    config::IpSpace,
    io::{connect_with_address, from_tcp_to_ratman, terminate_session},
};
use async_std::{io, net::TcpListener, stream::StreamExt, task};
use libratman::client::Address;

/// An inlet takes data from a socket and maps it into a Ratman
/// message to a particular peer (provided to the `new` function).
///
/// Otherwise the Inlet is completely stateless.  It can deduce its
/// state from incoming connections and exsiting tasks.
pub struct Inlet;

impl Inlet {
    /// Spawn an inlet listener
    pub fn new(
        bind: Option<&str>,
        ip: &IpSpace,
        peer_addr: Address,
        self_addr: Address,
    ) -> io::Result<()> {
        task::block_on(Self.spawn(bind, ip, peer_addr, self_addr))
    }

    async fn spawn(
        self,
        bind: Option<&str>,
        ip: &IpSpace,
        peer_addr: Address,
        self_addr: Address,
    ) -> io::Result<()> {
        let socket_addr = ip.socket_addr().clone();
        let tcp = TcpListener::bind(&socket_addr).await?;
        let ipc = connect_with_address(bind, self_addr).await?;

        debug!("Starting inlet loop");

        task::spawn(async move {
            let mut inc = tcp.incoming();

            // A new stream means a new session, so given that the
            // stream is valid we first generate a session ID
            while let Some(stream) = inc.next().await {
                let mut stream = match stream {
                    Ok(s) => s,
                    Err(e) => {
                        warn!("invalid stream tried to connect to {}: {}", socket_addr, e);
                        continue;
                    }
                };

                debug!("Accepted new stream!");

                // Then spawn a new task for this session.  We keep
                // reading messages from the TCP stream until we no
                // longer get any (i.e. the socket collapses or
                // something)
                let ipc = ipc.clone();
                task::spawn(async move {
                    let session = Address::random();
                    while let Ok(_) =
                        from_tcp_to_ratman(peer_addr, session, &mut stream, &ipc).await
                    {
                    }

                    // Before we kill the task we send one last
                    // message to the peer to terminate the session on
                    // their end
                    if let Err(e) = terminate_session(peer_addr, session, &ipc).await {
                        error!("failed to terminate session: {}", e);
                    }
                });
            }
        });

        Ok(())
    }
}
