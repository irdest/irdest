use ratman::daemon::config::Config;

#[test]
fn integration() {
    // Setup router A
    std::thread::spawn(|| {
        let cfg = Config {
            api_bind: "127.0.0.0:10999".into(),
            inet_bind: "[::]:10900".into(),
            no_discovery: true,
            no_dashboard: true,
            accept_unknown_peers: true,
            ..Default::default()
        };

        async_std::task::block_on(ratman::daemon::startup::run_app(cfg)).unwrap();
    });

    #[allow(deprecated)]
    std::thread::sleep_ms(100);

    std::thread::spawn(|| {
        let cfg = Config {
            api_bind: "127.0.0.0:11999".into(),
            inet_bind: "[::]:11900".into(),
            no_discovery: true,
            no_dashboard: true,
            peers: Some("inet#localhost:10900".into()),
            ..Default::default()
        };

        async_std::task::block_on(ratman::daemon::startup::run_app(cfg)).unwrap();
    });

    #[allow(deprecated)]
    std::thread::sleep_ms(100);

    // Now do other magic
}
