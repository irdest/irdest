use sled::{Db as Handle, open};

use crate::daemon::state::data_path;

/// A database newtype, abstracting internal Db operations
pub(crate) struct Db(Handle);
// it is meant to only be instantiated once at the start of the daemon (TODO).

impl Db {
    pub(crate) fn new() -> Db {
        let path = data_path().join("db");
        let handle = open(path).unwrap();
        Db(handle)
    }
}
