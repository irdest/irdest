// SPDX-FileCopyrightText: 2019-2023 Katharina Fey <kookie@spacekookie.de>
//
// SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

use async_std;
use libratman::netmod::{Endpoint, InMemoryEnvelope, Target};
use netmod_mem::MemMod;

#[async_std::test]
async fn ping_pong() {
    let a = MemMod::new();
    let b = MemMod::new();
    a.link(&b);

    a.send(InMemoryEnvelope::test_envelope(), Target::default(), None)
        .await
        .expect("Failed to send message from a. Error");
    b.next().await.expect("Failed to get message at b. Error");

    b.send(InMemoryEnvelope::test_envelope(), Target::default(), None)
        .await
        .expect("Failed to send message from b. Error");
    a.next().await.expect("Failed to get message at a. Error");
}

#[async_std::test]
async fn split() {
    let a = MemMod::new();
    let b = MemMod::new();
    a.link(&b);
    a.send(InMemoryEnvelope::test_envelope(), Target::default(), None)
        .await
        .expect("Failed to send message from a. Error");
    // Disconnect the two interfaces, so the message sent by A will never be
    // received by B.
    b.split();
    assert!(b.next().await.is_err());
}
