use sled::{open, Db as Handle};

use crate::daemon::state::data_path;

/// A database newtype, abstracting internal Db operations
pub(crate) struct Db(Handle);
// TODO: it is meant to only be instantiated once at the start of the daemon.
// a Db handle is completely safe to clone and access from multiple threads, but
// must only be opened once.

impl Db {
    pub(crate) fn new() -> Self {
        // Invariant:
        // there should only be one ratman router per device, so there's no need
        // for a unique db per-process. $datadir/db suffices as a path.
        // This invariant is broken in some tests, so we resort to this hack for
        // now, until the tests stop spawning many routers in the same filesystem.
        static mut HANDLE: Option<Handle> = None;
        unsafe {
            if let None = HANDLE {
                let path = data_path().join("db");
                let handle = open(path).unwrap();
                HANDLE = Some(handle)
            }
        }
        Db(unsafe { HANDLE.clone().unwrap() })
    }
}

#[test]
fn multiple_database_inits() {
    Db::new();
    Db::new();
}
