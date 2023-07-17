// SPDX-FileCopyrightText: 2023 Katharina Fey <kookie@spacekookie.de>
//
// SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

//! Central point for various error codes that can be returned from
//! the Ratmand daemon.  Currently these are set as the status code in
//! case the daemon needs to terminate due to an error.

/// Everything is ok actually
pub const SUCCESS: u16 = 0;

/// A general fatal error
pub const FATAL: u16 = u16::MAX;

/// During initialisation a wrong parameter was provided
pub const INVALID_PARAM: u16 = 2;

/// During initialisation a wrong parameter was provided
pub const INVALID_CONFIG: u16 = 10;
