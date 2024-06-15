// SPDX-FileCopyrightText: 2019-2022 Katharina Fey <kookie@spacekookie.de>
//
// SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

use libratman::tokio::sync::mpsc::{channel, Receiver, Sender};
use libratman::types::InMemoryEnvelope;

/// A simple I/O wrapper around channels
pub(crate) struct Io {
    pub out: Sender<InMemoryEnvelope>,
    pub inc: Receiver<InMemoryEnvelope>,
}

impl Io {
    pub(crate) fn make_pair() -> (Io, Io) {
        let (a_to_b, b_from_a) = channel(1);
        let (b_to_a, a_from_b) = channel(1);
        let a = Io {
            out: a_to_b,
            inc: a_from_b,
        };
        let b = Io {
            out: b_to_a,
            inc: b_from_a,
        };
        return (a, b);
    }
}
