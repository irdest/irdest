//! Ratman client bindings library
//!
//! To learn more about Ratman and Irdest, visit https://irde.st!
//!
//! In order to interact with the Ratman daemon your application must
//! send properly formatted API messages over a local TCP socket.
//! These data formats are outlined in [ratman-types](ratman_types)!
//!
//! This crate provides a simple API over these API messages!
//!
//! **This API is very unstable!** You may expect _slightly_ more
//! stability guarantees from `ratcat(1)` and `ratctl(1)`
//!
//! ## Version numbers
//!
//! The client library MAJOR and MINOR version follow a particular
//! Ratman release.  So for example, version `0.3.0` of this crate is
//! built against version `0.3.0` of `ratmand`.  Because Ratman itself
//! follows semantic versioning, this crate is in turn also
//! semantically versioned.
//!
//! Any change that needs to be applied to this library that does not
//! impact `ratmand` or the stability of this API will be implemented
//! as a patch-version.
//!
//! Also: by default this library will refuse to connect to a running
//! `ratmand` that does not match the libraries version number.  This
//! behaviour can be disabled via the `RatmanIpc` API.

#[macro_use]
extern crate tracing;

use async_std::{
    channel::{unbounded, Receiver, Sender},
    net::TcpStream,
    task,
};
pub use types::{api::Receive_Type, Error, Identity, Result};
use types::{
    api::{
        self, ApiMessageEnum,
        Peers_Type::{DISCOVER, RESP},
        Setup_Type::ACK,
    },
    message::Message,
    encode_message, parse_message, read_with_length, write_with_length,
};

/// An IPC handle for a particular address
///
/// This handle can be cloned safely.  An Ipc handle only refers to a
/// single address connection.  Your application is encouraged to
/// maintain many of these connections at the same time.
#[derive(Clone)]
pub struct RatmanIpc {
    socket: TcpStream,
    addr: Identity,
    recv: Receiver<(Receive_Type, Message)>, 
    disc: Receiver<Identity>,
}

impl RatmanIpc {
    /// Create a `DEFAULT` variant will always register a new address
    pub async fn default() -> Result<Self> {
        Self::connect("127.0.0.1:9020", None).await
    }

    pub async fn default_with_addr(addr: Identity) -> Result<Self> {
        Self::connect("127.0.0.1:9020", Some(addr)).await
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
        let (dtx, disc) = unbounded();
        task::spawn(run_receive(socket.clone(), tx, dtx));

        Ok(Self {
            socket,
            addr,
            recv,
            disc,
        })
    }

    /// Connect to the daemon without providing or wanting an address
    pub async fn anonymous(socket_addr: &str) -> Result<Self> {
        let mut socket = TcpStream::connect(socket_addr).await?;

        let introduction = api::api_setup(api::anonymous());
        write_with_length(&mut socket, &encode_message(introduction)?).await?;

        let addr = Identity::random(); // Never used
        let (_, recv) = unbounded(); // Never used
        let (_, disc) = unbounded(); // Never used
        Ok(Self {
            socket,
            addr,
            recv,
            disc,
        })
    }

    /// Return the currently assigned address
    pub fn address(&self) -> Identity {
        self.addr
    }

    /// Send some data to a remote peer
    pub async fn send_to(&self, recipient: Identity, payload: Vec<u8>) -> Result<()> {
        let msg = api::api_send(api::send_default(Message::new(
            self.addr,
            vec![recipient], // recipient
            payload,
            vec![], // signature
        ).into()));
        
        write_with_length(&mut self.socket.clone(), &encode_message(msg)?).await?;
        Ok(())
    }

    /// Send some data to a remote peer
    pub async fn flood(&self, payload: Vec<u8>) -> Result<()> {
        let msg = api::api_send(api::send_flood(Message::new(
            self.addr,
            vec![], // recipient
            payload,
            vec![], // signature
        ).into()));

        write_with_length(&mut self.socket.clone(), &encode_message(msg)?).await?;
        Ok(())
    }

    /// Receive a message sent to this address
    pub async fn next(&self) -> Option<(Receive_Type, Message)> {
        self.recv.recv().await.ok()
    }

    /// Listen for the next address discovery event
    pub async fn discover(&self) -> Option<Identity> {
        self.disc.recv().await.ok()
    }

    /// Get all currently known peers for this router
    pub async fn get_peers(&self) -> Result<Vec<Identity>> {
        let msg = api::api_peers(api::peers_req());
        write_with_length(&mut self.socket.clone(), &encode_message(msg)?).await?;

        match parse_message(&mut self.socket.clone())
            .await
            .map(|m| m.inner)
        {
            Ok(Some(one_of)) => match one_of {
                ApiMessageEnum::peers(s) if s.field_type == RESP => {
                    Ok(s.peers.iter().map(|p| Identity::from_bytes(p)).collect())
                }
                _ => unreachable!(),
            },
            _ => unreachable!(),
        }
    }
}

async fn run_receive(
    mut socket: TcpStream,
    tx: Sender<(Receive_Type, Message)>,
    dtx: Sender<Identity>,
) {
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
                    if let Err(e) = tx.send((tt, msg.into())).await {
                        error!("Failed to forward received message: {}", e);
                    }
                }
                ApiMessageEnum::peers(peers) if peers.get_field_type() == DISCOVER => {
                    match peers.peers.get(0) {
                        Some(p) => match dtx.send(Identity::from_bytes(p)).await {
                            Ok(_) => {}
                            _ => {
                                error!("Failed to send discovery to client poller...");
                                continue;
                            }
                        },
                        None => continue,
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
/// also you need to kill the daemon(kill process) after the test
#[async_std::test]
/// #[ignore]
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
            "--no-inet",
            "--accept-unknown-peers",
        ])
        .spawn()
        .unwrap();

    sleep(Duration::from_secs(1)).await;

    let client = RatmanIpc::default().await.unwrap();
    let msg = vec![1, 3, 1, 2];
    info!("Sending message: {:?}", msg);
    client.send_to(client.address(), msg).await.unwrap();

    let (_, recv) = client.next().await.unwrap();
    info!("Receiving message: {:?}", recv);
    assert_eq!(recv.get_payload(), &[1, 3, 1, 2]);
    
    // Exorcise the deamons!
    daemon.kill().unwrap();
}
