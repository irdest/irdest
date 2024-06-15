// SPDX-FileCopyrightText: 2019-2023 Katharina Fey <kookie@spacekookie.de>
//
// SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

use libratman::{
    endpoint::EndpointExt,
    tokio,
    types::{Ident32, InMemoryEnvelope, Neighbour},
};
use netmod_mem::MemMod;

#[tokio::test]
async fn ping_pong() {
    let a_kid = Ident32::random();
    let a = MemMod::new(a_kid);

    let b_kid = Ident32::random();
    let b = MemMod::new(b_kid);
    a.link(&b).await;

    a.send(
        InMemoryEnvelope::test_envelope(),
        Neighbour::Single(b_kid),
        None,
    )
    .await
    .expect("Failed to send message from a. Error");
    b.next().await.expect("Failed to get message at b. Error");

    b.send(
        InMemoryEnvelope::test_envelope(),
        Neighbour::Single(a_kid),
        None,
    )
    .await
    .expect("Failed to send message from b. Error");
    a.next().await.expect("Failed to get message at a. Error");
}

#[tokio::test]
async fn split() {
    let a_kid = Ident32::random();
    let a = MemMod::new(a_kid);

    let b_kid = Ident32::random();
    let b = MemMod::new(b_kid);
    a.link(&b).await;
    a.send(
        InMemoryEnvelope::test_envelope(),
        Neighbour::Single(b_kid),
        None,
    )
    .await
    .expect("Failed to send message from a. Error");
    // Disconnect the two interfaces, so the message sent by A will never be
    // received by B.
    b.split().await;
    assert!(b.next().await.is_err());
}
