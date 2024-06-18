//! Ratman client bindings library
//!
//! To learn more about Ratman and Irdest, visit https://irde.st!
//!
//! In order to interact with the Ratman daemon your application must
//! send properly formatted API messages over a local TCP socket.
//! These data formats are outlined in the [types
//! module](crate::types)!
//!
//! This crate provides a simple API over these API messages!
//!
//! **This API is currently still very unstable!**
//!
//! ## Version numbers
//!
//! The client library MAJOR and MINOR version follow a particular
//! Ratman release.  So for example, version `0.4.0` of this crate is
//! built against version `0.4.0` of `ratmand`.  Because Ratman itself
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

mod _trait;
pub use _trait::{RatmanIpcExtV1, RatmanStreamExtV1, ReadStream};

mod subscriber;
pub use subscriber::SubscriptionHandle;
use types::SendMany;

pub mod socket_v2;
pub mod types;

#[cfg(test)]
mod test;

use self::types::{self as ty, AddrList};
use crate::{
    api::{
        socket_v2::RawSocketHandle,
        types::{Handshake, RecvOne, SendTo, ServerPing, SubsCreate, SubsDelete, SubsRestore},
    },
    frame::micro::{client_modes as cm, MicroframeHeader},
    types::{error::UserError, AddrAuth, Address, Ident32, LetterheadV1, Modify, Recipient},
    ClientError, EncodingError, Result,
};
use async_trait::async_trait;
use std::{
    collections::BTreeMap,
    ffi::CString,
    net::{Ipv4Addr, SocketAddr, SocketAddrV4},
    sync::Arc,
};
use tokio::{
    io::{AsyncRead, AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    sync::Mutex,
};

/// Indicate the current version of this library.
///
/// If the router and client run different versions, they MUST
/// disconnect if the versions are incompatible.  Versions follow
/// semantic versioning.
pub const VERSION: [u8; 2] = [
    0, // current major version
    1, // current minor version
];

// TODO: replace this with a real semver library?
pub fn versions_compatible(this: [u8; 2], other: [u8; 2]) -> bool {
    match (this, other) {
        // For versions > 1.0 all minor versions are compatible
        ([t_major, _], [o_major, _]) if t_major == o_major && t_major != 0 => true,
        // For versions 0.x only the same minor is compatible
        ([0, t_minor], [0, o_minor]) if t_minor == o_minor => true,
        _ => false,
    }
}

pub fn version_str(v: &[u8; 2]) -> String {
    format!("{}.{}", v[0], v[1])
}

/// Represent a Ratman IPC socket and interfaces
pub struct RatmanIpc {
    socket: Option<Mutex<RawSocketHandle>>,
}

#[async_trait]
impl RatmanIpcExtV1 for RatmanIpc {
    async fn start(bind: SocketAddr) -> Result<Arc<Self>> {
        let mut socket = RawSocketHandle::new(TcpStream::connect(bind).await?);

        socket
            .write_microframe(
                MicroframeHeader {
                    modes: cm::make(cm::INTRINSIC, cm::UP),
                    auth: None,
                    ..Default::default()
                },
                Handshake::new(),
            )
            .await?;

        let (_, ping) = socket.read_microframe::<ServerPing>().await?;

        match ping? {
            ServerPing::Ok => Ok(Arc::new(Self {
                socket: Some(Mutex::new(socket)),
            })),
            ServerPing::IncompatibleVersion { router, client } => {
                Err(ClientError::IncompatibleVersion(
                    router.into_string().unwrap(),
                    client.into_string().unwrap(),
                )
                .into())
            }
            _ => Err(EncodingError::Internal(format!(
                "Invalid response data, this should not happen :(  Please open an issue if it does"
            ))
            .into()),
        }
    }

    async fn addr_list(self: &Arc<Self>) -> crate::Result<Vec<Address>> {
        let mut socket = self.socket().lock().await;

        socket
            .write_microframe(
                MicroframeHeader {
                    modes: cm::make(cm::ADDR, cm::LIST),
                    auth: None,
                    ..Default::default()
                },
                (),
            )
            .await?;

        let (_header, addrs) = socket.read_microframe::<AddrList>().await?;

        addrs.map(|addrs| addrs.list)
    }

    async fn addr_create<'n>(
        self: &Arc<Self>,
        name: Option<&'n String>,
        space_private_key: Option<Ident32>,
    ) -> crate::Result<(Address, AddrAuth)> {
        let mut socket = self.socket().lock().await;

        socket
            .write_microframe(
                MicroframeHeader {
                    modes: cm::make(cm::ADDR, cm::CREATE),
                    auth: None,
                    ..Default::default()
                },
                ty::AddrCreate {
                    name: name.map(|n| {
                        CString::new(n.as_bytes()).expect("failed to encode String to CString")
                    }),
                    namespace_data: space_private_key,
                },
            )
            .await?;

        let (header, addr) = socket.read_microframe::<Address>().await?;

        if let Some(auth) = header.auth {
            eprintln!(
                "Got Address({}) AddrAuth({})",
                addr.pretty_string(),
                auth.token.pretty_string()
            );

            Ok((addr, auth))
        } else {
            Err(crate::RatmanError::ClientApi(ClientError::Internal(
                "address registration failed!".to_string(),
            )))
        }
    }

    async fn addr_destroy(
        self: &Arc<Self>,
        auth: AddrAuth,
        addr: Address,
        force: bool,
    ) -> crate::Result<()> {
        let mut socket = self.socket().lock().await;

        socket
            .write_microframe(
                MicroframeHeader {
                    modes: cm::make(cm::ADDR, cm::DESTROY),
                    auth: Some(auth),
                    ..Default::default()
                },
                ty::AddrDestroy { addr, force },
            )
            .await?;

        let (_, ping) = socket.read_microframe::<ServerPing>().await?;

        match ping? {
            ServerPing::Ok => Ok(()),
            ServerPing::Error(e) => Err(e.into()),
            _ => Err(ClientError::ConnectionLost.into()),
        }
    }

    async fn addr_up(self: &Arc<Self>, auth: AddrAuth, addr: Address) -> crate::Result<()> {
        let mut socket = self.socket().lock().await;

        socket
            .write_microframe(
                MicroframeHeader {
                    modes: cm::make(cm::ADDR, cm::UP),
                    auth: Some(auth),
                    ..Default::default()
                },
                ty::AddrUp { addr },
            )
            .await?;

        let (_, ping) = socket.read_microframe::<ServerPing>().await?;

        match ping? {
            ServerPing::Ok => Ok(()),
            ServerPing::Error(e) => Err(e.into()),
            _ => Err(ClientError::ConnectionLost.into()),
        }
    }

    async fn addr_down(self: &Arc<Self>, auth: AddrAuth, addr: Address) -> crate::Result<()> {
        let mut socket = self.socket().lock().await;

        socket
            .write_microframe(
                MicroframeHeader {
                    modes: cm::make(cm::ADDR, cm::DOWN),
                    auth: Some(auth),
                    ..Default::default()
                },
                ty::AddrDown { addr },
            )
            .await?;

        let (_, ping) = socket.read_microframe::<ServerPing>().await?;

        match ping? {
            ServerPing::Ok => Ok(()),
            ServerPing::Error(e) => Err(e.into()),
            _ => Err(ClientError::ConnectionLost.into()),
        }
    }

    async fn contact_add(
        self: &Arc<Self>,
        _auth: AddrAuth,
        _addr: Address,
        _note: Option<String>,
        _tags: BTreeMap<String, String>,
        _trust: u8,
    ) -> crate::Result<Ident32> {
        todo!(
            "This API endpoint is unimplemented in {}",
            version_str(&crate::api::VERSION)
        );
    }

    async fn contact_modify(
        self: &Arc<Self>,
        _auth: AddrAuth,

        // Selection filter section
        _addr_filter: Vec<Address>,
        _note_filter: Option<String>,
        _tags_filter: BTreeMap<String, String>,

        // Modification section
        _note_modify: Modify<String>,
        _tags_modify: Modify<(String, String)>,
    ) -> crate::Result<Vec<Ident32>> {
        todo!(
            "This API endpoint is unimplemented in {}",
            version_str(&crate::api::VERSION)
        );
    }

    async fn contact_delete(
        self: &Arc<Self>,
        _auth: AddrAuth,
        _addr: Address,
    ) -> crate::Result<()> {
        todo!(
            "This API endpoint is unimplemented in {}",
            version_str(&crate::api::VERSION)
        );
    }

    async fn subs_available(
        self: &Arc<Self>,
        auth: AddrAuth,
        addr: Address,
    ) -> crate::Result<Vec<Ident32>> {
        let mut socket = self.socket().lock().await;
        socket
            .write_microframe(
                MicroframeHeader {
                    modes: cm::make(cm::STREAM, cm::LIST),
                    auth: Some(auth),
                    ..Default::default()
                },
                addr,
            )
            .await?;

        let (_, ping) = socket.read_microframe::<ServerPing>().await?;

        match ping? {
            ServerPing::Update {
                available_subscriptions,
            } => Ok(available_subscriptions),
            ServerPing::Error(e) => Err(e.into()),
            _ => Err(ClientError::ConnectionLost.into()),
        }
    }

    async fn subs_create(
        self: &Arc<Self>,
        auth: AddrAuth,
        addr: Address,
        recipient: Recipient,
    ) -> crate::Result<crate::api::SubscriptionHandle> {
        let mut socket = self.socket().lock().await;
        socket
            .write_microframe(
                MicroframeHeader {
                    modes: cm::make(cm::STREAM, cm::SUB),
                    auth: Some(auth),
                    ..Default::default()
                },
                SubsCreate { addr, recipient },
            )
            .await?;

        let (_, resp) = socket.read_microframe::<ServerPing>().await?;

        match resp? {
            ServerPing::Subscription { sub_id, sub_bind } => {
                let bind_str: String = sub_bind
                    .into_string()
                    .map_err(|e| EncodingError::Internal(e.to_string()))?;

                Ok(SubscriptionHandle {
                    id: sub_id,
                    curr_stream: None,
                    read_from_stream: 0,
                    socket: RawSocketHandle::new(TcpStream::connect(&bind_str).await?),
                })
            }
            ServerPing::Error(e) => Err(e.into()),
            _ => Err(ClientError::ConnectionLost.into()),
        }
    }

    async fn subs_restore(
        self: &Arc<Self>,
        auth: AddrAuth,
        addr: Address,
        req_sub_id: Ident32,
    ) -> crate::Result<SubscriptionHandle> {
        let mut socket = self.socket().lock().await;
        socket
            .write_microframe(
                MicroframeHeader {
                    modes: cm::make(cm::STREAM, cm::RESUB),
                    auth: Some(auth),
                    ..Default::default()
                },
                SubsRestore {
                    sub_id: req_sub_id,
                    addr,
                },
            )
            .await?;

        let (_, resp) = socket.read_microframe::<ServerPing>().await?;

        match resp? {
            ServerPing::Subscription { sub_id, sub_bind } => {
                warn!(
                    "Returned subscription ID ({}) does not match requested ({})",
                    req_sub_id, sub_id
                );

                let bind_str: String = sub_bind
                    .into_string()
                    .map_err(|e| EncodingError::Internal(e.to_string()))?;

                Ok(SubscriptionHandle {
                    id: sub_id,
                    curr_stream: None,
                    read_from_stream: 0,
                    socket: RawSocketHandle::new(TcpStream::connect(&bind_str).await?),
                })
            }
            ServerPing::Error(e) => Err(e.into()),
            _ => Err(ClientError::ConnectionLost.into()),
        }
    }

    async fn subs_delete(
        self: &Arc<Self>,
        auth: AddrAuth,
        addr: Address,
        sub_id: Ident32,
    ) -> crate::Result<()> {
        let mut socket = self.socket().lock().await;
        socket
            .write_microframe(
                MicroframeHeader {
                    modes: cm::make(cm::STREAM, cm::UNSUB),
                    auth: Some(auth),
                    ..Default::default()
                },
                SubsDelete { sub_id, addr },
            )
            .await?;

        let (_, resp) = socket.read_microframe::<ServerPing>().await?;
        match resp? {
            ServerPing::Ok => Ok(()),
            ServerPing::Error(e) => Err(e.into()),
            _ => Err(ClientError::ConnectionLost.into()),
        }
    }
}

