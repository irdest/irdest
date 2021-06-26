use crate::types::RoomId;
use irdest_sdk::error::Error as SdkError;

pub type Result<T> = std::result::Result<T, ChatError>;

#[derive(Debug, thiserror::Error)]
pub enum ChatError {
    #[error("irdest-sdk yielded an error: {0}")]
    Sdk(SdkError),
    #[error("duplicate: a room ('{0}')with the exact set of participants already exists!")]
    Duplicate(RoomId),
}

impl From<SdkError> for ChatError {
    fn from(e: SdkError) -> Self {
        Self::Sdk(e)
    }
}
