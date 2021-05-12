mod harness;
use harness::RpcState;

use irdest_sdk::{
    messages::{IdType, Message, Mode},
    tags::TagSet,
    IrdestSdk,
};
use irpc_sdk::{error::RpcResult, Subscription};

/// A simple test that connects to an Irdest instance over RPC
#[async_std::test]
async fn user_create() -> RpcResult<()> {
    // Create a small test network with 2 RPC sockets
    let state = RpcState::new(6060, 7070).await;

    // Register a service on one of them
    let serv = harness::make_service(6060).await?;

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
    harness::parse_log_level();

    let state = RpcState::new(6060, 7070).await;
    let serv = harness::make_service(6060).await?;
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

    while let Ok(msg) = sub.next().await {
        println!("Received message: {} => {:?}", msg.id, msg.payload);
    }

    Ok(())
}
