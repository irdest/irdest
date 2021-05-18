//! Alexandria specific error handling
//!
//! Generally, not all errors can be expressed as one, and many of the
//! errors that happen internally are filtered and repacked into a set
//! of common errors that users of the library will have to deal with.
//! They are most commonly related to user mistakes, scheduling
//! problems, etc.
//!
//! However there are some errors that the database itself can't
//! handle, and so it has to bubble up via the `IternalError` variant
//! on `Error`.  These can be bugs in Alexandria itself, or some
//! runtime constraint like having run out of memory or disk space.

use std::fmt::{self, Display, Formatter};

/// Common alexandria error fascade
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("failed to add a user that already exits")]
    UserAlreadyExists,

    #[error("operation failed because user `{}` doesn't exist", id)]
    NoSuchUser { id: String },

    #[error("failed to initialise library at offset `{}`", offset)]
    InitFailed { offset: String },

    #[error("suffered I/O error: `{}`", msg)]
    IoFailed { msg: String },

    #[error("failed to load library configuration: `{}`", msg)]
    InvalidCfg { msg: String },

    #[error("failed to sync data: `{}`", msg)]
    SyncFailed { msg: String },

    #[error("failed to perform action because user `{}` is locked", id)]
    UserNotOpen { id: String },

    #[error("bad unlock token (password?) for id `{}`", id)]
    UnlockFailed { id: String },

    #[error("tried to operate on locked encrypted state: {}", msg)]
    LockedState { msg: String },

    #[error("tried to unlock user Id `{}` twice", id)]
    AlreadyUnlocked { id: String },

    #[error("no such path: `{}`", path)]
    NoSuchPath { path: String },

    #[error("path exists already: {}", path)]
    PathExists { path: String },

    #[error("tried to apply Diff of incompatible type")]
    BadDiffType,

    #[error("a Diff failed to apply: \n{}", msgs)]
    BadDiff { msgs: DiffErrors },

    #[error(
        "can't merge two iterators with different underlying queries: a: '{}', b: '{}'",
        q1,
        q2
    )]
    IncompatibleQuery { q1: String, q2: String },

    #[doc(hidden)]
    #[error("An alexandria internal error occured: `{}`", msg)]
    InternalError { msg: String },
}

/// A convenience alias to contain a common alexandria error
pub type Result<T> = std::result::Result<T, Error>;

/// Span info errors that can occur while applying a diff to a record
#[derive(Debug)]
pub struct DiffErrors(Vec<(usize, String)>);

impl DiffErrors {
    pub(crate) fn add(mut self, new: Self) -> Self {
        let mut ctr = self.0.len();
        new.0.into_iter().for_each(|(_, e)| {
            self.0.push((ctr, e));
            ctr += 1;
        });

        self
    }

    /// Helper function to apply text replacements to nested messages
    pub(crate) fn replace_text<'n, 'o>(self, old: &'o str, new: &'n str) -> Self {
        Self(
            self.0
                .into_iter()
                .map(|(i, s)| (i, s.as_str().replace(old, new).into()))
                .collect(),
        )
    }
}

impl Display for DiffErrors {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.0.iter().fold(Ok(()), |res, (num, msg)| {
            res.and_then(|_| write!(f, r#"{}: "{}""#, num, msg))
        })
    }
}

impl From<Vec<(usize, String)>> for DiffErrors {
    fn from(vec: Vec<(usize, String)>) -> Self {
        Self(vec)
    }
}

impl From<(usize, String)> for DiffErrors {
    fn from(tup: (usize, String)) -> Self {
        Self(vec![tup])
    }
}

impl From<DiffErrors> for Error {
    fn from(msgs: DiffErrors) -> Self {
        Error::BadDiff { msgs }
    }
}

impl From<bincode::Error> for Error {
    fn from(be: bincode::Error) -> Self {
        use bincode::ErrorKind::*;

        // FIXME: this isn't great but like... whatevs
        let msg = match *be {
            Io(e) => format!("I/O error: '{}'", e),
            SizeLimit => "Payload too large!".into(),
            SequenceMustHaveLength => "Internal sequencing error".into(),
            e => format!("Encoding error: {}", e),
        };

        Self::SyncFailed { msg }
    }
}

impl From<async_std::io::Error> for Error {
    fn from(e: async_std::io::Error) -> Self {
        Self::IoFailed {
            msg: format!("{}", e),
        }
    }
}

impl From<toml::de::Error> for Error {
    fn from(e: toml::de::Error) -> Self {
        Self::InvalidCfg {
            msg: format!("{}", e),
        }
    }
}

impl From<toml::ser::Error> for Error {
    fn from(e: toml::ser::Error) -> Self {
        Self::SyncFailed {
            msg: format!("{}", e),
        }
    }
}
