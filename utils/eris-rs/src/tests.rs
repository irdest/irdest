// SPDX-FileCopyrightText: 2022 Katharina Fey <kookie@spacekookie.de>
//
// SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

use crate::{
    Block, BlockKey, BlockReference, BlockSize, BlockStorage, MemoryStorage, ReadCapability,
};
use serde::{Deserialize, Serialize};

use std::collections::{BTreeMap, HashMap};
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
        let locked = MemoryStorage::new(HashMap::new());

        for (block_id, block) in self.blocks.iter() {
            let block_ref = BlockReference::try_from(block_id)?;
            match self.read_capability.block_size {
                1024 => {
                    let block = Block::<1024>::try_from(block)?;
                    assert_eq!(block_ref, block.reference());
                    locked.store(&block).await.unwrap();
                }
                32768 => {
                    let block = Block::<32768>::try_from(block)?;
                    assert_eq!(block_ref, block.reference());
                    locked.store(&block).await.unwrap();
                }
                _ => panic!("Unsupported block size!"),
            };
        }

        Ok(locked)
    }
}

#[derive(Debug)]
pub struct TestHarness {
    blocks: MemoryStorage,
    read_cap: ReadCapability,
    _test: TestVectorContent,
}

impl TestHarness {
    async fn load(path: &Path) -> Option<Box<Self>> {
        let mut buf = String::new();
        let mut f = File::open(path).unwrap();
        f.read_to_string(&mut buf).unwrap();

        let vector_content: TestVectorContent = match serde_json::from_str(buf.as_str()) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("Error: {:?}.", e);
                return None;
            }
        };

        let read_cap = ReadCapability::try_from(&vector_content.read_capability).ok()?;
        let blocks = vector_content.blocks_to_blocks().await.ok()?;

        Some(Box::new(Self {
            blocks,
            read_cap,
            _test: vector_content,
        }))
    }
}

async fn verify_input_content(harness: &TestHarness) -> bool {
    let input_content = crate::vardecode_base32(&harness._test.content).unwrap();

    let secret: [u8; 32] = crate::decode_base32(harness._test.convergence_secret.as_str()).unwrap();
    let block_size = match harness._test.block_size {
        1024 => BlockSize::_1K,
        32768 => BlockSize::_32K,
        _ => unreachable!(),
    };
    let mut new_store = MemoryStorage::new(HashMap::new());

    let new_read_cap = crate::encode(
        &mut input_content.as_slice(),
        &secret,
        block_size,
        &mut new_store,
    )
    .await
    .unwrap();

    *harness.blocks.read().unwrap() == *new_store.read().unwrap()
        && harness.read_cap.root_reference == new_read_cap.root_reference
}

async fn run_test_for_vector(path: &Path, tx: tokio::sync::mpsc::Sender<()>) {
    let harness = match TestHarness::load(path).await {
        Some(h) => Box::new(h),
        _ => {
            eprintln!(
                "An error occured loading {:?} and the test will now fail!",
                path
            );
            std::process::exit(2);
        }
    };

    println!(
        "Loading file: {:?} has resulted in {} blocks",
        path,
        harness.blocks.read().unwrap().len()
    );

    // Decode input content and verify that this results in the same
    // set of blocks as in the test harness file!
    assert!(verify_input_content(&harness).await);

    // If we reach this point this vector was successfully parsed,
    // decoded, and re-encoded.
    tx.send(()).await.unwrap();
}

#[tokio::test]
async fn run_vectors() {
    let mut test_vectors = std::fs::read_dir("./res/eris-test-vectors")
        .unwrap()
        .filter_map(|res| match res {
            Ok(entry) if entry.file_name().to_str().unwrap().ends_with(".json") => Some(entry),
            _ => None, // Not important?
        })
        .collect::<Vec<_>>();
    test_vectors
        .as_mut_slice()
        .sort_by_key(|entry| entry.file_name().to_str().unwrap().to_owned());

    for res in test_vectors {
        // We do this little dance here because otherwise it's very
        // easy to get stack overflow errors in this test scenario!
        let path = res.path();
        let (tx, mut rx) = tokio::sync::mpsc::channel(1);
        tokio::spawn(async move {
            run_test_for_vector(path.as_path(), tx).await;
        });

        rx.recv().await.unwrap();
    }
}

#[tokio::test]
async fn big_message() {
    use crate as eris;
    use eris::{BlockSize, MemoryStorage};
    use rand::{rngs::OsRng, RngCore};

    let mut content = vec![0; 1024 * 64];
    OsRng {}.fill_bytes(&mut content);

    println!("Taking {} bytes of content...", content.len());

    let mut blocks = MemoryStorage::new(HashMap::new());
    // Randomly chosen key :)
    let key = [
        93, 72, 136, 16, 3, 194, 107, 102, 20, 11, 42, 105, 193, 208, 47, 23, 135, 76, 154, 63, 41,
        84, 85, 108, 86, 0, 90, 58, 6, 112, 22, 4,
    ];

    let read_capability = eris::encode(&mut &*content, &key, BlockSize::_1K, &mut blocks)
        .await
        .unwrap();

    println!("{}", read_capability.urn());
    println!("{:?}", read_capability);
    // for (reference, block) in &blocks {
    //     println!(
    //         "{}: {}",
    //         base32::encode(base32::Alphabet::RFC4648 { padding: false }, &**reference),
    //         base32::encode(base32::Alphabet::RFC4648 { padding: false }, &block)
    //     );
    // }

    let mut decoded = vec![];
    eris::decode(&mut decoded, &read_capability, &blocks)
        .await
        .unwrap();

    assert_eq!(decoded, content);
    println!("Input == Output");
}

////////////////////////////////////////////////////////////////////////////////

#[tokio::test]
async fn serde_enc_dec() {
    use crate::{Block, BlockSize, BlockStorage, MemoryStorage};
    use rand::{rngs::OsRng, RngCore};

    let mut content = vec![0; 1024 * 32];
    OsRng {}.fill_bytes(&mut content);

    println!("Taking {} bytes of content...", content.len());

    let mut blocks = MemoryStorage::new(HashMap::new());
    // Randomly chosen key :)
    let key = [
        84, 85, 108, 86, 0, 90, 58, 6, 112, 22, 4, 93, 72, 136, 16, 3, 194, 107, 102, 20, 11, 42,
        105, 193, 208, 47, 23, 135, 76, 154, 63, 41,
    ];

    let read_capability = crate::encode(&mut &*content, &key, BlockSize::_1K, &mut blocks)
        .await
        .unwrap();

    println!("{}", read_capability.urn());
    println!("{:?}", read_capability);

    /////// THE ACTUAL TEST ///////

    let bincode_serde = blocks
        .read()
        .unwrap()
        .iter()
        .map(|(_ref, block)| serde_json::to_string(&block).unwrap())
        .collect::<Vec<_>>();

    drop(blocks);

    println!(
        "Encoded {} blocks as json with serde \\o/",
        bincode_serde.len()
    );

    let blocks2 = MemoryStorage::new(HashMap::new());

    for json_block in bincode_serde.into_iter() {
        let block: Block<1024> = serde_json::from_str(&json_block).unwrap();
        blocks2.store(&block).await.unwrap();
    }

    let mut decoded = vec![];
    crate::decode(&mut decoded, &read_capability, &blocks2)
        .await
        .unwrap();

    assert_eq!(decoded, content);
    println!("Input(with_serde) == Output(with_serde)");
}
