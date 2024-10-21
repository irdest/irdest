use std::{thread, time::Duration};

use crate::config::ConfigTree;
use libratman::{
    api::{RatmanIpc, RatmanIpcExtV1},
    tokio::{runtime::Runtime, time::sleep},
};

#[test]
fn create_address() {
    let _jh = thread::spawn(|| {
        let tmp = tempdir::TempDir::new("create-addr").unwrap();
        crate::start_with_configuration(ConfigTree::default_in_memory(), tmp.path().to_path_buf());
    });

    let rt = Runtime::new().unwrap();
    rt.block_on(async move {
        println!("sleep 10 seconds");
        sleep(Duration::from_secs(10)).await;
        println!("woke up");

        let ipc = RatmanIpc::start("127.0.0.1:5852".parse().unwrap())
            .await
            .unwrap();
        let (addr, auth) = ipc.addr_create(None).await.unwrap();

        println!("Successfully registered new address {addr} and auth {auth:?}");
    });

    std::process::exit(0);
}
