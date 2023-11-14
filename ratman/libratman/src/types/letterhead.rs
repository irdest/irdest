use std::ffi::CString;

use crate::types::{Address, Id, Recipient, TimePair};

/// Message stream metadata section
///
/// This type is constructed by the sending client and controls the
/// sending behaviour of the router.
pub struct Letterhead {
    /// Who is sending this message?
    pub from: Address,
    /// Who are we sending this message to?
    pub to: Recipient,
    /// Both the sending and receiving time
    pub time: TimePair,
    /// This message stream ID
    pub stream_id: Id,
    /// Payload length
    pub payload_length: usize,
    /// Optional set of key-value attributes
    ///
    /// These can radically change the sending behaviour of your
    /// provided payload.  Please consult the manual for details.
    pub auxiliary_data: Vec<(CString, CString)>,
}
