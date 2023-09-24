pub use crate::types::{api::Receive_Type, Address, Id, Message, Recipient, Result, TimePair};
use crate::types::{
    api::{
        self, ApiMessageEnum,
        Peers_Type::{DISCOVER, RESP},
        Setup_Type::{self, ACK},
    },
    encode_message, parse_message, read_with_length, write_with_length, parse_message_nosey,
};
use async_std::{channel::Sender, net::TcpStream};

/// Abstraction for the Ratman API/ IPC socket connection
#[derive(Clone)]
pub struct IpcSocket {
    pub(crate) inner: TcpStream,
    /// Primary address that is registered to this socket
    // TODO: switch this to the `AddressBook` abstraction
    pub(crate) addr: Address,
    pub(crate) token: Id,
}

pub(crate) enum AddressRecipient {
    Network(Address),
    // FIXME pls
    Router(String),
}

impl IpcSocket {
    /// Connect to the IPC backend with a given bind location and an
    /// already registered address
    pub(crate) async fn start_with_address(bind: &str, addr: Address, token: Id) -> Result<Self> {
        Self::connect(bind, Some(addr), Some(token)).await
    }

    /// Connect to the IPC backend with a given bind location and
    /// start registering a new random address
    pub(crate) async fn start_registration(bind: &str) -> Result<Self> {
        Self::connect(bind, None, None).await
    }

    /// Connect to the daemon without providing or wanting an address
    // TODO: why does this exist? This should really not exist I think
    pub async fn anonymous(socket_addr: &str) -> Result<Self> {
        let mut inner = TcpStream::connect(socket_addr).await?;
        let introduction = api::api_setup(api::anonymous());
        write_with_length(&mut inner, &encode_message(introduction)?).await?;
        Ok(Self {
            inner,
            addr: Address::random(),
            token: Id::random(),
        })
    }

    async fn connect(socket_addr: &str, addr: Option<Address>, token: Option<Id>) -> Result<Self> {
        let mut inner = TcpStream::connect(socket_addr).await?;

        // Introduce ourselves to the daemon
        let online_msg = api::api_setup(match (addr, token) {
            (Some(addr), Some(token)) => api::online(addr, token),
            _ => api::online_init(),
        });

        debug!("Sending introduction message!");
        write_with_length(&mut inner, &encode_message(online_msg)?).await?;
        trace!("Waiting for ACK message!");

        // Then wait for a response and assign the used address
        let (addr, token) = match parse_message(&mut inner).await.map(|m| m.inner) {
            Ok(Some(one_of)) => match one_of {
                ApiMessageEnum::setup(ref s) if s.field_type == ACK => {
                    if s.id.len() > 0 && s.token.len() > 0 {
                        (
                            Address::from_bytes(s.get_id()),
                            Id::from_bytes(s.get_token()),
                        )
                    } else {
                        panic!("failed to initialise new address!");
                    }
                }
                _ => unreachable!(
                    "make sure that your libratman version matches the ratmand version!"
                ),
            },
            err => panic!("failed to authenticate: {:?}", err),
        };

        debug!("IPC client initialisation done!");
        Ok(Self { inner, addr, token })
    }

    /// Send some data to a remote peer
    pub async fn send_to(&self, recipient: Address, payload: Vec<u8>) -> Result<()> {
        let msg = api::api_send(api::send_default(
            Message::new(
                self.addr,
                vec![recipient], // recipient
                payload,
                vec![], // signature
            )
            .into(),
        ));

        write_with_length(&mut self.inner.clone(), &encode_message(msg)?).await?;
        Ok(())
    }

    pub async fn ping(&self, recipient: &str) -> Result<String> {
        let msg = api::api_setup(api::ping(recipient.to_owned()).into());
        write_with_length(&mut self.inner.clone(), &encode_message(msg)?).await?;

        eprintln!("Sending always works!");
        match parse_message_nosey(&mut self.inner.clone())
            .await
            .map(|m| m.inner)
        {
            Ok(Some(one_of)) => match one_of {
                ApiMessageEnum::setup(s) if s.field_type == Setup_Type::PING => {
                    Ok(String::from_utf8(s.id).unwrap_or_else(|_| "<invalid utf-8".to_string()))
                }
                _ => unreachable!(),
            },
            _ => unreachable!(),
        }
    }

    /// Send some data to a remote peer
    pub async fn flood(&self, namespace: Address, payload: Vec<u8>, mirror: bool) -> Result<()> {
        let msg = api::api_send(api::send_flood(
            Message::new(
                self.addr,
                vec![], // recipient
                payload,
                vec![], // signature
            )
            .into(),
            namespace,
            mirror,
        ));

        write_with_length(&mut self.inner.clone(), &encode_message(msg)?).await?;
        Ok(())
    }

    /// Get all currently known peers for this router
    pub async fn get_peers(&self) -> Result<Vec<Address>> {
        let msg = api::api_peers(api::peers_req());
        write_with_length(&mut self.inner.clone(), &encode_message(msg)?).await?;

        match parse_message(&mut self.inner.clone())
            .await
            .map(|m| m.inner)
        {
            Ok(Some(one_of)) => match one_of {
                ApiMessageEnum::peers(s) if s.field_type == RESP => {
                    Ok(s.peers.iter().map(|p| Address::from_bytes(p)).collect())
                }
                _ => unreachable!(),
            },
            _ => unreachable!(),
        }
    }
}

pub(super) async fn run_receive(
    mut socket: IpcSocket,
    tx: Sender<(Receive_Type, Message)>,
    dtx: Sender<Address>,
) {
    loop {
        trace!("Reading message from stream...");
        let msg = match read_with_length(&mut socket.inner).await {
            Ok(msg) => msg,
            Err(e) => {
                error!("Failed to read from socket: {:?}", e);
                break;
            }
        };

        trace!("Parsing message from stream...");
        match crate::types::decode_message(&msg).map(|m| m.inner) {
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
                        Some(p) => match dtx.send(Address::from_bytes(p)).await {
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
