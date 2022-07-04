// SPDX-FileCopyrightText: 2022 Katharina Fey <kookie@spacekookie.de>
// SPDX-FileCopyrightText: 2022 Lux <lux@lux.name>
//
// SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

//! Ratman daemon entrypoint

extern crate tracing;

use crate::daemon;
pub(crate) use ratman::*;

fn main() {
    daemon::init_daemon();
}
