//! Encrypted record-oriented database
//!
//! **Experimental:** please note that this database is being written
//! for [irdest](https://git.irde.st/we/irdest).  There will be data
//! retention bugs, and you shouldn't use Alexandria unless you're
//! okay with losing the data you're storing!
//!
//! A multi-payload, zone-encrypting, journaled persistence module,
//! built with low-overhead applications in mind.
//!
//! ## Features
//!
//! * Easy to use database interface
//! * Transactional diff operations
//! * Dynamic queries

#[macro_use]
extern crate tracing;

pub(crate) mod core;
pub(crate) mod crypto;
pub(crate) mod delta;
pub(crate) mod dir;
pub(crate) mod io;
pub(crate) mod meta;
pub(crate) mod notify;
pub(crate) mod store;
pub(crate) mod wire;

pub mod error;
pub mod query;
pub mod record;
pub mod utils;

pub use crate::core::{Builder, Library, Session, SessionsApi, GLOBAL};

pub(crate) type Locked<T> = async_std::sync::RwLock<T>;
