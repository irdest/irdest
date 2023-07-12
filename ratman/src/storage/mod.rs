#![allow(unused)]

use deadpool_sqlite::{Config, Pool, Runtime};
use libratman::types::Result;
use std::path::PathBuf;

pub mod addrs;
pub mod client;

/// Represent a database connection
#[derive(Clone)]
pub struct MetaDbHandle {
    inner: Pool,
}

impl MetaDbHandle {
    /// Create a new handle for a given sqlite database path
    pub fn init(path: PathBuf) -> Result<Self> {
        let inner = Config::new(path)
            .create_pool(Runtime::AsyncStd1)
            .expect("failed to start db connection pool");
        Ok(Self { inner })
    }
}
