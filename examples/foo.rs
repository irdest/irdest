fn main() {
    let content = [0; 4096];
    let key = [0; 32];
    let encoded = eris::encode(&content, key, eris::BlockSize::_1K);
    println!("root-reference: {}", base32::encode(base32::Alphabet::RFC4648 { padding: false }, &encoded.1));
    println!("root-key: {}", base32::encode(base32::Alphabet::RFC4648 { padding: false }, &encoded.2));
    println!("blocks:");
    for block in encoded.0 {
        println!("{}", base32::encode(base32::Alphabet::RFC4648 { padding: false }, &block));
    }
    println!("");
}
