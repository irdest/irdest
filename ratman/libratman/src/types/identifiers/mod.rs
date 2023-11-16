// SPDX-FileCopyrightText: 2023 Katharina Fey <kookie@spacekookie.de>
//
// SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

/// Length of the identity buffer to align with an ed25519 pubkey
pub const ID_LEN: usize = 32;

pub mod address;
pub mod id;
pub mod subnet;
pub mod target;
