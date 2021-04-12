//! libqaul message tests

#![allow(unused)]

mod harness;
use harness::{millis, sec10, sec5, zzz};

use irdest_core::{
    helpers::TagSet,
    messages::{IdType, Mode, MsgQuery},
    users::UserAuth,
    Identity, Irdest,
};
use std::{sync::Arc, time::Instant};

async fn send_simple(q: &Arc<Irdest>, auth: &UserAuth, target: Identity) -> Identity {
    dbg!(q
        .messages()
        .send(
            auth.clone(),
            Mode::Std(target),
            IdType::unique(),
            "org.qaul.testing",
            TagSet::empty(),
            vec![1 as u8, 3, 1, 2],
        )
        .await
        .unwrap())
}

#[async_std::test]
#[ignore]
async fn send_one() {
    let net = harness::init().await;
    let auth_a = net.a().users().create("abc").await.unwrap();
    let auth_b = net.b().users().create("abc").await.unwrap();

    // The announcements need to spread
    zzz(millis(2000)).await;

    // Send a message from a
    let id = send_simple(net.a(), &auth_a, auth_b.0).await;

    let msg = harness::timeout(sec5(), async {
        let b = Arc::clone(net.b());
        loop {
            let mut all = b
                .messages()
                .query(auth_b.clone(), "org.qaul.testing", MsgQuery::new())
                .await
                .unwrap()
                .all()
                .await
                .unwrap();

            if all.len() > 0 {
                break all.remove(0);
            } else {
                harness::zzz(millis(20)).await;
            }
        }
    })
    .await
    .unwrap();

    assert_eq!(msg.id, id);
}

#[async_std::test]
#[ignore]
async fn send_three() {
    let net = harness::init().await;
    let auth_a = net.a().users().create("abc").await.unwrap();
    let auth_b = net.b().users().create("abc").await.unwrap();

    // The announcements need to spread
    zzz(millis(2000)).await;

    let t1 = Instant::now();

    dbg!(send_simple(net.a(), &auth_a, auth_b.0).await);
    dbg!(send_simple(net.a(), &auth_a, auth_b.0).await);
    dbg!(send_simple(net.a(), &auth_a, auth_b.0).await);

    harness::timeout(sec10() * 2, async {
        let b = Arc::clone(&net.b());
        while b
            .messages()
            .query(auth_b.clone(), "org.qaul.testing", MsgQuery::new())
            .await
            .unwrap()
            .all()
            .await
            .unwrap()
            .len()
            != 3
        {
            zzz(millis(20)).await
        }

        dbg!("?");

        println!(
            "Message transmission took {} millis",
            (Instant::now() - t1).as_millis()
        );
    })
    .await
    .unwrap();
}

#[async_std::test]
#[ignore]
async fn grouped_send_ids() {
    let net = harness::init().await;
    let auth_a = net.a().users().create("abc").await.unwrap();
    let auth_b = net.b().users().create("abc").await.unwrap();

    zzz(millis(2000)).await;

    let id_type = IdType::group(Identity::random());

    let id = net
        .a()
        .messages()
        .send(
            auth_a.clone(),
            Mode::Std(auth_b.0),
            id_type,
            "org.qaul.testing",
            TagSet::empty(),
            vec![1 as u8, 3, 1, 2],
        )
        .await
        .unwrap();

    assert_eq!(id, id_type.consume());

    let msg = net
        .a()
        .messages()
        .query(auth_a.clone(), "org.qaul.testing", MsgQuery::id(id))
        .await
        .unwrap()
        .resolve()
        .await
        .remove(0);

    assert_eq!(msg.id, id_type.consume());
}
