//! User profile database wrappers (models)

use super::Conv;
use crate::users::{UserProfile, UserUpdate};
use alexandria::{record::Record, utils::Diff};

const UID: &'static str = "id";
const D_NAME: &'static str = "display_name";
const R_NAME: &'static str = "real_name";
const BIO: &'static str = "bio";
const SERV: &'static str = "services";
const AVI: &'static str = "avatar";

/// Get a UserProfile from a record
impl From<&Record> for UserProfile {
    fn from(rec: &Record) -> Self {
        let kv = rec.kv();

        Self {
            id: Conv::id(kv.get(UID).unwrap()),
            display_name: kv.get(D_NAME).map(|v| Conv::string(v)),
            real_name: kv.get(R_NAME).map(|v| Conv::string(v)),
            bio: kv
                .get(BIO)
                .map(|v| Conv::map(v))
                .unwrap_or_else(|| Default::default()),
            services: kv
                .get(SERV)
                .map(|v| Conv::set(v))
                .unwrap_or_else(|| Default::default()),
            avatar: kv.get(AVI).map(|v| Conv::binvec(v)),
        }
    }
}

impl UserProfile {

    /// Diff based on how a `UserUpdate` applies to a `UserProfile`
    pub(crate) fn gen_diff(&self, update: UserUpdate) -> Diff {
        use UserUpdate::*;

        match update {
            // Update data if it was previously set
            DisplayName(Some(name)) if self.display_name.is_some() => {
                Diff::map().update(D_NAME, name)
            }
            RealName(Some(name)) if self.real_name.is_some() => Diff::map().update(R_NAME, name),
            SetBioLine(key, val) if self.bio.contains_key(&key) => {
                Diff::map().nested(D_NAME, Diff::map().update(key, val))
            }
            RemoveBioLine(key) if self.display_name.is_some() => {
                Diff::map().nested(D_NAME, Diff::map().delete(key))
            }
            AddService(service) if self.services.contains(&service) => unimplemented!(),
            RemoveService(service) if self.services.contains(&service) => unimplemented!(),

            // Insert if it wasn't
            DisplayName(Some(name)) => Diff::map().insert(D_NAME, name),
            RealName(Some(name)) => Diff::map().insert(R_NAME, name),
            SetBioLine(key, val) => Diff::map().nested(BIO, Diff::map().insert(key, val)),
            RemoveBioLine(key) => Diff::map().nested(BIO, Diff::map().delete(key)),
            AddService(_) => unimplemented!(),
            RemoveService(_) => unimplemented!(),

            // Delete if set to None
            DisplayName(None) => Diff::map().delete(D_NAME),
            RealName(None) => Diff::map().delete(R_NAME),

            // Avatars are a little special
            AvatarData(Some(data)) => Diff::map().delete(AVI).insert(AVI, data),
            AvatarData(None) => Diff::map().delete(BIO),
        }
    }
}