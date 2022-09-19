

// TODO: can we set no_std only when the "std" feature is not set?
// #[cfg(not(feature = "std"))] // <-- this doesn't seem to work?
// #![no_std]

/// Encode a frame for basic wire formats
pub fn encode_frame(f: &Frame, max_size: usize) -> Vec<u8> {
    let mut buf = Vec::with_capacity(max_size);



    buf
}
