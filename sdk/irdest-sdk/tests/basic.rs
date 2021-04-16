mod harness;
use harness::RpcState;

use irdest_sdk::IrdestSdk;
use irpc_sdk::error::RpcResult;

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
