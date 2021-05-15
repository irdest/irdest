//! A utility module to generate paths, formats, and i/o structure
//!
//! In the future this module may be versioned to allow for data
//! layout migrations.

use crate::{dir::Dirs, utils::Path, Session};
use std::path::PathBuf;

/// Create an fs path from a db path
pub(crate) fn path(dirs: &Dirs, user: &Session, p: &Path) -> PathBuf {
    dirs.records()
        .join(&format!("{}_{}.bin", user.to_slug(), p))
}
