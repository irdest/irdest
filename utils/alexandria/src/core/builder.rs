use crate::{
    dir::Dirs,
    io::{versions, Config, Sync},
    meta::{tags::TagCache, users::UserTable},
    query::SubHub,
    store::Store,
    Library,
};
use async_std::sync::{Arc, RwLock};
use std::{
    path::{Path, PathBuf},
    result::Result as StdResult,
};

/// A utility to configure and initialise an alexandria database
///
/// To load an existing database from disk, look at
/// [`Library::load()`][load]!
///
/// [load]: struct.Library.html#load
///
/// ```
/// # use alexandria::{Builder, Library, error::Result};
/// # use tempfile::tempdir;
/// # fn test() -> Result<()> {
/// let dir = tempdir().unwrap();
/// let lib = Builder::new()
///               .offset(dir.path())
///               .root_sec("car horse battery staple")
///               .build();
/// # drop(lib);
/// # Ok(()) }
/// ```
#[derive(Default)]
pub struct Builder {
    /// The main offset path
    offset: Option<PathBuf>,
}

impl Builder {
    pub fn new() -> Self {
        Self::default()
    }

    /// Inspect a path to load an existing alexandria library
    ///
    /// If no library exists at the path yet (or the path doesn't
    /// exist), the `Err(_)` variant is a new builder with an
    /// initialised `offset` that can then be used to create a new
    /// database.
    pub fn inspect_path<'tmp, P, S>(offset: P, _: S) -> StdResult<Arc<Library>, Self>
    where
        P: Into<&'tmp Path>,
        S: Into<String>,
    {
        let p = offset.into();
        let root = Dirs::new(p);

        // If the path doesn't exist it can't be a database
        if !root.valid() {
            return Err(Self::new().offset(p));
        }

        // TODO: load database with provided root secret

        // Load the database config and return a builder if it fails
        let cfg = Config::load(&root).map_err(|_| Self::new())?;
        if cfg.version == versions::ALPHA {
            warn!("Loading an ALPHA library from disk; data loss may occur!");
        }

        Ok(Arc::new(Library {
            root: root.clone(),
            cfg: RwLock::new(cfg),
            users: RwLock::new(UserTable::new()),
            tag_cache: RwLock::new(TagCache::new()),
            store: RwLock::new(Store::new()),
            subs: SubHub::new(),
            sync: Sync::new(root),
        }))
    }

    /// Specify a normal path offset
    ///
    /// This will act as the root metadata store.  On multi-user
    /// devices it needs to be a directory that's accessibly from the
    /// daemon that owns the alexandria scope.
    pub fn offset<'tmp, P: Into<&'tmp Path>>(self, offset: P) -> Self {
        let offset = Some(PathBuf::from(offset.into()));
        Self { offset, ..self }
    }

    /// Some secret that will be used for the root namespace
    ///
    /// When loading a library from disk in a future session, this
    /// secret will have to be provided to [`Library::load()`][load]
    ///
    /// [load]: struct.Library.html#load
    pub fn root_sec<S: Into<String>>(self, _: S) -> Self {
        self
    }

    /// Consume the builder and create a Library
    ///
    /// Note that this will not create a persistent storage on-disk.
    /// To achieve this, you must call `sync()` on the created
    /// library.
    pub fn build(self) -> Arc<Library> {
        let root = Dirs::new(
            self.offset
                .expect("Builder without `offset` cannot be built"),
        );

        Arc::new(Library {
            root: root.clone(),
            cfg: RwLock::new(Config::init()),
            users: RwLock::new(UserTable::new()),
            tag_cache: RwLock::new(TagCache::new()),
            store: RwLock::new(Store::new()),
            subs: SubHub::new(),
            sync: Sync::new(root),
        })
    }
}
