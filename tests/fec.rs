//! Test that supplying empty packets does forward error correction.

extern crate opus_rs;
use opus_rs::*;

#[test]
fn blah() {
    let mut opus_rs = Decoder::new(48000, Channels::Mono).unwrap();

    let mut output = vec![0i16; 5760];
    let size = opus_rs.decode(&[], &mut output[..], true).unwrap();
    assert_eq!(size, 5760);
}
