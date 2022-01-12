//! Ratman client bindings library
//!
//! To learn more about Ratman and Irdest, visit https://irde.st!
//!
//! In order to interact with the Ratman daemon your application must
//! send properly formatted API messages over a local TCP socket.
//! These data formats are outlined in [ratman-types](ratman_types)!
//!
//! This crate provides a simple API over these API messages!

use async_std::{
    net::TcpStream,
    sync::{Arc, Mutex},
    task,
};
use types::{
    api::{self, ApiMessageEnum, Setup_Type::ACK},
    encode_message, parse_message, read_with_length, write_with_length, Identity,
};
pub use types::{Error, Result};

pub struct RatmanIpc {
    socket: Arc<Mutex<TcpStream>>,
    addr: Identity,
}
impl Default for RatmanIpc {
    /// Create a `DEFAULT` variant will always register a new address
    fn default() -> Self {
        match task::block_on(RatmanIpc::connect("127.0.0.1:9020", None)) {
            Ok(ipc) => ipc,
            Err(e) => panic!("Failed to initialise RatmanIpc: {}", e),
        }
    }
}

impl RatmanIpc {
    /// Connect to a Ratman IPC backend with an optional address
    ///
    /// `socket_addr` refers to the local address the Ratman daemon is
    /// listening on.  `addr` refers to the Ratman cryptographic
    /// routing address associated with your application
    pub async fn connect(socket_addr: &str, addr: Option<Identity>) -> Result<RatmanIpc> {
        let mut socket = TcpStream::connect(socket_addr).await?;

        // Introduce ourselves to the daemon
        let online_msg = api::api_setup(match addr {
            Some(addr) => api::online(addr, vec![]),
            None => api::online_init(),
        });
        write_with_length(&mut socket, &encode_message(online_msg)?).await?;

        // Then wait for a response and assign the used address
        let addr = match parse_message(&mut socket).await.map(|m| m.inner) {
            Ok(Some(one_of)) => match one_of {
                ApiMessageEnum::setup(s) if s.field_type == ACK => s
                    ._id
                    .as_ref()
                    .map(|_| Identity::from_bytes(s.get_id()))
                    .or(addr)
                    .expect("failed to initialise new address!"),
                _ => unreachable!(),
            },
            _ => unreachable!(),
        };

        // TODO: spawn receive daemon here
        let socket = Arc::new(Mutex::new(socket));

        Ok(Self { socket, addr })
    }

    /// Return the currently assigned address
    pub fn address(&self) -> Identity {
        self.addr
    }

    /// Send some data to a remote peer
    pub async fn send(&self, data: Vec<u8>) -> Result<()> {
        Ok(())
    }
}
