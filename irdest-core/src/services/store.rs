use crate::{
    error::{Error, Result},
    services::StoreKey,
    store::{FromRecord, MetadataExt, MetadataMap},
    Identity,
};
use alexandria::{
    query::{Query, QueryResult},
    record::kv::Value,
    utils::{Diff, Path, Tag, TagSet},
    Library, Session,
};
use async_std::sync::Arc;
use std::{
    collections::BTreeMap,
    ops::{Deref, DerefMut},
};
use tracing::warn;

/// Generate the service metedata entry path
fn gen_path(serv: &String) -> Path {
    Path::from(format!("/service:{}", serv))
}

fn tag_service(serv: &String) -> Tag {
    Tag::new("irdest._int.service", serv.as_bytes().to_vec())
}

const TAG_METADATA: &'static str = "irdest._int.metadata";

/// Internal metadata store wrapper for Alexandria
#[derive(Clone)]
pub(crate) struct MetadataStore {
    inner: Arc<Library>,
}

impl MetadataStore {
    pub(crate) fn new(inner: Arc<Library>) -> Self {
        Self { inner }
    }

    /// Every (user/service) tuple has ONE metadata map.  This
    /// function makes sure it exists so that future operations don't
    /// have to worry about creating it.
    async fn ensure(&self, user: Identity, service: &String) {
        let s = Session::Id(user);
        let path = gen_path(service);

        if let Err(_) = self.inner.query(s, Query::path(path.clone())).await {
            debug!("Service storage not found; creating path `{}`", path);

            self.inner
                .insert(s, path, TagSet::empty(), Diff::map())
                .await
                .unwrap();
        }
    }

    pub(crate) async fn insert(
        &self,
        user: Identity,
        serv: String,
        key: StoreKey,
        data: Vec<u8>,
    ) -> Result<()> {
        self.ensure(user, &serv).await;
        let s = Session::Id(user);
        let path = gen_path(&serv);

        self.ensure(user, &serv).await;
        let s = Session::Id(user);
        let path = gen_path(&serv);

        // Try to update the key first.  If this fails the key does
        // not exist (presumably, seeing as we have already passed
        // authentication), so we insert it instead.
        if let Err(_) = self
            .inner
            .update(
                s,
                path.clone(),
                Diff::map().update(key.to_string(), data.clone()),
            )
            .await
        {
            debug!("Key `{}` does not exist in record; inserting", key);
            self.inner
                .insert(
                    s,
                    path,
                    // Most importantly this system does NOT use the TagSet types
                    // to search, and instead operates on a single record with a
                    // Key-Value store
                    TagSet::empty(),
                    Diff::map().insert(key.to_string(), data),
                )
                .await;
        }

        Ok(())
    }

    #[tracing::instrument(skip(self), level = "debug")]
    pub(crate) async fn delete(&self, user: Identity, serv: String, key: StoreKey) {
        self.ensure(user, &serv).await;
        let s = Session::Id(user);
        let path = gen_path(&serv);

        match self
            .inner
            .update(s, path, Diff::map().delete(key.to_string()))
            .await
        {
            Ok(()) => {}
            Err(_) => {
                warn!("Attempted to delete a non-existing key `{}`", key);
            }
        }
    }

    pub(crate) async fn query(
        &self,
        user: Identity,
        serv: String,
        key: StoreKey,
    ) -> Result<Vec<u8>> {
        self.ensure(user, &serv).await;
        let s = Session::Id(user);
        let path = gen_path(&serv);

        let rec = self
            .inner
            .query(s, Query::path(path))
            .await
            .map_err(|_| Error::NoData)?;

        match rec.as_single().kv().get(&key.to_string()) {
            Some(Value::Vec(vec)) => Ok(vec.clone()),
            _ => Err(Error::NoData),
        }
    }
}

#[cfg(test)]
async fn test_setup() -> (crate::IrdestRef, Identity) {
    let i = crate::Irdest::dummy();
    let auth = i.users().create("foooooo").await.unwrap();
    (i, auth.0)
}

#[async_std::test]
async fn insert() {
    let (ird, id) = test_setup().await;

    ird.services
        .store()
        .insert(
            id,
            "test".to_string(),
            "fav-numbers".to_string().into(),
            vec![13, 12],
        )
        .await
        .unwrap();

    let data = ird
        .services
        .store()
        .query(id, "test".to_string(), "fav-numbers".to_string().into())
        .await
        .unwrap();

    assert_eq!(data, vec![13, 12]);
}

#[async_std::test]
async fn update() {
    let (ird, id) = test_setup().await;

    // Insert one
    ird.services
        .store()
        .insert(
            id,
            "test".to_string(),
            "fav-numbers".to_string().into(),
            vec![13, 12],
        )
        .await
        .unwrap();

    let data = ird
        .services
        .store()
        .query(id, "test".to_string(), "fav-numbers".to_string().into())
        .await
        .unwrap();

    assert_eq!(data, vec![13, 12]);

    // Insert again, but with different data
    ird.services
        .store()
        .insert(
            id,
            "test".to_string(),
            "fav-numbers".to_string().into(),
            vec![1, 2, 3, 4],
        )
        .await
        .unwrap();

    let data = ird
        .services
        .store()
        .query(id, "test".to_string(), "fav-numbers".to_string().into())
        .await
        .unwrap();

    assert_eq!(data, vec![1, 2, 3, 4]);
}
