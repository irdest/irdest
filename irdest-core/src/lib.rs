//! This library sits at the core of [Irdest], providing user
//! identities, peer trust management, message sending abstractions,
//! and high-level peer discovery mechanisms.
//!
//! **Note: You most likely do not want to use this library directly, and
//! instead use the [irdest-sdk]!**
//!
//! [Irdest]: https://irde.st
//! [irdest-sdk]: https://docs.irde.st/api/irdest_sdk/index.html
//! 
//! This API (in documentation also called the "service API") enables
//! applications to run on decentralised networks, without relying on
//! servers to facilitate data exchange.
//!
//! You can use the irdest-core API in two ways:
//!
//! 1. Require the `irdest-core` crate as a dependency, and use the
//!    Rust API natively.  This will result in your application
//!    running a decentralised router (See `ratman` crate).
//!
//! 2. Include the `irdest-sdk` crate as a dependency, which exposes
//!    the same API as irdest-core (and other services), but connects
//!    to a running daemon via the QRPC protocol.
//!
//! The second method is recommended for most users, as it will allow
//! many different applications to use the same routing daemon for
//! their tasks.
//!
//!
//! ## Basic architecture
//!
//! In irdest we make the distinction between a **client** (a
//! user-facing entity with some kind of UI), and a **service** (a
//! micro-application, exposing an API to clients, and other
//! services).  A client is in turn implemented _as_ a service!
//!
//! Following is a short overview of a full irdestapplication stack.
//! You can find a more detailed description in the [developer
//! manual](https://docs.irde.st/developer/).
//!
//! | Component            | Description                                                                                                |
//! |----------------------|------------------------------------------------------------------------------------------------------------|
//! | End-user Application | A UI for users to interact with (either graphical, or textual)                                             |
//! | Services             | Micro-applications providing very specificy and re-usable functionality to other services and applications |
//! | RPC System           | (Optional) The IRPC broker and SDK system to connect to an external irdest daemon                          |
//! | irdest-core          | Core identity, data, and peer management library                                                           |
//! | Ratman               | A decentralised, delay-tolerant, userspace router                                                          |
//! | Network drivers      | Platform-specific network driver plugins for Ratman                                                        |
//!
//! When initialising an instance of irdest-core, components need to
//! be initialised in reverse-order (network drivers first, user
//! application state last).
//!
//! ## Example
//!
//! Following is a short example of how to use irdest-core directly.
//! For examples on how to use the SDK, check out the
//! [irdest-sdk](../irdest_sdk) documentation!
//!
//! ```rust,no_run
//! # async fn foo() -> irdest_core::error::Result<()> {
//! # use irdest_core::{helpers::{Directories, ItemDiff}, users::UserUpdate};
//! # let router = todo!();
//! use irdest_core::Irdest;
//! let ird = Irdest::new(router, Directories::new("my-client-id")?);
//!
//! // Create an anonymous user with a password
//! let alice = ird.users().create("password alice secret keeps").await?;
//!
//! // Alice decides she wants to publish her handle
//! let update = UserUpdate { handle: ItemDiff::set("@alice"), ..Default::default() };
//! ird.users().update(alice, update).await?;
//!
//! // irdest will now advertise Alice as `@alice` along side with
//! // her cryptographic public key!
//! # Ok(())
//! # }
//! ```
//!
//! ## Functionality
//!
//! irdest-core handles user registration, sign-in, authentication,
//! binary payload messaging, peer discovery, and management.  Some
//! concepts are not implemented in libqaul directly (such as message
//! groups, or text-payload messages), but instead require a service.
//!
//! For an overview of services written by the qaul project, check
//! [this
//! page](https://docs.irde.st/developer/technical/services.html) in
//! the developer manual!

#![doc(html_favicon_url = "https://irde.st/favicon.ico")]
#![doc(html_logo_url = "https://irde.st/img/qaul_icon-128.png")]
#![allow(warnings)]

#[macro_use]
extern crate tracing;

// Internally re-export types library
pub(crate) use ircore_types as types;

// Internal modules
mod auth;
mod discover;
mod security;
mod store;
mod utils;

// Exposed API modules
pub mod api;
pub mod contacts;
pub mod error;
pub mod helpers;
pub mod messages;
pub mod services;
pub mod users;

#[cfg(feature = "ffi-java")]
pub mod ffi;

#[cfg(feature = "rpc")]
pub mod rpc;

// Core state should be in the root
mod ird;
pub use ird::{Identity, Irdest, IrdestRef};
