// SPDX-FileCopyrightText: 2019-2022 Katharina Fey <kookie@spacekookie.de>
//
// SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

//! Sending messages over a TCP-connected router system

use async_std::task;
use netmod_inet::InetEndpoint;
use ratman::{Address, Router};
use std::time::Duration;
use tracing::warn;
use tracing_subscriber::{filter::LevelFilter, fmt, EnvFilter};

pub fn setup_logging(lvl: &str) {
    let filter = EnvFilter::default()
        .add_directive(match lvl {
            "trace" => LevelFilter::TRACE.into(),
            "debug" => LevelFilter::DEBUG.into(),
            "info" => LevelFilter::INFO.into(),
            "warn" => LevelFilter::WARN.into(),
            "error" => LevelFilter::ERROR.into(),
            _ => unreachable!(),
        })
        .add_directive("async_std=error".parse().unwrap())
        .add_directive("async_io=error".parse().unwrap())
        .add_directive("polling=error".parse().unwrap())
        .add_directive("trust_dns_proto=error".parse().unwrap())
        .add_directive("trust_dns_resolver=warn".parse().unwrap())
        .add_directive("mio=error".parse().unwrap());

    // Initialise the logger
    if let Err(_) = fmt().with_env_filter(filter).try_init() {
        warn!("Logger already initialised");
    }
}

#[async_std::test]
async fn announce_over_inet() {
    setup_logging("info");

    // Device A
    let ep1 = InetEndpoint::start("[::0]:7100").await.unwrap();

    // Device C
    let ep3 = InetEndpoint::start("[::0]:7101").await.unwrap();

    // Make devices A and C connect to B
    ep1.add_peers(vec!["[::1]:7101".into()]).await.unwrap();

    let r1 = Router::new();
    let r3 = Router::new();

    r1.add_endpoint(ep1).await;
    r3.add_endpoint(ep3).await;

    /////// Create some identities and announce people

    let u1 = r1.add_user().await.unwrap();
    r1.online(u1).await.unwrap();

    let u3 = r3.add_user().await.unwrap();
    r3.online(u3).await.unwrap();

    task::sleep(Duration::from_millis(500)).await;

    // The routers will now start announcing their new users on the
    // micro-network.  You can now poll for new user discoveries.
    assert_eq!(r1.discover().await, u3);
    assert_eq!(r3.discover().await, u1);
}

#[async_std::test]
async fn flood_over_inet() {
    setup_logging("info");

    // Device A
    let ep1 = InetEndpoint::start("[::0]:7200").await.unwrap();

    // Device B
    let ep2 = InetEndpoint::start("[::0]:7210").await.unwrap();

    // Device C
    let ep3 = InetEndpoint::start("[::0]:7201").await.unwrap();

    // FIXME: peering in the other direction breaks the unit test ONLY
    // when cargo is running all of them at once.  Why?? I do not know
    ep1.add_peers(vec!["[::1]:7210".into()]).await.unwrap();
    ep3.add_peers(vec!["[::1]:7210".into()]).await.unwrap();

    let r1 = Router::new();
    let _r2 = Router::new();
    let r3 = Router::new();

    r1.add_endpoint(ep1).await;
    r1.add_endpoint(ep2).await;
    r3.add_endpoint(ep3).await;

    /////// Create some identities and announce people

    let u1 = r1.add_user().await.unwrap();
    r1.online(u1).await.unwrap();

    let u3 = r3.add_user().await.unwrap();
    r3.online(u3).await.unwrap();

    let flood_ns = Address::random();

    task::sleep(Duration::from_millis(500)).await;
    r1.flood(u1, flood_ns, vec![1, 3, 1, 2], vec![])
        .await
        .unwrap();

    let recv = r3.next().await;
    assert_eq!(recv.payload, vec![1, 3, 1, 2]);
}
