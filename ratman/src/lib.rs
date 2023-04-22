// SPDX-FileCopyrightText: 2019-2022 Katharina Fey <kookie@spacekookie.de>
//
// SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

//! # Ratman
//!
//! **Note** most likely you are interested in the
//! [ratman-client](https://docs.rs/ratman-client) crate, which allows
//! you to connect to a Ratman daemon via an IPC socket.
//!
//! ## License
//!
//! Ratman is part of the Irdest project, and licensed under the [GNU
//! Affero General Public License version 3 or
//! later](../licenses/agpl-3.0.md).
//!
//! See the Irdest repository README for additional permissions
//! granted by the authors for this code.

#[macro_use]
extern crate tracing;

pub mod scaffold;

pub mod clock;
mod core;
mod crypto;
mod data;
mod protocol;
mod router;
mod slicer;
mod storage;

#[cfg(feature = "daemon")]
pub mod daemon;

use async_std::channel::{Receiver, Sender};

// Provide exports to the rest of the crate
pub(crate) use crate::{core::Core, data::Payload, protocol::Protocol, slicer::TransportSlicer};
pub(crate) type IoPair<T> = (Sender<T>, Receiver<T>);

// Public API facade
pub use crate::{
    data::{Message, MsgId},
    router::Router,
};
pub use netmod;
pub use types::{Address, Error, Recipient, Result, TimePair, ID_LEN};
