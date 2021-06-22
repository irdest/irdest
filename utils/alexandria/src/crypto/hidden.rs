use crate::utils::Id;

/// A utility type to ensure that user IDs are not leaked
pub(crate) struct Hid(Id);

impl Hid {
    /// Generate a Hid simply by hashing an existing Id
    pub(crate) fn hash(id: Id) -> Self {
        todo!()
    }

    /// Generate a Hid from the ID and password
    pub(crate) fn aes(id: Id, pw: &str) -> Self {
        todo!()
    }
}
