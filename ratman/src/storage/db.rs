use sled::{open, Db as Handle};

use crate::daemon::state::data_path;

/// A database newtype, abstracting internal Db operations
pub(crate) struct Db(Handle);
// it is meant to only be instantiated once at the start of the daemon (TODO).

impl Db {
    pub(crate) fn new() -> Db {
        // there should only be one ratman daemon per device, so there's no need
        // for a unique db per-process. $datadir/db suffices as a path.
        let path = data_path().join("db");
        let handle = open(path).unwrap();
        Db(handle)
    }
}
