// SPDX-FileCopyrightText: 2023 Katharina Fey <kookie@spacekookie.de>
//
// SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

use async_eris as eris;
use eris::{BlockSize, MemoryStorage};
use rand::{rngs::OsRng, RngCore};
use std::collections::HashMap;

#[tokio::main]
async fn main() {
    let mut content = vec![0; 1024 * 64]; // 64k
    OsRng {}.fill_bytes(&mut content);

    println!("Taking {} bytes of content...", content.len());

    let blocks = MemoryStorage::new(HashMap::new());
    // Randomly chosen key :)
    let key = [
        93, 72, 136, 16, 3, 194, 107, 102, 20, 11, 42, 105, 193, 208, 47, 23, 135, 76, 154, 63, 41,
        84, 85, 108, 86, 0, 90, 58, 6, 112, 22, 4,
    ];

    let read_capability = eris::encode(&mut &*content, &key, BlockSize::_1K, &blocks)
        .await
        .unwrap();

    println!("{}", read_capability.urn());
    println!("{:?}", read_capability);
    // for (reference, block) in &blocks {
    //     println!(
    //         "{}: {}",
    //         base32::encode(base32::Alphabet::RFC4648 { padding: false }, &**reference),
    //         base32::encode(base32::Alphabet::RFC4648 { padding: false }, &block)
    //     );
    // }

    let mut decoded = vec![];
    eris::decode(&mut decoded, &read_capability, &blocks)
        .await
        .unwrap();

    assert_eq!(decoded, content);
    println!("Input == Output");
}
