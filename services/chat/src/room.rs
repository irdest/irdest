//! Room logic module
//!
//! A room has a version to encode its capability set, and the room
//! state MUST be configured according to this version

use crate::types::Room;
use irdest_sdk::{messages::MsgId, Identity, IrdestSdk};
use std::{collections::BTreeSet, sync::Arc};

/// Internal room state encoding
///
/// The database internal room-state does not know how many messaes
/// are unread.  It does however have access to the backing store for
/// the room, so can check messages since `last_msg` to generate this
/// number.
// In the future this will be handled by alexandria (0.3 onwards)
pub(crate) struct RoomState {
    version: u8,
    name: String,
    participants: BTreeSet<Identity>,
    last_msg: MsgId,
}

impl RoomState {
    /// Create a new room state from a set of participants
    ///
    /// During the creation of this state the participants will be
    /// de-duplicated.
    pub(crate) fn v1(p: Vec<Identity>, name: String, last_msg: MsgId) -> Self {
        Self {
            version: 1,
            participants: p.into_iter().collect(),
            name,
            last_msg,
        }
    }

    /// Take this room state, fetch latest data from the backend, and
    /// compile an up-to-date metadata struct for a client
    pub(crate) fn make_room_meta(&self, ird: &Arc<IrdestSdk>) -> Room {
        todo!()
    }
}

/// A utility trait for functions implemented on a set of rooms
pub(crate) trait RoomSet {
    fn duplicate(&self, p: Vec<Identity>) -> bool;
}