impl RatmanIpc {
    fn socket(&self) -> &Mutex<RawSocketHandle> {
        self.socket.as_ref().unwrap()
    }

    async fn shutdown(&self) {
        self.socket()
            .lock()
            .await
            .write_microframe(
                MicroframeHeader {
                    modes: cm::make(cm::INTRINSIC, cm::DOWN),
                    auth: None,
                    payload_size: 0,
                },
                (),
            )
            .await
            .unwrap();
        self.socket().lock().await.shutdown().await.unwrap();
    }
}

impl Drop for RatmanIpc {
    fn drop(&mut self) {
        let socket = core::mem::replace(&mut self.socket, None);
        tokio::task::spawn(async move {
            let this = Self { socket };
            this.shutdown().await;
        });
    }
}

/// Return the default socket bind for the ratmand API socket
///
/// If the local ratmand instance is configured to listen to a different socket
/// this function will not work.
pub fn default_api_bind() -> SocketAddr {
    SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 5852))
}

#[async_trait]
impl RatmanStreamExtV1 for RatmanIpc {
    /// Send a message stream to a single address on the network
    ///
    /// A send action needs a valid authentication token for the address that it
    /// is being sent from.  The letterhead contains metadata about the stream:
    /// what address is sending where, and how much.
    ///
    /// Optionally you can call `.add_send_time()` on the letterhead before
    /// passing it to this function to include the current time in the stream
    /// for the receiving client.
    async fn send_to<I: AsyncRead + Unpin + Send>(
        self: &Arc<Self>,
        auth: AddrAuth,
        letterhead: LetterheadV1,
        data_reader: I,
    ) -> crate::Result<()> {
        let plen = letterhead.stream_size;
        let chunk_size = if plen < 1024 {
            plen
        } else if plen > 1024 && plen < (1024 * 32) {
            4 * 1024
        } else {
            16 * 1025
        };

        let mut socket = self.socket().lock().await;
        socket
            .write_microframe(
                MicroframeHeader {
                    modes: cm::make(cm::SEND, cm::ONE),
                    auth: Some(auth),
                    ..Default::default()
                },
                SendTo { letterhead },
            )
            .await?;

        let mut reader = Box::pin(data_reader);

        let mut remaining = plen;
        loop {
            let mut buf = vec![0_u8; chunk_size.min(remaining) as usize];
            reader.read_exact(&mut buf).await?;
            remaining -= buf.len() as u64;

            println!("Writing chunk to router socket {buf:?}");
            socket.write_buffer(buf).await?;

            if remaining == 0 {
                break;
            }
        }

        Ok(())
    }

