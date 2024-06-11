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
        types::{Handshake, ServerPing},
    },
    frame::micro::{client_modes as cm, MicroframeHeader},
    rt::new_async_thread,
    types::Address,
    ClientError, EncodingError, Result,
};
pub use _trait::RatmanIpcExtV1;
use async_trait::async_trait;
use std::{collections::BTreeMap, ffi::CString, net::SocketAddr, sync::Arc};
use tokio::{net::TcpStream, runtime::Runtime, spawn, sync::Mutex, task::spawn_local};

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
        // let msg = encode_micro_frame(
        //     cm::make(cm::ADDR, cm::DELETE),
        //     Some(auth),
        //     Some(ty::AddrDestroy { addr, force }),
        // )?;

        todo!()
    }

    async fn addr_up(
        self: &Arc<Self>,
        auth: crate::types::AddrAuth,
        addr: crate::types::Address,
    ) -> crate::Result<()> {
        // let msg = encode_micro_frame(
        //     cm::make(cm::ADDR, cm::UP),
        //     Some(auth),
        //     Some(ty::AddrUp { addr }),
        // )?;

        todo!()
    }

    async fn addr_down(
        self: &Arc<Self>,
        auth: crate::types::AddrAuth,
        addr: crate::types::Address,
    ) -> crate::Result<()> {
        // let msg = encode_micro_frame(
        //     cm::make(cm::ADDR, cm::DOWN),
        //     Some(auth),
        //     Some(ty::AddrUp { addr }),
        // )?;

        todo!()
    }

    async fn contact_add(
        self: &Arc<Self>,
        auth: crate::types::AddrAuth,
        addr: crate::types::Address,
        note: Option<String>,
        tags: BTreeMap<String, String>,
        trust: u8,
    ) -> crate::Result<crate::types::Ident32> {
        // let msg = encode_micro_frame(
        //     cm::make(cm::CONTACT, cm::ADD),
        //     Some(auth),
        //     Some(ty::ContactAdd::new(addr, note, tags.into_iter(), trust)),
        // )?;

        todo!()
    }

    async fn contact_modify(
        self: &Arc<Self>,
        auth: crate::types::AddrAuth,

        // Selection filter section
        addr_filter: Vec<crate::types::Address>,
        note_filter: Option<String>,
        tags_filter: BTreeMap<String, String>,

        // Modification section
        note_modify: crate::types::Modify<String>,
        tags_modify: crate::types::Modify<(String, String)>,
    ) -> crate::Result<Vec<crate::types::Ident32>> {
        let modes = cm::make(cm::CONTACT, cm::MODIFY);
        todo!()
    }

    async fn contact_delete(
        self: &Arc<Self>,
        auth: crate::types::AddrAuth,
        addr: crate::types::Address,
    ) -> crate::Result<()> {
        let modes = cm::make(cm::CONTACT, cm::DELETE);
        todo!()
    }

    async fn subs_available(
        self: &Arc<Self>,
        auth: crate::types::AddrAuth,
    ) -> crate::Result<Vec<crate::types::Ident32>> {
        // let msg = encode_micro_frame::<()>(cm::make(cm::SUB, cm::LIST), Some(auth), None)?;

        todo!()
    }

    async fn subs_create(
        self: &Arc<Self>,
        auth: crate::types::AddrAuth,
        subscription_recipient: crate::types::Recipient,
    ) -> crate::Result<crate::api::SubscriptionHandle> {
        // let msg = encode_micro_frame(
        //     cm::make(cm::CONTACT, cm::ADD),
        //     Some(auth),
        //     Some(ty::SubsCreate {
        //         recipient: subscription_recipient,
        //     }),
        // )?;

        // self.socket.write_buffer(msg).await?;

        todo!()
    }

    async fn subs_restore(
        self: &Arc<Self>,
        auth: crate::types::AddrAuth,
        sub_id: crate::types::Ident32,
    ) -> crate::Result<SubscriptionHandle> {
        todo!()
    }

    async fn subs_delete(
        self: &Arc<Self>,
        auth: crate::types::AddrAuth,
        subscription_id: crate::types::Ident32,
    ) -> crate::Result<()> {
        todo!()
    }
}

impl RatmanIpc {
    fn socket(&self) -> &Mutex<RawSocketHandle> {
        self.socket.as_ref().unwrap()
    }

    // async fn shutdown(&self) {
    //     self.socket()
    //         .lock()
    //         .await
    //         .write_microframe(
    //             MicroframeHeader {
    //                 modes: cm::make(cm::INTRINSIC, cm::DOWN),
    //                 auth: None,
    //                 payload_size: 0,
    //             },
    //             (),
    //         )
    //         .await
    //         .unwrap();
    //     self.socket().lock().await.shutdown().await.unwrap();
    // }
}

// impl Drop for RatmanIpc {
//     fn drop(&mut self) {
//         let socket = core::mem::replace(&mut self.socket, None);
//         spawn(async move {
//             let this = Self { socket };
//             this.shutdown().await;
//         });
//     }
// }

// pub struct IpcSocket(RawSocketHandle, Receiver<(Letterhead, Vec<u8>)>);

// impl IpcSocket {
//     async fn connect_to(
//         addr: impl ToSocketAddrs,
//         sender: Sender<(MicroframeHeader, Vec<u8>)>,
//     ) -> Result<Self> {
//         let socket = TcpStream::connect(addr).await?;
//         Ok(Self(RawSocketHandle::new(socket, sender)))
//     }

//     pub async fn default_address() -> Result<IpcSocket> {
//         let (send, recv) = channel(4);
//         let inner = Self::connect_to("localhost:5862", send).await?;

//         todo!()
//     }
// }
