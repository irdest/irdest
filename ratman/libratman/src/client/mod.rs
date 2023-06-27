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

#[cfg(feature = "ffi")]
pub mod ffi;

mod socket;

#[cfg(test)]
mod tests;

pub use crate::types::{
    api::Receive_Type, Address, Error, Id, Message, Recipient, Result, TimePair,
};
use async_std::{
    channel::{unbounded, Receiver},
    task,
};

use self::socket::IpcSocket;

/// An IPC handle for a particular address
///
/// This handle can be cloned safely.  An Ipc handle only refers to a
/// single address connection.  Your application is encouraged to
/// maintain many of these connections at the same time.
#[derive(Clone)]
pub struct RatmanIpc {
    socket: IpcSocket,
    addr: Address,
    recv: Receiver<(Receive_Type, Message)>,
    disc: Receiver<Address>,
}

impl RatmanIpc {
    /// Create a `DEFAULT` variant will always register a new address
    pub async fn default() -> Result<Self> {
        Self::connect("127.0.0.1:9020", None).await
    }

    pub async fn default_with_addr(addr: Address) -> Result<Self> {
        Self::connect("127.0.0.1:9020", Some(addr)).await
    }

    /// Connect to a Ratman IPC backend with an optional address
    ///
    /// `socket_addr` refers to the local address the Ratman daemon is
    /// listening on.  `addr` refers to the Ratman cryptographic
    /// routing address associated with your application
    pub async fn connect(socket_addr: &str, addr: Option<Address>) -> Result<RatmanIpc> {
        let socket = match addr {
            Some(registered) => IpcSocket::start_with_address(socket_addr, registered).await,
            None => IpcSocket::start_registration(socket_addr).await,
        }?;

        // TODO: spawn receive daemon here
        let addr = socket.addr;
        let (tx, recv) = unbounded();
        let (dtx, disc) = unbounded();
        task::spawn(socket::run_receive(socket.clone(), tx, dtx));

        Ok(Self {
            socket,
            addr,
            recv,
            disc,
        })
    }

    /// Connect to the daemon without providing or wanting an address
    // TODO: why does this exist? This should really not exist I think
    pub async fn anonymous(socket_addr: &str) -> Result<Self> {
        let socket = IpcSocket::anonymous(socket_addr).await?;
        let addr = socket.addr;
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
    pub fn address(&self) -> Address {
        self.addr
    }

    /// Send some data to a remote peer
    pub async fn send_to(&self, recipient: Address, payload: Vec<u8>) -> Result<()> {
        self.socket.send_to(recipient, payload).await
    }

    /// Send some data to a remote peer
    pub async fn flood(&self, namespace: Address, payload: Vec<u8>, mirror: bool) -> Result<()> {
        self.socket.flood(namespace, payload, mirror).await
    }

    /// Receive a message sent to this address
    pub async fn next(&self) -> Option<(Receive_Type, Message)> {
        self.recv.recv().await.ok()
    }

    /// Listen for the next address discovery event
    pub async fn discover(&self) -> Option<Address> {
        self.disc.recv().await.ok()
    }

    /// Get all currently known peers for this router
    pub async fn get_peers(&self) -> Result<Vec<Address>> {
        self.socket.get_peers().await
    }
}
