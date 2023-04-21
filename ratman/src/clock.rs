// SPDX-FileCopyrightText: 2019-2022 Katharina Fey <kookie@spacekookie.de>
//
// SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

//! Enables router internal clock manipulation
//!
//! This module largely re-exposes the [`clockctrl`] crate and API.
//! For detailed instructions on how to use this API, check these
//! crate docs instead.
//!
//! [`clockctrl`]: https://docs.rs/clockctrl

// Re-export the entire clockctrl crate here for convenience
pub use clockctrl::*;

/// A collection of tasks running inside the Ratman router
#[derive(Hash, Ord, PartialOrd, Eq, PartialEq)]
pub enum Tasks {
    /// Periodically tries to send undeliverable frames
    Journal,
    /// Waits for local addressed frames to desequence them
    Collector,
    /// Main router poll loop checking for new frames
    Switch,
}
