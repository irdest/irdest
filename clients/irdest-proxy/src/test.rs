use ratmand::{config::ConfigTree, start_with_configuration};

#[test]
fn integration() {
    // Setup router A
    std::thread::spawn(|| {
        let cfg = ConfigTree::default_in_memory()
            .patch("ratmand/api_bind", "127.0.0.1:10999")
            .patch("ratmand/ephemeral", true)
            .patch("inet/bind", "[::]:10900")
            .patch("ratmand/enable_dashboard", false)
            .patch("ratmand/accept_unknown_peers", true)
            .patch("lan/enable", false);

        async_std::task::block_on(start_with_configuration(cfg));
    });

    #[allow(deprecated)]
    std::thread::sleep_ms(100);

    std::thread::spawn(|| {
        let cfg = ConfigTree::default_in_memory()
            .patch_list("ratmand/peers", "inet:localhost:10900")
            .patch("ratmand/api_bind", "127.0.0.1:11999")
            .patch("ratmand/ephemeral", true)
            .patch("inet/bind", "[::]:11900")
            .patch("ratmand/enable_dashboard", false)
            .patch("ratmand/accept_unknown_peers", true)
            .patch("lan/enable", false);

        async_std::task::block_on(start_with_configuration(cfg));
    });

    #[allow(deprecated)]
    std::thread::sleep_ms(100);

    // Now do other magic
}
