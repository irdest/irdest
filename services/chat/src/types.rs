//! Message and service types
//!
//! This module contains all types used by irdest-chat to communicate
//! between users actively, and via hidden command messages.

use chrono::{DateTime, Utc};
use irdest_sdk::{messages::MsgId, Identity};

/// A text-message sent in a [`ChatRoom`](self::ChatRoom)
pub struct ChatMessage {
    /// Unique identity
    pub id: MsgId,
    /// Message sender identity
    pub sender: Identity,
    /// The timestamp at which the message was received (in utc)
    pub timestamp: DateTime<Utc>,
    /// Text payload
    pub context: String,
    /// Additional user payload
    pub attachment: Option<Attachment>,
    /// Additional service payloads
    pub(crate) state: RoomState,
}

/// A rich-type attachment to a chat message
///
/// This type can be used to send either images, voice messages, or
/// videos between users.
pub enum Attachment {
    /// A PNG formatted image buffer
    Image(Vec<u8>),
    /// An MP3 formatted voice buffer
    Audio(Vec<u8>),
}

pub type RoomId = Identity;

pub(crate) enum RoomState {
    /// A simple chat message needs to know its room id
    Id(RoomId),
    /// Indicate to peers that a room must be created
    Create(Room),
    /// Confirm any previous command between peers
    Confirm(RoomId, MsgId),
    /// Apply changes to the metadata of a room
    Diff(RoomDiff),
}

/// Metadata for a room
pub struct Room {
    /// A computer-friendly identifier
    pub id: RoomId,
    /// A human-friendly identifier
    pub name: String,
    /// List of room participants.  Currently it is not possible to
    /// add or remove participants
    pub participants: Vec<Identity>,
    /// An unread message counter.  Utility to make implementing
    /// front-ends easier
    pub unread: usize,
    /// Last known message
    pub last_msg: MsgId,
}

pub(crate) enum RoomDiff {}
