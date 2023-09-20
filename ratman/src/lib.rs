// SPDX-FileCopyrightText: 2019-2023 Katharina Fey <kookie@spacekookie.de>
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

mod api;
mod clock;
mod context;
mod core;
mod crypto;
mod dispatcher;
mod protocol;
mod storage;

#[cfg(feature = "dashboard")]
mod web;

pub mod config;
pub mod util;

/// Start a new Ratman router instance with a launch configuration
///
/// When embedding Ratman into an existing application context (i.e. a
/// mobile app), take care to provide a [Config](crate::util::Config)
/// that will initialise drivers and OS operations correctly.
///
/// Special permissions may be required for certain features!
pub async fn start_with_configuration(cfg: config::ConfigTree) {
    // TODO: this function currently doesn't return at all.  Instead,
    // what we want to do is listen to various signals here and
    // respond to them.
    let _ctx = context::RatmanContext::start(cfg).await;
}
