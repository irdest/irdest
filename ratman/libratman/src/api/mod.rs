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

pub mod types;
use types as ty;

pub use _trait::RatmanIpcExtV1;

use crate::{
    frame::micro::{client_modes as cm, create_micro_frame},
    types::ClientAuth,
    Result,
};
use async_trait::async_trait;
use std::{collections::BTreeMap, ffi::CString, sync::Arc, time::Duration};

/// Represent a Ratman IPC socket and interfaces
pub struct RatmanIpc {}

#[async_trait]
impl RatmanIpcExtV1 for RatmanIpc {
    async fn start(&mut self) {
        todo!()
    }

    async fn register_client(self: &Arc<Self>) -> Result<ClientAuth> {
        todo!()
    }

    async fn addr_create(
        self: &Arc<Self>,
        auth: ClientAuth,
        name: Option<String>,
    ) -> crate::Result<crate::types::Address> {
        let msg = create_micro_frame(
            cm::make(cm::ADDR, cm::CREATE),
            Some(auth),
            Some(ty::AddrCreate {
                name: name.map(|n| {
                    CString::new(n.as_bytes()).expect("Failed to encode String to CString")
                }),
            }),
        )?;

        todo!()
    }

    async fn addr_destroy(
        self: &Arc<Self>,
        auth: crate::types::ClientAuth,
        addr: crate::types::Address,
        force: bool,
    ) -> crate::Result<()> {
        let msg = create_micro_frame(
            cm::make(cm::ADDR, cm::DELETE),
            Some(auth),
            Some(ty::AddrDelete { addr, force }),
        )?;

        todo!()
    }

    async fn addr_up(
        self: &Arc<Self>,
        auth: crate::types::ClientAuth,
        addr: crate::types::Address,
    ) -> crate::Result<()> {
        let msg = create_micro_frame(
            cm::make(cm::ADDR, cm::UP),
            Some(auth),
            Some(ty::AddrUp { addr }),
        )?;

        todo!()
    }

    async fn addr_down(
        self: &Arc<Self>,
        auth: crate::types::ClientAuth,
        addr: crate::types::Address,
    ) -> crate::Result<()> {
        let msg = create_micro_frame(
            cm::make(cm::ADDR, cm::DOWN),
            Some(auth),
            Some(ty::AddrUp { addr }),
        )?;

        todo!()
    }

    async fn contact_add(
        self: &Arc<Self>,
        auth: crate::types::ClientAuth,
        addr: crate::types::Address,
        note: Option<String>,
        tags: BTreeMap<String, String>,
        trust: u8,
    ) -> crate::Result<crate::types::Id> {
        let msg = create_micro_frame(
            cm::make(cm::CONTACT, cm::ADD),
            Some(auth),
            Some(ty::ContactAdd::new(addr, note, tags.into_iter(), trust)),
        )?;

        todo!()
    }

    async fn contact_modify(
        self: &Arc<Self>,
        auth: crate::types::ClientAuth,

        // Selection filter section
        addr_filter: Vec<crate::types::Address>,
        note_filter: Option<String>,
        tags_filter: BTreeMap<String, String>,

        // Modification section
        note_modify: crate::types::Modify<String>,
        tags_modify: crate::types::Modify<(String, String)>,
    ) -> crate::Result<Vec<crate::types::Id>> {
        let modes = cm::make(cm::CONTACT, cm::MODIFY);
        todo!()
    }

    async fn contact_delete(
        self: &Arc<Self>,
        auth: crate::types::ClientAuth,
        addr: crate::types::Address,
    ) -> crate::Result<()> {
        let modes = cm::make(cm::CONTACT, cm::DELETE);
        todo!()
    }

    async fn subs_add(
        self: &Arc<Self>,
        auth: crate::types::ClientAuth,
        subscription_recipient: crate::types::Recipient,
        synced: bool,
        timeout: Option<Duration>,
    ) -> crate::Result<crate::types::Id> {
        let modes = cm::make(cm::SUB, cm::ADD);
        todo!()
    }
}

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
