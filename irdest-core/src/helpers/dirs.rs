use crate::error::{Error, Result};
use directories::ProjectDirs;
use std::{path::PathBuf, sync::Arc};
use tempfile::TempDir;

/// A config, data, and cache directories helper
///
/// Irdest can run on a lot of platforms that have different
/// requirements for where and how data can be stored.  To make
/// initialisation as easy as possible (for both tests, and clients on
/// different platforms) this structure is meant to be used to
/// initialise the irdest core!
#[allow(unused)]
#[derive(Clone, Debug)]
pub struct Directories {
    pub(crate) data: PathBuf,
    pub(crate) config: PathBuf,
    pub(crate) cache: PathBuf,

    // Tempdir will remote itself if Directories is dropped
    temp: Option<Arc<TempDir>>,
}

impl Directories {
    /// Create a temporary directory tree for tests
    pub fn temp() -> Result<Self> {
        let temp = tempfile::tempdir()?;
        let temp_path = temp.path().to_path_buf();

        Ok(Self {
            data: temp_path.join("data"),
            config: temp_path.join("config"),
            cache: temp_path.join("cache"),
            temp: Some(Arc::new(temp)),
        })
    }

    /// Create directory metadata for the current platform
    ///
    /// Provide a client ID to separate data stored by different
    /// clients.  The 'organisation' of the client is always `irdest`.
    pub fn new(client_id: &str) -> Result<Self> {
        ProjectDirs::from("", "irdest", client_id)
            .ok_or(Error::IoFault)
            .map(|dirs| Self {
                data: dirs.data_dir().into(),
                config: dirs.config_dir().into(),
                cache: dirs.cache_dir().into(),
                temp: None,
            })
    }
}
