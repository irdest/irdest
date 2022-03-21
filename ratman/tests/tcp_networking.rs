//! Sending messages over a TCP-connected router system

use netmod_inet::{Endpoint, Mode};
use ratman::{Identity, Router};
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
    fmt().with_env_filter(filter).init();
}

#[async_std::test]
async fn announce_over_inet() {
    setup_logging("trace");

    // Device A
    let ep1 = Endpoint::new("127.0.0.1:7100", "", Mode::Static)
        .await
        .unwrap();

    // Device C
    let ep3 = Endpoint::new("127.0.0.1:7101", "", Mode::Static)
        .await
        .unwrap();

    // Make devices A and C connect to B
    ep1.add_peers(vec!["127.0.0.1:7101".into()]).await.unwrap();
    ep3.add_peers(vec!["127.0.0.1:7100".into()]).await.unwrap();

    let r1 = Router::new();
    let r3 = Router::new();

    r1.add_endpoint(ep1).await;
    r3.add_endpoint(ep3).await;

    /////// Create some identities and announce people

    let u1 = Identity::random();
    r1.add_user(u1).await.unwrap();
    r1.online(u1).await.unwrap();

    let u3 = Identity::random();
    r3.add_user(u3).await.unwrap();
    r3.online(u3).await.unwrap();

    // The routers will now start announcing their new users on the
    // micro-network.  You can now poll for new user discoveries.
    assert_eq!(r1.discover().await, u3);
    assert_eq!(r3.discover().await, u1);
}
