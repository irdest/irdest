use crate::types::{Address, Recipient, TimePair};

/// Message stream metadata section
pub struct Letterhead {
    /// Who is sending this message?
    pub from: Address,
    /// Who are we sending this message to?
    pub to: Recipient,
    /// Both the sending and receiving time
    pub time: TimePair,
}
