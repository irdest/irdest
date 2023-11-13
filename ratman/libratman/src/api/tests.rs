use crate::client::RatmanIpc;

/// This test is horrible and a bad idea but whatever
/// also you need to kill the daemon(kill process) after the test
#[async_std::test]
#[ignore]
async fn send_message() {
    pub fn setup_logging() {
        use tracing_subscriber::{filter::LevelFilter, fmt, EnvFilter};
        let filter = EnvFilter::default()
            .add_directive(LevelFilter::TRACE.into())
            .add_directive("async_std=error".parse().unwrap())
            .add_directive("async_io=error".parse().unwrap())
            .add_directive("polling=error".parse().unwrap())
            .add_directive("mio=error".parse().unwrap());

        // Initialise the logger
        fmt().with_env_filter(filter).init();
    }

    setup_logging();

    use async_std::task::sleep;
    use std::{process::Command, time::Duration};

    let mut daemon = Command::new("cargo")
        .current_dir("../..")
        .args(&[
            "run",
            "--bin",
            "ratmand",
            "--features",
            "daemon",
            "--",
            "--no-inet",
            "--accept-unknown-peers",
        ])
        .spawn()
        .unwrap();

    sleep(Duration::from_secs(1)).await;

    let client = RatmanIpc::default().await.unwrap();
    let msg = vec![1, 3, 1, 2];
    info!("Sending message: {:?}", msg);
    client.send_to(client.address(), msg).await.unwrap();

    let (_, recv) = client.next().await.unwrap();
    info!("Receiving message: {:?}", recv);
    assert_eq!(recv.get_payload(), &[1, 3, 1, 2]);

    // Exorcise the deamons!
    daemon.kill().unwrap();
}
