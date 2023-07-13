use crate::{
    config::IpSpace,
    io::{connect_with_address, from_ratman, to_tcp},
    server::SessionMap,
};
use async_std::{io, net::TcpStream, task};
use libratman::client::Address;

pub struct Outlet {
    map: SessionMap,
}

impl Outlet {
    pub fn new(
        map: &SessionMap,
        bind: Option<&str>,
        ip: &IpSpace,
        addr: Address,
    ) -> io::Result<()> {
        task::block_on(Self { map: map.clone() }.spawn(bind, ip, addr))
    }

    async fn spawn(self, bind: Option<&str>, ip: &IpSpace, addr: Address) -> io::Result<()> {
        let socket_addr = ip.socket_addr().clone();
        let ipc = connect_with_address(bind, addr).await?;

        debug!("Starting the outlet loop");

        // We don't set up a lot of state for this type.  We do
        // however move Self into the task so it doesn't go out of
        // scope.
        task::spawn(async move {
            let this = self;

            while let Some((session, data)) = from_ratman(&ipc).await {
                let session_exists = this.map.read().await.contains_key(&session);

                match (session_exists, data) {
                    // Session exists, no data sent --> drop session
                    (true, None) => {
                        debug!("Clearing session: {:?}", session);
                        this.map.write().await.remove(&session);
                    }
                    // Session exists and we received data --> forward
                    (true, Some(data)) => {
                        let mut tcp = this.map.read().await.get(&session).unwrap().clone();
                        trace!("Received data, forwarding to {:?}", tcp.peer_addr());
                        if let Err(e) = to_tcp(&mut tcp, data).await {
                            error!("failed to send message for session {}: {}", session, e);
                            this.map.write().await.remove(&session);
                        }
                    }
                    // No session exists, but we received data --> create session
                    (false, Some(data)) => {
                        debug!("Creating new session: {:?}", session);
                        let mut tcp = match TcpStream::connect(&socket_addr).await {
                            Ok(tcp) => tcp,
                            Err(e) => {
                                error!(
                                    "failed to establish outbound connection to {}: {}",
                                    socket_addr, e
                                );
                                break;
                            }
                        };

                        if let Err(e) = to_tcp(&mut tcp, data).await {
                            error!("failed to send message for session {}: {}", session, e);
                        }

                        this.map.write().await.insert(session, tcp);
                    }
                    // No session exists and no data was sent --> ignore
                    (false, None) => continue,
                }
            }
        });

        Ok(())
    }
}
