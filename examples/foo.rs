use eris::{BlockSize, MemoryStorage};

#[async_std::main]
async fn main() {
    //let mut content = b"Hello, world!".as_slice();
    let content = [0; 4096];
    let key = [0; 32];
    let mut blocks = MemoryStorage::new();
    let read_capability = eris::encode(&mut content.as_slice(), &key, BlockSize::_1K, &mut blocks).await.unwrap();
    println!("{:?}", read_capability);
    println!("blocks:");
    for (reference, block) in &blocks {
        println!("{}: {}", base32::encode(base32::Alphabet::RFC4648 { padding: false }, &**reference), base32::encode(base32::Alphabet::RFC4648 { padding: false }, &block));
    }

    let mut decoded = vec![];
    eris::decode(&mut decoded, &read_capability, &blocks).await.unwrap();
    println!("{}", String::from_utf8_lossy(&decoded));
}
