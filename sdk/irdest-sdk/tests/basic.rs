mod harness;
use harness::RpcState;

use irdest_sdk::{
    messages::{IdType, Message, Mode},
    services::ServiceEvent,
    tags::TagSet,
    IrdestSdk,
};
use irpc_sdk::{error::RpcResult, Subscription};

/// A simple test that connects to an Irdest instance over RPC
#[async_std::test]
async fn user_create() -> RpcResult<()> {
    // Create a small test network with 2 RPC sockets
    let _state = RpcState::new(6000, 6500).await;

    // Register a service on one of them
    let serv = harness::make_service(6000).await?;

    // Initialise Irdest SDK
    let sdk = IrdestSdk::connect(&serv)?;

    // Create a user
    let auth = sdk
        .users()
        .create("dont write your passwords in unit tests duh")
        .await?;

    println!("User auth: {:?}", auth);
    let is_auth = dbg!(sdk.users().is_authenticated(auth.clone()).await);
    assert_eq!(is_auth, Ok(()));
    Ok(())
}

#[async_std::test]
async fn subscription() -> RpcResult<()> {
    let state = RpcState::new(6010, 6510).await;
    let serv = harness::make_service(6010).await?;
    let sdk = IrdestSdk::connect(&serv)?;
    let auth = sdk
        .users()
        .create("dont write your passwords in unit tests duh")
        .await?;

    // Create a user on node B manually
    let user_b = state
        .tp
        .b()
        .users()
        .create("bad passwords get your nudes leaked")
        .await
        .unwrap();

    // Create a subscription for all incoming messages
    let sub: Subscription<Message> = sdk
        .messages()
        .subscribe(auth.clone(), "test", TagSet::empty())
        .await?;

    // Spawn a task to send messages to our node
    async_std::task::spawn(async move {
        harness::zzz(1000).await;

        state
            .tp
            .b()
            .messages()
            .send(
                user_b,
                Mode::Std(auth.0),
                IdType::Unique,
                "test",
                TagSet::empty(),
                "Hello you!".as_bytes().to_vec(),
            )
            .await
            .unwrap();
    });

    let msg = sub.next().await.unwrap();

    assert_eq!(msg.payload, "Hello you!".as_bytes().to_vec());
    Ok(())
}

/// A simple test that connects to an Irdest instance over RPC
#[async_std::test]
async fn service_create() -> RpcResult<()> {
    // Create a small test network with 2 RPC sockets
    let _state = RpcState::new(6020, 6520).await;

    // Register a service on one of them
    let serv = harness::make_service(6020).await?;

    // Initialise Irdest SDK
    let sdk = IrdestSdk::connect(&serv)?;

    // Register a new service
    let s = sdk.services().register("test").await?;

    let auth = sdk.users().create("foo bar baz boink").await?;

    let event = match s.next().await {
        Ok(ServiceEvent::Open(e)) => e,
        _ => unreachable!(),
    };

    assert_eq!(auth, event);
    Ok(())
}
