//! User profile database wrappers (models)

use super::{Conv, FromRecord};
use crate::{
    security::Keypair,
    types::diff::ItemDiff,
    users::{UserProfile, UserUpdate},
};
use alexandria::{
    record::{kv::Value, RecordRef},
    utils::Diff,
};
use bincode;
use std::collections::BTreeMap;

const KPAIR: &'static str = "keypair";
const UID: &'static str = "id";
const HANDLE: &'static str = "handle";
const D_NAME: &'static str = "display_name";
const BIO: &'static str = "bio";
const SERV: &'static str = "services";
const AVI: &'static str = "avatar";

pub(crate) struct KeyWrap(pub(crate) Keypair);

impl KeyWrap {
    /// Generate the initial diff of metadata
    pub(crate) fn make_diff(&self) -> Diff {
        Diff::map().insert(KPAIR, bincode::serialize(&self.0).unwrap())
    }
}

impl FromRecord<KeyWrap> for KeyWrap {
    fn from_rec(rec: RecordRef) -> Self {
        KeyWrap(
            bincode::deserialize(
                rec.kv()
                    .get(KPAIR)
                    .map(|v| match v {
                        Value::Vec(bytes) => bytes,
                        _ => unreachable!(),
                    })
                    .unwrap(),
            )
            .unwrap(),
        )
    }
}

/// Get a UserProfile from a record
impl FromRecord<UserProfile> for UserProfile {
    fn from_rec(rec: RecordRef) -> Self {
        let kv = rec.kv();

        Self {
            id: Conv::id(kv.get(UID).unwrap()),
            handle: kv.get(HANDLE).map(|v| Conv::string(v)),
            display_name: kv.get(D_NAME).map(|v| Conv::string(v)),
            bio: kv
                .get(BIO)
                .map(|v| Conv::map(v))
                .unwrap_or_else(|| Default::default()),
            services: kv
                .get(SERV)
                .map(|v| Conv::str_set(v))
                .unwrap_or_else(|| Default::default()),
            avatar: kv.get(AVI).map(|v| Conv::binvec(v)),
        }
    }
}

// FIXME: Combine all the diff tools
pub(crate) trait UserProfileExt {
    fn init_diff(&self) -> Vec<Diff>;
    fn gen_diff(&self, update: UserUpdate) -> Vec<Diff>;
}

impl UserProfileExt for UserProfile {
    /// Generate the first insert diff based on an empty record
    fn init_diff(&self) -> Vec<Diff> {
        let mut v = vec![Diff::map().insert(UID, self.id.as_bytes().to_vec())];

        if let Some(ref d_name) = self.handle {
            v.push(Diff::map().insert(HANDLE, d_name.clone()));
        }
        if let Some(ref r_name) = self.display_name {
            v.push(Diff::map().insert(D_NAME, r_name.clone()));
        }

        v.push(
            Diff::map().insert(
                BIO,
                self.bio
                    .iter()
                    .map(|(k, v)| (k.clone(), v.clone().into()))
                    .collect::<BTreeMap<String, Value>>(),
            ),
        );
        v.push(
            Diff::map().insert(
                SERV,
                self.services
                    .iter()
                    .map(|k| k.clone().into())
                    .collect::<Vec<Value>>(),
            ),
        );

        if let Some(ref avi) = self.avatar {
            v.push(Diff::map().insert(AVI, avi.clone()));
        }

        v
    }

    /// Diff based on how a `UserUpdate` applies to a `UserProfile`
    fn gen_diff(&self, update: UserUpdate) -> Vec<Diff> {
        let UserUpdate {
            handle,
            display_name,
            add_to_bio,
            rm_from_bio,
            services,
            avi_data,
        } = update;

        let mut vec = vec![];

        match (handle, &self.handle) {
            (ItemDiff::Set(name), Some(_)) => vec.push(Diff::map().update(HANDLE, name)),
            (ItemDiff::Set(name), None) => vec.push(Diff::map().insert(HANDLE, name)),
            (ItemDiff::Unset, Some(_)) => vec.push(Diff::map().delete(HANDLE)),
            _ => {}
        }

        match (display_name, &self.display_name) {
            (ItemDiff::Set(name), Some(_)) => vec.push(Diff::map().update(D_NAME, name)),
            (ItemDiff::Set(name), None) => vec.push(Diff::map().insert(D_NAME, name)),
            (ItemDiff::Unset, Some(_)) => vec.push(Diff::map().delete(D_NAME)),
            _ => {}
        }
        // TODO: handle add_to_bio/ rm_from_bio/ services updates

        match (avi_data, &self.avatar) {
            (ItemDiff::Set(avi), Some(_)) => vec.push(Diff::map().update(AVI, avi)),
            (ItemDiff::Set(avi), None) => vec.push(Diff::map().insert(AVI, avi)),
            (ItemDiff::Unset, Some(_)) => vec.push(Diff::map().delete(AVI)),
            _ => {}
        }
        vecc
    }
}

#[test]
fn persist_user_profile() {
    use crate::Identity;
    use alexandria::{
        utils::{Path, TagSet},
        Builder, GLOBAL,
    };

    let dir = tempfile::tempdir().unwrap();
    let lib = Builder::new().offset(dir.path()).build().unwrap();

    let profile = UserProfile {
        id: Identity::random(),
        handle: Some("spacekookie".into()),
        display_name: Some("Katharina Fey".into()),
        bio: {
            let mut tree = BTreeMap::new();
            tree.insert("location".into(), "The internet".into());
            tree.insert("languages".into(), "en, de, fr, eo, ru".into());
            tree
        },
        services: vec![
            "net.qaul.chat",
            "net.qaul.feed",
            "net.qaul.voice",
            "space.kookie.chess",
        ]
        .into_iter()
        .map(|s| s.into())
        .collect(),
        avatar: None,
    };

    let path = Path::from(format!("/users:{}", profile.id));

    let diffs = profile.init_diff();
    async_std::task::block_on(async {
        lib.batch(GLOBAL, path.clone(), TagSet::empty(), diffs)
            .await
    })
    .unwrap();
}
