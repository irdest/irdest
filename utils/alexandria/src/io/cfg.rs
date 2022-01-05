use crate::{dir::Dirs, error::Result};
use serde::{Deserialize, Serialize};
use std::{
    fs::{File, OpenOptions as Open},
    io::{Read, Write},
};

/// The current version of the library
pub const VERSION: u32 = 0;
pub const DEFAULT_CHUNK_SIZE: u64 = 4096;

/// A module with older version numbers to match against
pub mod legacy {
    /// Prototype/ rapid development phase
    ///
    /// It is not recommended to load _any_ database that was written
    /// in this version, due to no backwards compatible library
    /// structures.
    pub const ALPHA: u32 = 0;
}

/// Database configuration
#[derive(Serialize, Deserialize)]
pub struct Config {
    pub version: u32,
    pub chunk_size: u64,
}

impl Config {
    pub(crate) fn new() -> Self {
        Self {
            version: VERSION,
            chunk_size: DEFAULT_CHUNK_SIZE,
        }
    }

    pub(crate) fn load(d: &Dirs) -> Result<Self> {
        let path = d.root().join("db.config");

        let mut buf = String::new();
        let mut f = File::open(path)?;
        f.read_to_string(&mut buf)?;

        Ok(toml::from_str(buf.as_str())?)
    }

    pub(crate) fn write(&self, d: &Dirs) -> Result<()> {
        let path = d.root().join("db.config");

        let mut f = Open::new()
            .truncate(true)
            .write(true)
            .create(true)
            .open(path)?;
        f.write_all(toml::to_string(&self)?.as_bytes())?;
        Ok(())
    }
}