    /// Send the same message stream to multiple recipients
    ///
    /// Most of the Letterhead
    async fn send_many<I: AsyncRead + Unpin + Send>(
        self: &Arc<Self>,
        auth: AddrAuth,
        letterheads: Vec<LetterheadV1>,
        data_reader: I,
    ) -> crate::Result<()> {
        let stream_size = letterheads.iter().fold(
            Err(UserError::MissingInput("letterhead>stream-size".to_owned())),
            |res, lh| match res {
                Err(UserError::MissingInput(_)) => Ok(lh.stream_size),
                Ok(prev) if prev == lh.stream_size => Ok(prev),
                Ok(prev) if prev != lh.stream_size => Err(UserError::InvalidInput(
                    "letterhead>stream_size".to_owned(),
                    Some("the same stream-size for all recipients".to_owned()),
                )),
                err => err,
            },
        )?;

        let chunk_size = if stream_size < 1024 {
            stream_size
        } else if stream_size > 1024 && stream_size < (1024 * 32) {
            4 * 1024
        } else {
            16 * 1025
        };

        let mut socket = self.socket().lock().await;
        socket
            .write_microframe_debug(
                MicroframeHeader {
                    modes: cm::make(cm::SEND, cm::MANY),
                    auth: Some(auth),
                    ..Default::default()
                },
                SendMany { letterheads },
            )
            .await?;

        let (_, ping) = socket.read_microframe::<ServerPing>().await?;
        let bind = match ping? {
            ServerPing::SendSocket { socket_bind } => socket_bind,
            ServerPing::Error(e) => return Err(e.into()),
            _ => return Err(EncodingError::Parsing("Invalid payload response!".into()).into()),
        };

        let mut send_s =
            TcpStream::connect(bind.to_str().unwrap().parse::<SocketAddr>().unwrap()).await?;

        let mut reader = Box::pin(data_reader);

        let mut remaining = stream_size;
        let mut stalling = false;
        loop {
            let mut buf = vec![0_u8; chunk_size.min(remaining) as usize];
            let amount_read = reader.read_exact(&mut buf).await?;
            if amount_read == 0 && !stalling {
                eprintln!("Nothing read from sending client: set socket to 'stalling'");
                stalling = true;
            } else if amount_read == 0 && stalling {
                eprintln!("Terminating stalled socket");
                break;
            }
            remaining -= buf.len() as u64;

            println!("Writing chunk to router socket {buf:?}");
            send_s.write_all(&buf).await?;

            if remaining == 0 {
                break;
            }
        }

        drop(send_s);

        let (_, ping) = socket.read_microframe::<ServerPing>().await?;
        match ping? {
            ServerPing::Ok => Ok(()),
            ServerPing::Error(e) => Err(e.into()),
            i => Err(ClientError::Internal(format!("Invalid router response: {i:?}")).into()),
        }
    }

