use async_eris::{Block, BlockReference, BlockStorage};
use async_trait::async_trait;
use deadpool_sqlite::{Config, Pool, Runtime};
use libratman::Result;
use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

pub mod addrs;
pub mod block;
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
            .create_pool(Runtime::Tokio1)
            .expect("failed to start db connection pool");
        Ok(Self { inner })
    }
}

// #[derive(Clone)]
// pub struct JournalStorage {
//     inner: Arc<sled::Db>,
// }

// impl JournalStorage {
//     /// Create a new JournalStorage handle and ensure the "blocks"
//     /// tree exists
//     pub fn new(inner: Arc<sled::Db>) -> Self {
//         inner.open_tree("blocks").unwrap();
//         Self { inner }
//     }
// }

// pub fn open_sled_tree(path: &Path) -> Arc<sled::Db> {
//     Arc::new(sled::open(path).unwrap())
// }

// #[async_trait]
// impl<const L: usize> BlockStorage<L> for JournalStorage {
//     async fn store(&mut self, block: &Block<L>) -> std::io::Result<()> {
//         let mut blocks = self.inner.open_tree("blocks")?;
//         blocks.insert(block.reference().as_slice(), block.as_slice());
//         Ok(())
//     }

//     async fn fetch(&self, reference: &BlockReference) -> std::io::Result<Option<Block<L>>> {
//         let blocks = self.inner.open_tree("blocks")?;
//         Ok(blocks
//             .get(reference.as_slice())
//             .map(|o| o.map(|ivec| Block::<L>::copy_from_vec(ivec.to_vec())))?)
//     }
// }

#[test]
fn setup_block_store() {
    todo!()
}
