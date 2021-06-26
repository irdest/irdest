use crate::{dir::Dirs, error::Result, meta::KeyStore};
use async_std::sync::Arc;
use std::path::PathBuf;

/// In-memory representation of an alexandria database
pub struct Library {
    pub(crate) keys: Arc<KeyStore>,
    pub(crate) dirs: Dirs,
}

impl Library {
    pub fn create<P: Into<PathBuf>>(path: P) -> Result<Self> {
        let path = path.into();
        let dirs = Dirs::new(path).scaffold()?;

        Ok(Self {
            keys: todo!(),
            dirs,
        })
    }
}
