use crate::journal::page::SerdeFrameType;
use libratman::{frame::carrier::PeerDataV1, types::Id};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LinkData {
    link_id: Id,
    neighbour_data: SerdeFrameType<PeerDataV1>,
}
