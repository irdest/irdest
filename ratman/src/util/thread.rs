///
pub struct NetmodThreadPair;

/// A thread meant to run a Netmod reiceve state system.
///
/// Can be configured in one of three ways:
///
/// - Complete system :: A single receiver thread will run all
/// receiver endpoints.  If a single endpoint blocks, it will block
/// all endpoints.
///
/// - Tandem system :: Up to two endpoints will run per receiver
/// thread, providing a balanced alternative
///
/// - Single system :: Each endpoint will run on its own receiver
/// thread.  This way every netmod can run completely independently.
pub struct ReceiverThread {}

impl ReceiverThread {
    pub fn new_complete() -> Self {
        std::thread::spawn(|| {});
        todo!()
    }

    pub fn new_tandem() -> Self {
        todo!()
    }

    pub fn new_single() -> Self {
        todo!()
    }
}

/// A thread meant to run a Netmod sender state system
pub struct SenderThread {}

impl SenderThread {
    pub fn new_set(num: u8) -> Self {
        todo!()
    }
}
