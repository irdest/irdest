use async_std::channel::{bounded, Receiver, Sender};
use ratman_netmod::Frame;

/// A simple I/O wrapper around channels
pub(crate) struct Io {
    pub out: Sender<Frame>,
    pub inc: Receiver<Frame>,
}

impl Io {
    pub(crate) fn make_pair() -> (Io, Io) {
        let (a_to_b, b_from_a) = bounded(1);
        let (b_to_a, a_from_b) = bounded(1);
        let a = Io {
            out: a_to_b,
            inc: a_from_b,
        };
        let b = Io {
            out: b_to_a,
            inc: b_from_a,
        };
        return (a, b);
    }
}
