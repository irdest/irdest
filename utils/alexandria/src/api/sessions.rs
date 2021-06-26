use crate::{error::Result, utils::Id};
use rand::prelude::*;
use std::fmt::{self, Debug};

/// An active session identifier
///
/// Provide this identifier to all future operations on the database.
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Session {
    pub id: Id,
    token: [u8; 32],
}

impl Session {
    /// Create a new Id with a random token
    pub(crate) fn new(id: Id) -> Self {
        let mut rng = rand::thread_rng();
        Self {
            id,
            token: rng.gen(),
        }
    }

    /// Turn this session into a slug for paths
    pub(crate) fn to_slug(&self) -> String {
        self.id.to_string()
    }
}

impl Debug for Session {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Session: {}", self.id)
    }
}

/// Database session API structure
///
/// In order to perform operations on the database your application
/// needs access to an active (or known inactive) session.  A session
/// is a user namespace in the database.  Data can not be shared
/// between sessions!
///
/// ## Hidden User Session
///
/// In order to preserve the privacy of data holders in Alexandria it
/// is possible to mark a user session as "hidden".  This means that
/// no reference to it will be made in the public sessions list.  Data
/// will obviously still be present in the database for this session,
/// but the ID is encrypted with the user passphares, which makes it
/// impossible to infer the session ID from the session ID on disk.
///
/// This does however have consequences for your application, as the
/// user ID can not be known before a user has logged in.  This means
/// that a user can not log-in by selecting their user ID from a list
/// but must instead generate the session ID themselves via a PBKDF2
/// mechanism.
///
/// A hidden user session can also no longer accept dead-drop
/// encryption data.  **Any data inserted or updated in the database
/// for an inactive hidden user session will be discarded!**
///
/// See [`generate_secret`](Sessions::generate_secret) and
/// [`create_hidden`](Sessions::create_hidden) for more details!
pub struct Sessions<'db> {
    pub(crate) inner: &'db (),
}

impl<'db> Sessions<'db> {
    /// Derive a secret identity from a user factoid via PBKDF2
    pub fn generate_secret<S: Into<String>>(&self, phrase: S) -> Id {
        todo!()
    }

    /// Create a new (well known) user session in this database
    pub fn create<S: Into<String>>(&self, id: Id, pw: S) -> Result<Session> {
        todo!()
    }

    /// Create a new secret user session in this database
    ///
    /// Please read the section "Hidden User Session" on the
    /// [`Sessions`](Sessions) type and be aware of the caveats of
    /// such a user session.
    pub fn create_hidden<S: Into<String>>(&self, id: Id, pw: S) -> Result<Session> {
        todo!()
    }

    /// Destroy an existing user session and all data associated to it
    pub fn destroy(&self, session: Session) -> Result<()> {
        todo!()
    }

    /// Open an existing user session
    ///
    /// This may either be a well-known or secret session.  The
    /// provided ID must have been created via one of the two
    /// mechanisms first!
    pub fn open<S: Into<String>>(&self, id: Id, pw: S) -> Result<Session> {
        todo!()
    }

    /// Close a currently open user session
    pub fn close(&self, session: Session) -> Result<()> {
        todo!()
    }
}
