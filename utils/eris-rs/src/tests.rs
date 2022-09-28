use crate::{BlockKey, BlockReference, BlockSize, RKPair, ReadCapability};
use serde::{Deserialize, Serialize};

use std::collections::HashMap;
use std::convert::TryInto;
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

impl TryInto<ReadCapability> for ReadCapabilityTest {
    type Error = crate::Error;

    fn try_into(self) -> Result<ReadCapability, Self::Error> {
        ReadCapability {
            root_reference: BlockReference::try_from(&self.root_reference)?,
            root_key: BlockKey::try_from(&self.root_key)?,
            level: self.level,
            block_size: self.block_size,
        };

        // let base32_alphabet = base32::Alphabet::RFC4648 { padding: false };
        // match base32::decode(base32_alphabet, &self.root_key) {
        //     Some(root_key) => match base32::decode(base32_alphabet, &self.root_reference) {
        //         Some(root_reference) => {

        //             let mut reference: [u8; 32] = Default::default();
        //             let mut key: [u8; 32] = Default::default();
        //             key.copy_from_slice(&root_key);
        //             reference.copy_from_slice(&root_reference);

        //             let root =

        //             let root = ReferenceKeyPair {
        //                 reference: reference,
        //                 key: key,
        //             };
        //             let block_size = match self.block_size {
        //                 1024 => BlockSize::_1K,
        //                 32768 => BlockSize::_32K,
        //                 _ => return Err(()),
        //             };
        //             Ok(ReadCapability {
        //                 root_reference,
        //                 root_key: root,
        //                 level: self.level,
        //                 block_size: block_size,
        //             })
        //         }
        //         None => Err(()),
        //     },
        //     None => Err(()),
        // }

        todo!()
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
    pub blocks: HashMap<String, String>,
}

#[derive(Debug)]
pub struct TestVector {
    pub file_name: String,
    pub data: TestVectorContent,
}

fn load_and_run(path: &Path) {
    let mut buf = String::new();
    let mut f = File::open(path).unwrap();
    f.read_to_string(&mut buf).unwrap();
}

#[test]
fn run_vectors() {
    std::fs::read_dir("./res/eris-test-vectors")
        .unwrap()
        .filter_map(|res| match res {
            Ok(entry) if entry.file_name().to_str().unwrap().ends_with(".json") => Some(entry),
            _ => None,
        })
        .for_each(|res| println!("{:?}", res));
}
