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
pub mod socket_v2;

mod subscriber;
pub use subscriber::SubscriptionHandle;

pub mod types;
use types as ty;

#[cfg(test)]
mod test;

use crate::{
    api::{
        socket_v2::RawSocketHandle,
        types::{Handshake, ServerPing, SubsCreate, SubsDelete},
    },
    frame::micro::{client_modes as cm, MicroframeHeader},
    types::Address,
    ClientError, EncodingError, Result,
};
pub use _trait::RatmanIpcExtV1;
use async_trait::async_trait;
use std::{
    collections::BTreeMap,
    ffi::CString,
    net::{AddrParseError, Ipv4Addr, SocketAddr, SocketAddrV4},
    str::FromStr,
    sync::Arc,
};
use tokio::{net::TcpStream, sync::Mutex};

use self::types::SubsRestore;

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

    async fn addr_create(
        self: &Arc<Self>,
        name: Option<String>,
    ) -> crate::Result<(crate::types::Address, crate::types::AddrAuth)> {
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
                        CString::new(n.as_bytes()).expect("Failed to encode String to CString")
                    }),
                },
            )
            .await?;

        let (header, addr) = socket.read_microframe::<Address>().await?;
        Ok((addr, header.auth.unwrap()))
    }

    async fn addr_destroy(
        self: &Arc<Self>,
        auth: crate::types::AddrAuth,
        addr: crate::types::Address,
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

    async fn addr_up(
        self: &Arc<Self>,
        auth: crate::types::AddrAuth,
        addr: crate::types::Address,
    ) -> crate::Result<()> {
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

    async fn addr_down(
        self: &Arc<Self>,
        auth: crate::types::AddrAuth,
        addr: crate::types::Address,
    ) -> crate::Result<()> {
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
        _auth: crate::types::AddrAuth,
        _addr: crate::types::Address,
        _note: Option<String>,
        _tags: BTreeMap<String, String>,
        _trust: u8,
    ) -> crate::Result<crate::types::Ident32> {
        todo!()
    }

    async fn contact_modify(
        self: &Arc<Self>,
        _auth: crate::types::AddrAuth,

        // Selection filter section
        _addr_filter: Vec<crate::types::Address>,
        _note_filter: Option<String>,
        _tags_filter: BTreeMap<String, String>,

        // Modification section
        _note_modify: crate::types::Modify<String>,
        _tags_modify: crate::types::Modify<(String, String)>,
    ) -> crate::Result<Vec<crate::types::Ident32>> {
        todo!()
    }

    async fn contact_delete(
        self: &Arc<Self>,
        _auth: crate::types::AddrAuth,
        _addr: crate::types::Address,
    ) -> crate::Result<()> {
        todo!()
    }

    async fn subs_available(
        self: &Arc<Self>,
        auth: crate::types::AddrAuth,
    ) -> crate::Result<Vec<crate::types::Ident32>> {
        let mut socket = self.socket().lock().await;
        socket
            .write_microframe(
                MicroframeHeader {
                    modes: cm::make(cm::SUB, cm::LIST),
                    auth: Some(auth),
                    ..Default::default()
                },
                (), // no payload needed
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
        auth: crate::types::AddrAuth,
        addr: crate::types::Address,
        recipient: crate::types::Recipient,
    ) -> crate::Result<crate::api::SubscriptionHandle> {
        let mut socket = self.socket().lock().await;
        socket
            .write_microframe(
                MicroframeHeader {
                    modes: cm::make(cm::SUB, cm::CREATE),
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
        auth: crate::types::AddrAuth,
        addr: crate::types::Address,
        req_sub_id: crate::types::Ident32,
    ) -> crate::Result<SubscriptionHandle> {
        let mut socket = self.socket().lock().await;
        socket
            .write_microframe(
                MicroframeHeader {
                    modes: cm::make(cm::SUB, cm::UP),
                    auth: Some(auth),
                    ..Default::default()
                },
                SubsRestore { sub_id: req_sub_id, addr },
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
        auth: crate::types::AddrAuth,
        addr: crate::types::Address,
        sub_id: crate::types::Ident32,
    ) -> crate::Result<()> {
        let mut socket = self.socket().lock().await;
        socket
            .write_microframe(
                MicroframeHeader {
                    modes: cm::make(cm::SUB, cm::CREATE),
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
