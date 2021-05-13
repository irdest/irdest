//! Handling service store (metadata) interaction with Alexandria

use super::{Conv, FromRecord};
use crate::services::StoreKey;
use alexandria::{
    record::{kv::Value, RecordRef},
    utils::Diff,
};
use std::collections::BTreeMap;

const NAME: &'static str = "name";
const MAP: &'static str = "map";

pub(crate) struct MetadataMap(BTreeMap<StoreKey, Vec<u8>>);

impl From<Vec<(String, Vec<u8>)>> for MetadataMap {
    fn from(v: Vec<(String, Vec<u8>)>) -> Self {
        Self(v.into_iter().fold(BTreeMap::new(), |mut map, (k, v)| {
            let key = StoreKey::from(k);
            map.insert(key, v);
            map
        }))
    }
}

impl FromRecord<MetadataMap> for MetadataMap {
    fn from_rec(rec: RecordRef) -> Self {
        Self(
            Conv::bin_map(rec.kv().get(MAP).unwrap())
                .into_iter()
                .map(|(k, v)| (StoreKey::from(k), v))
                .collect(),
        )
    }
}

pub(crate) trait MetadataExt {
    fn init_diff(&self) -> Vec<Diff>;
    fn gen_diffset(&self, prev: &Self) -> Vec<Diff>;
}

impl MetadataExt for MetadataMap {
    fn init_diff(&self) -> Vec<Diff> {
        vec![Diff::map().insert(
            MAP,
            self.0
                .iter()
                .map(|(k, v)| (k.clone().to_string(), Value::Vec(v.clone())))
                .collect::<BTreeMap<String, Value>>(),
        )]
    }

    /// Generate a diffset based on the previous version of the map
    fn gen_diffset(&self, prev: &Self) -> Vec<Diff> {
        let mut vec = vec![];

        self.0.iter().for_each(|(key, val)| {
            match prev.0.get(key) {
                // If the key was present in the previous map, generate an update if the value has changed
                Some(prev) if prev != val => {
                    vec.push(
                        Diff::map().nested(MAP, Diff::map().update(key.to_string(), val.clone())),
                    );
                }
                // And if it wasn't we insert it normally
                None => {
                    vec.push(
                        Diff::map().nested(MAP, Diff::map().insert(key.to_string(), val.clone())),
                    );
                }
                _ => {}
            }
        });

        // Do another run in reverse and delete all keys that are now missing
        prev.0.iter().for_each(|(key, _)| {
            if self.0.get(key).is_none() {
                vec.push(Diff::map().nested(MAP, Diff::map().delete(key.to_string())));
            }
        });

        vec
    }
}

// #[test]
// fn metadata_diff_empty() {
//     let m = MetadataMap::new("test");
//     let diffs = m.init_diff();
//     assert_eq!(diffs.len(), 2);
// }

// #[test]
// fn metadata_diff_simple() {
//     let m = MetadataMap::from(vec![("key", vec![1, 2, 3, 4]), ("acab", vec![1, 3, 1, 2])]);
//     let diffs = m.init_diff();
//     assert_eq!(diffs.len(), 2);
// }

// #[test]
// fn metadata_diff_delete() {
//     let old = MetadataMap::from(vec![("key", vec![1, 2, 3, 4]), ("acab", vec![1, 3, 1, 2])]);

//     let new = MetadataMap::from(vec![("acab", vec![1, 3, 1, 2])]);

//     let diffs = new.gen_diffset(&old);
//     assert_eq!(diffs.len(), 1);
// }

// #[test]
// fn metadata_diff_delete_update() {
//     let old = MetadataMap::from(vec![("key", vec![1, 2, 3, 4]), ("acab", vec![1, 3, 1, 2])]);

//     let new = MetadataMap::from(vec![("acab", vec![13, 12])]);

//     let diffs = new.gen_diffset(&old);
//     assert_eq!(diffs.len(), 2);
// }
