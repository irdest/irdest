//! Ratman client bindings library
//!
//! To learn more about Ratman and Irdest, visit https://irde.st!
//!
//! In order to interact with the Ratman daemon your application must
//! send properly formatted API messages over a local TCP socket.
//! These data formats are outlined in [ratman-types](ratman_types)!
//!
//! This crate provides a simple API over these API messages!

#[macro_use]
extern crate tracing;

use async_std::{
    channel::{unbounded, Receiver, Sender},
    net::TcpStream,
    task,
};
pub use types::{api::Receive_Type, message::Message, Error, Identity, Result};
use types::{
    api::{self, ApiMessageEnum, Setup_Type::ACK},
    encode_message, message, parse_message, read_with_length, write_with_length,
};

pub struct RatmanIpc {
    socket: TcpStream,
    addr: Identity,
    recv: Receiver<(Receive_Type, Message)>,
}

impl RatmanIpc {
    /// Create a `DEFAULT` variant will always register a new address
    pub async fn default() -> Result<Self> {
        Self::connect("127.0.0.1:9020", None).await
    }

    pub async fn default_with_addr(addr: Identity) -> Result<Self> {
        Self::connect("127.0.0.1:920", Some(addr)).await
    }

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
        info!("Sending introduction message!");
        write_with_length(&mut socket, &encode_message(online_msg)?).await?;

        trace!("Waiting for ACK message!");
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

        debug!("IPC client initialisation done!");

        // TODO: spawn receive daemon here
        let (tx, recv) = unbounded();
        task::spawn(run_receive(socket.clone(), tx));

        Ok(Self { socket, addr, recv })
    }

    /// Return the currently assigned address
    pub fn address(&self) -> Identity {
        self.addr
    }

    /// Send some data to a remote peer
    pub async fn send_to(&self, recipient: Identity, payload: Vec<u8>) -> Result<()> {
        let msg = api::api_send(api::send_default(message::new(
            self.addr,
            vec![recipient], // recipient
            payload,
            vec![], // signature
        )));

        write_with_length(&mut self.socket.clone(), &encode_message(msg)?).await?;
        Ok(())
    }

    /// Send some data to a remote peer
    pub async fn flood(&self, payload: Vec<u8>) -> Result<()> {
        let msg = api::api_send(api::send_flood(message::new(
            self.addr,
            vec![], // recipient
            payload,
            vec![], // signature
        )));

        write_with_length(&mut self.socket.clone(), &encode_message(msg)?).await?;
        Ok(())
    }

    /// Receive a message sent to this address
    pub async fn next(&self) -> Option<(Receive_Type, Message)> {
        self.recv.recv().await.ok()
    }
}

async fn run_receive(mut socket: TcpStream, tx: Sender<(Receive_Type, Message)>) {
    loop {
        trace!("Reading message from stream...");
        let msg = match read_with_length(&mut socket).await {
            Ok(msg) => msg,
            Err(e) => {
                error!("Failed to read from socket: {:?}", e);
                break;
            }
        };

        trace!("Parsing message from stream...");
        match types::decode_message(&msg).map(|m| m.inner) {
            Ok(Some(one_of)) => match one_of {
                ApiMessageEnum::recv(mut msg) => {
                    let tt = msg.field_type;
                    let msg = msg.take_msg();

                    debug!("Forwarding message to IPC wrapper");
                    if let Err(e) = tx.send((tt, msg)).await {
                        error!("Failed to forward received message: {}", e);
                    }
                }
                _ => {} // This might be a problem idk
            },
            _ => {
                warn!("Invalid payload received; skipping...");
                continue;
            }
        }
    }
}

/// This test is horrible and a bad idea but whatever
#[async_std::test]
#[ignore]
async fn send_message() {
    pub fn setup_logging() {
        use tracing_subscriber::{filter::LevelFilter, fmt, EnvFilter};
        let filter = EnvFilter::default()
            .add_directive(LevelFilter::TRACE.into())
            .add_directive("async_std=error".parse().unwrap())
            .add_directive("async_io=error".parse().unwrap())
            .add_directive("polling=error".parse().unwrap())
            .add_directive("mio=error".parse().unwrap());

        // Initialise the logger
        fmt().with_env_filter(filter).init();
    }

    setup_logging();

    use async_std::task::sleep;
    use std::{process::Command, time::Duration};

    let mut daemon = Command::new("cargo")
        .current_dir("../..")
        .args(&[
            "run",
            "--bin",
            "ratmand",
            "--features",
            "daemon",
            "--",
            "--no-peering",
        ])
        .spawn()
        .unwrap();

    sleep(Duration::from_secs(1)).await;

    let client = RatmanIpc::default().await.unwrap();
    let msg = vec![1, 3, 1, 2];
    info!("Sending message: {:?}", msg);
    client.send_to(client.address(), msg).await.unwrap();

    let recv = client.next().await.unwrap();
    info!("Receiving message: {:?}", recv);
    assert_eq!(recv.get_payload(), &[1, 3, 1, 2]);

    // Exorcise the deamons!
    daemon.kill().unwrap();
}
