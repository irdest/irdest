use eris::{BlockReference, BlockSize};
use std::collections::HashMap;

#[async_std::main]
async fn main() {
    let content = b"Hello, world!";
    let key = [0; 32];
    let mut blocks = HashMap::<BlockReference, Vec<u8>>::new();
    let read_capability = eris::encode(content, &key, BlockSize::_1K, &mut blocks).await.unwrap();
    println!("{:?}", read_capability);
    println!("blocks:");
    for (reference, block) in &blocks {
        println!("{}: {}", base32::encode(base32::Alphabet::RFC4648 { padding: false }, &**reference), base32::encode(base32::Alphabet::RFC4648 { padding: false }, &block));
    }

    let decoded = eris::decode(&read_capability, &blocks).await.unwrap();
    println!("{}", String::from_utf8_lossy(&decoded));
}
