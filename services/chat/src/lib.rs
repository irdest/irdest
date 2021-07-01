//! A simple group chat running on an irdest network

mod room;
mod storage;
mod sub;

pub mod error;
pub mod types;

use crate::{
    error::Result,
    storage::ServiceStore,
    sub::Subscription,
    types::{Attachment, Room, RoomId},
};
use irdest_sdk::{messages::MsgId, users::UserAuth, Identity, IrdestSdk};
use std::sync::Arc;
use types::ChatMessage;

/// A simple, decentralised chat that runs on an irdest network
pub struct ChatService {
    store: ServiceStore,
}

impl ChatService {
    /// Create a chat service via an existing irdest SDK connection
    pub fn bind(inner: Arc<IrdestSdk>) -> Arc<Self> {
        Arc::new(Self {
            store: ServiceStore::new(&inner),
        })
    }

    /// Create a new room with a set of peers
    ///
    /// If no name is given, one will be inferred from the set of
    /// participants.
    ///
    /// Returns a copy of the new metadata set for the room.  If a
    /// room with that exact set of participants exists already an
    /// `Error::DuplicateRoom(RoomId)` will be returned instead.
    pub async fn create_room<I>(
        self: &Arc<Self>,
        auth: UserAuth,
        peers: I,
        name: Option<String>,
    ) -> Result<Room>
    where
        I: Into<Vec<Identity>>,
    {
        let v = peers.into();

        let name = match name {
            Some(n) => n,
            None => self.store.generate_name(&v).await,
        };

        self.store.create_room(auth, v, name).await
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
