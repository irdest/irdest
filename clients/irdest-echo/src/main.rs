// SPDX-FileCopyrightText: 2022 Katharina Fey <kookie@spacekookie.de>
//
// SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

use libratman::client::{RatmanIpc, Receive_Type};

#[async_std::main]
async fn main() {
    let ipc = RatmanIpc::default()
        .await
        .expect("Failed to connect to Ratman daemon!");

    println!("Listening on address: {}", ipc.address());
    while let Some((tt, msg)) = ipc.next().await {
        // Ignore flood messages
        if tt == Receive_Type::FLOOD {
            continue;
        }

        // Get the message sender identity and reply
        let sender = msg.get_sender();
        ipc.send_to(sender, msg.get_payload()).await.unwrap();
    }
}
