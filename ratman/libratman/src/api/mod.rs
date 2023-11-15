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

pub use _trait::RatmanIpcExtV1;

/// Represent a Ratman IPC socket and interfaces
pub struct RatmanIpc {}

impl RatmanIpc {}

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
