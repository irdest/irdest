#![no_main]
use libfuzzer_sys::fuzz_target;

use lazy_static::lazy_static;
use std::sync::Arc;
use ratman::{Router, Identity};
use netmod_mem::MemMod;
use netmod_fuzz::FuzzEndpoint;

lazy_static! {
    static ref ROUTER: Arc<FuzzEndpoint> = {
        async_std::task::block_on(async {
            let ep = Arc::new(FuzzEndpoint::new());

            let (m1, m2) = MemMod::make_pair();

            let r1 = Router::new();
            let r2 = Router::new();

            r1.add_endpoint(m1).await;
            r2.add_endpoint(m2).await;
            r2.add_endpoint(ep.clone()).await;

            let u1 = Identity::truncate(&[1u8; 32]);
            let u2 = Identity::truncate(&[2u8; 32]);

            r1.add_user(u1).await.unwrap();
            r2.add_user(u2).await.unwrap();

            r1.online(u1).await.unwrap();
            r2.online(u2).await.unwrap();

            let _ = r1.discover().await;
            let _ = r2.discover().await;

            ep
        })
    };
}

fuzz_target!(|data: &[u8]| {
    async_std::task::block_on(ROUTER.recv(data))
});
