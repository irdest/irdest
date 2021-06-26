pub mod error;
pub mod types;
mod sub;

use crate::{
    error::Result,
    sub::Subscription,
    types::{Attachment, Room, RoomId},
};
use irdest_sdk::{users::UserAuth, messages::MsgId, Identity, IrdestSdk};
use std::sync::Arc;
use types::ChatMessage;

/// A simple, decentralised chat that runs on an irdest network
pub struct ChatService {
    // ¯\_ (ツ)_/¯
}

impl ChatService {
    /// Create a chat service via an existing irdest SDK connection
    pub fn bind(_: Arc<IrdestSdk>) -> Arc<Self> {
        todo!()
    }

    /// Create a new room with a set of peers
    ///
    /// Returns a copy of the new metadata set for the room.  If a
    /// room with that exact set of participants exists already an
    /// `Error::DuplicateRoom(RoomId)` will be returned instead.
    pub fn create_room<I>(self: &Arc<Self>, _auth: UserAuth, _peers: I) -> Result<Room>
    where
        I: Into<Vec<Identity>>,
    {
        todo!()
    }

    /// Get the current metadata state for a room with an ID
    pub fn get_room(self: &Arc<Self>, _auth: UserAuth, _room: RoomId) -> Result<Room> {
        todo!()
    }

    /// Send a new message to an existing room
    pub fn send_message(
        self: &Arc<Self>,
        _auth: UserAuth,
        _room: RoomId,
        _text: String,
        _attachment: Option<Attachment>,
    ) -> Result<()> {
        todo!()
    }

    /// Query for new (or old) messages in a room
    // FIXME: introduce query iterator abstraction to avoid having to
    // load all messages in the backend to send them to the front-end
    // -- this will not scale for long
    pub fn query(
        self: &Arc<Self>,
        _auth: UserAuth,
        _room: RoomId,
        _last: Option<MsgId>,
    ) -> Result<Vec<ChatMessage>> {
        todo!()
    }

    /// Subscribe to all future room messages
    pub fn subscribe(self: &Arc<Self>, _auth: UserAuth, _room: RoomId) -> Result<Subscription> {
        todo!()
    }
}
