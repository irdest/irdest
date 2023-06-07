// SPDX-FileCopyrightText: 2019-2022 Katharina Fey <kookie@spacekookie.de>
//
// SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

//! # Ratman packet router
//!
//! **Note** most likely you are interested in the
//! [libratman](https://docs.rs/ratman) crate, which allows you to
//! connect to a Ratman daemon via an IPC socket, or write a Ratman
//! compatible network driver.
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

mod clock;
mod core;
mod crypto;
mod protocol;
mod scaffold;
mod slicer;
mod storage;

use async_std::channel::{Receiver, Sender};

/// A convevient Sender/Receiver pair for a type
pub(crate) type IoPair<T> = (Sender<T>, Receiver<T>);

pub async fn start_with_configuration() {}
