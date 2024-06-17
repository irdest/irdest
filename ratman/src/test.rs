use std::time::Duration;

use crate::config::ConfigTree;
use libratman::{
    api::{RatmanIpc, RatmanIpcExtV1},
    tokio::{runtime::Runtime, time::sleep},
};

#[test]
fn create_address() {
    let rrt = Runtime::new().unwrap();
    let jh = rrt.spawn_blocking(|| {
        crate::start_with_configuration(ConfigTree::default_in_memory());
    });

    let rt = Runtime::new().unwrap();
    rt.block_on(async move {
        sleep(Duration::from_millis(200)).await;

        let ipc = RatmanIpc::start("127.0.0.1:5852".parse().unwrap())
            .await
            .unwrap();
        let (addr, auth) = ipc.addr_create(None, None).await.unwrap();

        println!("Successfully registered new address {addr} and auth {auth:?}");
    });

    std::process::exit(0);
}
