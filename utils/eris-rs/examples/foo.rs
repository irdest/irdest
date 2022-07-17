// SPDX-FileCopyrightText: 2019-2021 Yureka Lilian <yuka@yuka.dev>
//
// SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

use eris::{BlockSize, MemoryStorage};

#[async_std::main]
async fn main() {
    let examples = vec![
        b"Hello world!".as_slice(),
        [0; 4096].as_slice(),
    ];

    for content in examples {
        let key = [0; 32];
        let mut blocks = MemoryStorage::new();
        let read_capability = eris::encode(&mut &*content, &key, BlockSize::_1K, &mut blocks).await.unwrap();
        println!("{}", read_capability.urn());
        println!("{:?}", read_capability);
        for (reference, block) in &blocks {
            println!("{}: {}", base32::encode(base32::Alphabet::RFC4648 { padding: false }, &**reference), base32::encode(base32::Alphabet::RFC4648 { padding: false }, &block));
        }

        let mut decoded = vec![];
        eris::decode(&mut decoded, &read_capability, &blocks).await.unwrap();
        println!("{}", String::from_utf8_lossy(&decoded));
    }
}
