use ratman_client::{Identity, RatmanIpc, Receive_Type};

#[async_std::main]
async fn main() {
    let ipc = RatmanIpc::default()
        .await
        .expect("Failed to connect to Ratman daemon!");

    println!("Listening on address: {}", ipc.address());
    while let Some((tt, mut msg)) = ipc.next().await {
        // Ignore flood messages
        if tt == Receive_Type::FLOOD {
            continue;
        }

        // Get the message sender identity and reply
        let sender = Identity::from_bytes(msg.get_sender());
        ipc.send_to(sender, msg.take_payload()).await.unwrap();
    }
}
