// SPDX-FileCopyrightText: 2019-2022 Katharina Fey <kookie@spacekookie.de>
//
// SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

use ratman_netmod::Frame;
/// A `Frame` tagged with an identifier and a time to live, for propagation through
/// a simulated medium.
pub(crate) struct TaggedFrame {
    pub tag: u32,
    pub ttl: u32,
    pub frame: Frame,
}

impl TaggedFrame {
    pub(crate) fn new(tag: u32, ttl: u32, frame: Frame) -> Self {
        Self { tag, ttl, frame }
    }
}
