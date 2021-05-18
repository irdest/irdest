//! Various metadata tables in the database

use crate::crypto::asym::KeyPair;

pub(crate) mod tags;
pub(crate) mod users;

/// Central metadata storage
///
/// An alexandria library can hold associated user metadata, as well
/// as a root secret key shared for a whole library.
pub(crate) struct Metadata {
    users: users::UserTable,
    tags: tags::TagCache,
    shared: KeyPair,
}
