//! # Irdest RPC SDK
//!
//! A toolkit for writing clients on the irpc message bus, which
//! creates the backbone of the [irdest](https://irde.st) service
//! ecosystem.
//!
//! In the irdest architecture you can think of a services as a set of
//! [actors](https://en.wikipedia.org/wiki/Actor_(programming_language))
//! running across different binaries.  Each service has a name,
//! version, and description chosen by the service authors.  Different
//! services can talk to each other via the
//! [`irpc-broker`](../irpc-broker/).
//!
//! Services come in two flavours: **consumers**, and **producers**.
//! This library is meant for people who want to develop producer
//! services.  To use existing irdest services via the irpc system,
//! check out the [irdest-sdk](../irdest-sdk/) crate.
//!
//! ## Defining a service
//!
//! Every service needs to publish a set of metadata to be reachable
//! via the `irpc-broker`.  The minimal set is a name, version, and
//! description.  This will create a consuming service which can
//! interact with other services on the bus - this would be enough to
//! write a user front-end.
//!
//! If you want to write a service that can be used by other services
//! (such as a front-end), you need to provide two other sets of
//! metadata:
//!
//! - **capabilities**: specify available function endpoints, and
//! their required parameters.  This information is used by the
//! `irpc-broker` to filter messages to avoid crashing older
//! implementations with data that it would not be able to parse.
//!
//! - **dependencies**: a producing service must declare its
//! dependencies before being started, since runtime errors would be
//! hard to communicate to the user.  A consuming service MAY also do
//! this.

#[macro_use]
extern crate tracing;
#[macro_use]
extern crate serde;

mod caps;
mod service;
mod socket;
mod subs;

pub mod error;
pub mod io;
pub mod proto;

pub use caps::{Capabilities, ENCODING_JSON, ENCODING_MSGPACK};
pub use identity::Identity;
pub use service::{Dependencies, Service};
pub use socket::{default_socket_path, RpcSocket};
pub use subs::Subscription;

/// The address of the irpc broker itself
pub const DEFAULT_BROKER_ADDRESS: &'static str = "org.irdest._broker";
