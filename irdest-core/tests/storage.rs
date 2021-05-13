//! irdest-core storage API tests

#![allow(unused)]

mod harness;
use harness::{sec10, sec5};

use irdest_core::error::Result;

#[async_std::test]
async fn service_create() -> Result<()> {
    let net = harness::init().await;

    // Create a user
    let auth = net.a().users().create("abcdefg").await?;

    // Setup a new service
    net.a().services().register("test", |_| {}).await?;

    // Delete the service again
    net.a().services().unregister("test").await?;

    Ok(())
}

#[async_std::test]
async fn service_store_data() -> Result<()> {
    let net = harness::init().await;

    // Create a user
    let auth = net.a().users().create("abcdefg").await?;

    // Setup a new service
    net.a().services().register("test", |_| {}).await?;

    net.a()
        .services()
        .insert(auth.clone(), "test", "namespace#data", vec![1, 2, 3, 4])
        .await?;

    assert_eq!(
        net.a()
            .services()
            .query(auth, "test", "namespace#data")
            .await?,
        vec![1, 2, 3, 4]
    );

    Ok(())
}

#[async_std::test]
async fn service_update_data() -> Result<()> {
    let net = harness::init().await;

    // Create a user
    let auth = net.a().users().create("abcdefg").await?;

    // Setup a new service
    net.a().services().register("test", |_| {}).await?;

    net.a()
        .services()
        .insert(auth.clone(), "test", "namespace#data", vec![1, 2, 3, 4])
        .await?;

    // Then override it
    net.a()
        .services()
        .insert(auth.clone(), "test", "namespace#data", vec![4, 3, 2, 1])
        .await?;

    assert_eq!(
        net.a()
            .services()
            .query(auth, "test", "namespace#data")
            .await?,
        vec![4, 3, 2, 1]
    );

    Ok(())
}
