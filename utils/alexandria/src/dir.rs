//! Directory helper to create and manage Alexandria instances

use crate::error::Result;
use std::{fs, path::PathBuf};

/// Metadata for where things are stored
#[derive(Clone, Debug)]
pub(crate) struct Dirs {
    /// The root path, contains metadata
    root: PathBuf,
}

impl Dirs {
    pub(crate) fn new<P: Into<PathBuf>>(root: P) -> Self {
        Self { root: root.into() }
    }

    pub(crate) fn valid(&self) -> bool {
        self.root.exists()
    }

    pub(crate) fn scaffold(self) -> Result<Self> {
        let _ = fs::create_dir_all(&self.root);
        let _ = fs::create_dir(self.records());
        let _ = fs::create_dir(self.meta());
        let _ = fs::create_dir(self.cache());
        Ok(self)
    }

    /// Return the root of the library
    pub(crate) fn root(&self) -> PathBuf {
        self.root.clone()
    }

    /// Return the records directory in the library
    pub(crate) fn records(&self) -> PathBuf {
        self.root.join("records")
    }

    /// Return the meta directory in the library
    #[allow(unused)]
    pub(crate) fn meta(&self) -> PathBuf {
        self.root.join("meta")
    }

    /// Grab all files from a directory based on a filter
    pub(crate) fn filter_meta(&self, reg: &str) -> Vec<PathBuf> {
        vec![]
    }
    
    /// Return the cache directory in the library
    #[allow(unused)]
    pub(crate) fn cache(&self) -> PathBuf {
        self.root.join("cache")
    }
}

#[test]
fn scaffold_lib() -> Result<()> {
    use std::path::Path;
    use tempfile::tempdir;

    let root = tempdir().unwrap();
    let mut offset = root.path().to_path_buf();
    offset.push("library");

    let d = Dirs::new(offset.clone());
    d.scaffold()?;

    assert!(Path::new(dbg!(&offset.join("records"))).exists());
    assert!(Path::new(dbg!(&offset.join("meta"))).exists());
    assert!(Path::new(dbg!(&offset.join("cache"))).exists());
    Ok(())
}