    /// Block this task/ socket to wait for a single incoming message stream
    async fn recv_one<'s>(
        self: &'s Arc<Self>,
        auth: AddrAuth,
        addr: Address,
        to: Recipient,
    ) -> crate::Result<(LetterheadV1, ReadStream<'s>)> {
        let mut socket = self.socket().lock().await;
        socket
            .write_microframe_debug(
                MicroframeHeader {
                    modes: cm::make(cm::RECV, cm::ONE),
                    auth: Some(auth),
                    ..Default::default()
                },
                RecvOne { addr, to },
            )
            .await?;

        let (_, letterhead) = socket.read_microframe_debug::<LetterheadV1>().await?;
        Ok((letterhead?, ReadStream(socket)))
    }

    /// Return an iterator over a stream of letterheads and read streams
    async fn recv_many<'s, I>(
        self: &'s Arc<Self>,
        auth: AddrAuth,
        addr: Address,
        to: Recipient,
        num: u32,
    ) -> crate::Result<I>
    where
        I: Iterator<Item = (LetterheadV1, ReadStream<'s>)>,
    {
        // let mut socket = self.socket().lock().await;
        // socket
        //     .write_microframe(
        //         MicroframeHeader {
        //             modes: cm::make(cm::RECV, cm::MANY),
        //             auth: Some(auth),
        //             ..Default::default()
        //         },
        //         RecvMany { addr, to, num },
        //     )
        //     .await?;

        todo!(
            "This API endpoint is unimplemented in {}",
            version_str(&crate::api::VERSION)
        );
        // 0..num.into_iter().map(|_| {
        //     let join =
        //         spawn_local(async move { socket.read_microframe::<LetterheadV1>().await.unwrap() });

        //     join.

        //     todo!()
        // })
    }
}
