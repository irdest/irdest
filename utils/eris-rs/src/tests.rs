use crate::{
    Block, BlockKey, BlockReference, BlockSize, BlockStorage, MemoryStorage, ReadCapability,
};
use serde::{Deserialize, Serialize};

use std::collections::BTreeMap;
use std::{fs::File, io::Read, path::Path};

#[derive(Serialize, Deserialize, Debug)]
pub struct ReadCapabilityTest {
    #[serde(rename = "block-size")]
    pub block_size: usize,
    pub level: u8,
    #[serde(rename = "root-reference")]
    pub root_reference: String,
    #[serde(rename = "root-key")]
    pub root_key: String,
}

impl TryFrom<&ReadCapabilityTest> for ReadCapability {
    type Error = crate::Error;

    fn try_from(cap_test: &ReadCapabilityTest) -> Result<Self, Self::Error> {
        Ok(ReadCapability {
            root_reference: BlockReference::try_from(&cap_test.root_reference)?,
            root_key: BlockKey::try_from(&cap_test.root_key)?,
            level: cap_test.level,
            block_size: cap_test.block_size,
        })
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TestVectorContent {
    pub id: u32,
    #[serde(rename = "spec-version")]
    pub spec_version: String,
    pub name: String,
    pub description: String,
    pub content: String,
    #[serde(rename = "convergence-secret")]
    pub convergence_secret: String,
    #[serde(rename = "block-size")]
    pub block_size: usize,
    #[serde(rename = "read-capability")]
    pub read_capability: ReadCapabilityTest,
    pub urn: String,
    pub blocks: BTreeMap<String, String>,
}

impl TestVectorContent {
    pub async fn blocks_to_blocks(&self) -> Result<MemoryStorage, crate::Error> {
        let mut store = MemoryStorage::new();

        for (block_id, block) in self.blocks.iter() {
            let block_ref = BlockReference::try_from(block_id)?;
            match self.read_capability.block_size {
                1024 => {
                    let block = Block::<1024>::try_from(block)?;
                    assert_eq!(block_ref, block.reference());
                    store.store(&block).await.unwrap();
                }
                32768 => {
                    let block = Block::<32768>::try_from(block)?;
                    assert_eq!(block_ref, block.reference());
                    store.store(&block).await.unwrap();
                }
                _ => panic!("Unsupported block size!"),
            };
        }

        Ok(store)
    }
}

#[derive(Debug)]
pub struct TestHarness {
    blocks: Box<MemoryStorage>,
    read_cap: ReadCapability,
    _test: Box<TestVectorContent>,
}

impl TestHarness {
    async fn load(path: &Path) -> Option<Self> {
        let mut buf = String::new();
        let mut f = File::open(path).unwrap();
        f.read_to_string(&mut buf).unwrap();

        let vector_content: Box<TestVectorContent> = match serde_json::from_str(buf.as_str()) {
            Ok(v) => Box::new(v),
            Err(e) => {
                eprintln!("Error: {:?}.", e);
                return None;
            }
        };

        let read_cap = ReadCapability::try_from(&vector_content.read_capability).ok()?;
        let blocks = vector_content
            .blocks_to_blocks()
            .await
            .ok()
            .map(Into::into)?;

        Some(Self {
            blocks,
            read_cap,
            _test: vector_content,
        })
    }
}

#[async_std::test]
async fn run_vectors() {
    for res in std::fs::read_dir("./res/eris-test-vectors")
        .unwrap()
        .filter_map(|res| match res {
            Ok(entry) if entry.file_name().to_str().unwrap().ends_with(".json") => Some(entry),
            Ok(entry) => {
                eprintln!("Ignoring: {:?}", entry.file_name().to_str().unwrap());
                None
            }
            _ => None, // Not important?
        })
    {
        let path = res.path();
        let harness = match TestHarness::load(path.as_path()).await {
            Some(h) => Box::new(h),
            None => {
                eprintln!(
                    "An error occured loading {:?} and will thus be skipped!",
                    path
                );
                continue;
            }
        };

        println!(
            "Loading file: {:?} has resulted in {} blocks",
            path,
            harness.blocks.len()
        );

        // let mut buf = vec![];
        // crate::decode(&mut buf, &harness.read_cap, &harness.blocks)
        //     .await
        //     .unwrap();

        // if let Some(input) = std::str::from_utf8(&buf).ok() {
        //     println!("Input data: {}", input);
        // }
    }
}
