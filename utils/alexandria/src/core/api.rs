use crate::{
    core::{Builder, Session, SessionsApi},
    delta::{DeltaBuilder, DeltaType},
    dir::Dirs,
    error::{Error, Result},
    io::{Config, Sync},
    meta::{tags::TagCache, users::UserTable},
    query::{Query, QueryIterator, QueryResult, SetQuery, SubHub, Subscription},
    store::Store,
    utils::{Diff, Id, Path, TagSet},
};
use async_std::sync::{Arc, RwLock};
use std::{fmt::Debug, path};

/// In-memory representation of an alexandria database
///
/// Refer to [`Builder`][builder] to configure and initialise an alexandria
/// instance.
///
/// [builder]: struct.Builder.html
pub struct Library {
    /// The main management path
    pub(crate) root: Dirs,
    /// On-disk configuration
    pub(crate) cfg: RwLock<Config>,
    /// Table with encrypted user metadata
    pub(crate) users: RwLock<UserTable>,
    /// Cache of tag/path mappings
    pub(crate) tag_cache: RwLock<TagCache>,
    /// The main data store
    pub(crate) store: RwLock<Store>,
    /// The state handler for subscriptions
    pub(crate) subs: Arc<SubHub>,
    /// Synchronisation engine
    pub(crate) sync: Arc<Sync>,
}

impl Library {
    /// Internally called setup function
    pub(crate) async fn init(&self) -> Result<()> {
        warn!("Creating an ALPHA library on disk; data loss may occur!");

        // Write the library configuration
        self.cfg.read().await.write(&self.root)?;

        Ok(())
    }

    /// Load and re-initialise a previous database session from disk
    ///
    /// While this function technically does the same thing, you
    /// should switch to `builder::inspect_path` to load or create
    /// databases on the fly instead.
    #[deprecated]
    pub fn load<'tmp, P, S>(offset: P, root_sec: S) -> Result<Arc<Self>>
    where
        P: Into<&'tmp path::Path>,
        S: Into<String>,
    {
        let offset = offset.into();
        let p = offset.to_str().unwrap_or("<Unknown directory>");

        Builder::inspect_path(offset, root_sec).map_err(|_| Error::IoFailed {
            msg: format!("Invalid library path: {}", p),
        })
    }

    /// Load the database sessions API scope
    pub fn sessions<'lib>(&'lib self) -> SessionsApi<'lib> {
        SessionsApi { inner: self }
    }

    /// Start the synchronisation engine for this database
    ///
    /// If you call `sync()` on `Builder` calling this function is no
    /// longer required.
    ///
    /// By default an alexandria `Library` exists purely in-memory.
    /// This is useful for testing purposes.  To allow alexandria to
    /// synchronise data to disk dynamically, call this function to
    /// start an async task to supervise this behaviour.
    ///
    /// This function returns `Err(_)` if the offset provided during
    /// initialisation can't be written to.
    pub async fn sync(self: &Arc<Self>) -> Result<()> {
        self.init().await?;

        let sync = Arc::clone(&self.sync);
        let this = Arc::clone(&self);

        sync.start(this).await.map_err(|e| {
            error!("Synchronisation engine emitted an error: {}", e);
            e
        })
    }

    /// Force unfinished transactions to sync to disk
    pub fn ensure(&self) {
        self.sync.block_on_clear();
    }

    /// Create a new record and apply a batch of diffs to it immediately
    ///
    /// Semantically this function works the same as `insert`, meaning
    /// that the provided path must be unique for the given session,
    /// and the tags provided can not be updated in the future.
    #[tracing::instrument(skip(self, data, tags), level = "info")]
    pub async fn batch<T, D>(&self, id: Session, path: Path, tags: T, data: Vec<D>) -> Result<Id>
    where
        T: Into<TagSet>,
        D: Into<Diff>,
    {
        if let Session::Id(id) = id {
            self.users.read().await.is_open(id)?;
            info!("Passed open-auth for id `{}`", id.to_string());
        }

        let mut db = DeltaBuilder::new(id, DeltaType::Insert);
        let tags = tags.into();

        let mut store = self.store.write().await;
        let rec_id = store.batch(
            &mut db,
            id,
            &path,
            tags.clone(),
            data.into_iter().map(|d| d.into()).collect(),
        )?;
        drop(store);

        let mut tc = self.tag_cache.write().await;
        tags.iter().fold(Ok(()), |res, t| {
            res.and_then(|_| tc.insert(id, path.clone(), t.clone()))
        })?;
        drop(tc);

        let delta = db.make();
        self.sync.queue(&delta);
        self.subs.queue(delta).await;

        info!("Batch insert succeeded");
        Ok(rec_id)
    }

    /// Insert a new record into the library and return it's ID
    ///
    /// The provided path must be unique for the provided user
    /// session.  This function will return an error if a record
    /// already exists at the given path.
    ///
    /// Note that it is currently not possible to update the tag set
    /// of a record after it has been created, so make sure to include
    /// any relevant search tags in this function call.  If you want
    /// to change them in the future, you will have to delete and
    /// re-create the record.
    #[tracing::instrument(skip(self, data, tags), level = "info")]
    pub async fn insert<T, D>(&self, id: Session, path: Path, tags: T, data: D) -> Result<Id>
    where
        T: Into<TagSet>,
        D: Into<Diff>,
    {
        if let Session::Id(id) = id {
            self.users.read().await.is_open(id)?;
            info!("Passed open-auth for id `{}`", id.to_string());
        }

        let mut db = DeltaBuilder::new(id, DeltaType::Insert);
        let tags = tags.into();

        let mut store = self.store.write().await;
        let rec_id = store.insert(&mut db, id, &path, tags.clone(), data.into())?;
        drop(store);

        let mut tc = self.tag_cache.write().await;
        tags.iter().fold(Ok(()), |res, t| {
            res.and_then(|_| tc.insert(id, path.clone(), t.clone()))
        })?;
        drop(tc);

        let delta = db.make();
        self.sync.queue(&delta);
        self.subs.queue(delta).await;

        info!("Record insert succeeded");
        Ok(rec_id)
    }

    /// Delete a path from the database
    ///
    /// Note that a record MAY not be deleted immediately, if it has a
    /// GC guard hold on it from one (or multiple) query iterators.
    /// In that case a path may still be available via a direct query,
    /// even though it has already been deleted.
    #[tracing::instrument(skip(self), level = "info")]
    pub async fn delete(&self, id: Session, path: Path) -> Result<()> {
        if let Session::Id(id) = id {
            self.users.read().await.is_open(id)?;
            info!("Passed open-auth for id `{}`", id.to_string());
        }

        let mut db = DeltaBuilder::new(id, DeltaType::Delete);

        let mut store = self.store.write().await;
        store.destroy(&mut db, id, &path)?;
        drop(store);

        let mut tc = self.tag_cache.write().await;
        tc.delete_path(id, path)?;
        drop(tc);

        let delta = db.make();
        self.sync.queue(&delta);
        self.subs.queue(delta).await;

        info!("Record delete succeeded");
        Ok(())
    }

    /// Update an existing record with a diff
    ///
    /// This function requires the given record path to already exist.
    #[tracing::instrument(skip(self, diff), level = "info")]
    pub async fn update<D>(&self, id: Session, path: Path, diff: D) -> Result<()>
    where
        D: Into<Diff>,
    {
        if let Session::Id(id) = id {
            self.users.read().await.is_open(id)?;
            info!("Passed open-auth for id `{}`", id.to_string());
        }

        let mut db = DeltaBuilder::new(id, DeltaType::Update);

        let mut store = self.store.write().await;
        store.update(&mut db, id, &path, diff.into())?;
        drop(store);

        let delta = db.make();
        self.sync.queue(&delta);
        self.subs.queue(delta).await;

        info!("Record update succeeded");
        Ok(())
    }

    /// Update, or insert a record into the database
    ///
    /// This function will update a record in the database, or create
    /// it if the provided path doesn't already exist.  If the record
    /// doesn't already exist, and no `tags` parameter is provided, it
    /// will initialise to `TagSet::empty()`.
    ///
    /// Furthermore, once a record is created its tag set can not
    /// currently be changed.  Thus in update-mode this function will
    /// ignore additional parameter passed into `tags`.
    pub async fn upsert<T, D>(&self, id: Session, path: Path, tags: T, diff: D) -> Result<Id>
    where
        T: Into<Option<TagSet>>,
        D: Into<Diff>,
    {
        if let Session::Id(id) = id {
            self.users.read().await.is_open(id)?;
            info!("Passed open-auth for id `{}`", id.to_string());
        }

        // Read-lock the store to check for the existance of a path
        let store = self.store.read().await;
        let exists = store.get_path(id, &path);
        drop(store);

        match exists {
            Ok(rec) => self.update(id, path, diff).await.map(|_| rec.header.id),
            Err(_) => {
                self.insert(id, path, tags.into().unwrap_or(TagSet::empty()), diff)
                    .await
            }
        }
    }

    /// Query the database with a specific query object
    ///
    /// Request data from alexandria via a `Query` object.  A query
    /// can only touch a single parameter, such as the Record Id, the
    /// path or a set query via tags.  The data returned are snapshots
    /// or records that are immutable.  If you want to make changes to
    /// them, use `update()` with a Diff instead.
    ///
    /// Also: future writes will not propagate to the copy of the
    /// Record returned from this function, because alexandria is
    /// Copy-on-Write.  You will need to query the database again in
    /// the future.
    ///
    /// ## Examples
    ///
    /// This code makes a direct query via the path of a record.  This
    /// will only return a single record if successful.
    ///
    /// ```
    /// # use alexandria::{Builder, GLOBAL, Library, error::Result, utils::{Tag, TagSet, Path}, query::Query};
    /// # async fn foo() -> Result<()> {
    /// # let tmp = tempfile::tempdir().unwrap();
    /// # let lib = Builder::new().offset(tmp.path()).build();
    /// let path = Path::from("/msg:alice");
    /// lib.query(GLOBAL, Query::Path(path)).await;
    /// # Ok(()) }
    /// ```
    ///
    /// ### Search tags
    ///
    /// In alexandria you can tag records with extra metadata (which
    /// is also encrypted), to make queries easier and even build
    /// relationships between records in your application.  These tags
    /// are String-keyed, with an arbitrary (or no) payload and can be
    /// used to make more precise (and fast!) search queries into the
    /// database.
    ///
    /// The constraints imposed by tag queries are modelled on set
    /// theory and can be created via the [`TagQuery`][tq] helper type.
    ///
    /// [tq]: query/struct.TagQuery.html
    ///
    /// Following are a few examples for tag queries.
    ///
    /// ```
    /// # use alexandria::{GLOBAL, Builder, Library, error::Result, utils::{Tag, TagSet, Path}, query::Query};
    /// # async fn foo() -> Result<()> {
    /// # let tmp = tempfile::tempdir().unwrap();
    /// # let lib = Builder::new().offset(tmp.path()).build();
    /// # let tag1 = Tag::new("tag1", vec![1, 3, 1, 2]);
    /// # let tag2 = Tag::new("tag2", vec![13, 12]);
    /// let tags = TagSet::from(vec![tag1, tag2]);
    /// lib.query(GLOBAL, Query::tags().subset(tags)).await;
    /// # Ok(()) }
    /// # async_std::task::block_on(async { foo().await }).unwrap();
    /// ```
    #[tracing::instrument(skip(self), level = "info")]
    pub async fn query<S>(&self, id: S, q: Query) -> Result<QueryResult>
    where
        S: Into<Session> + Debug,
    {
        let id = id.into();
        if let Session::Id(id) = id {
            self.users.read().await.is_open(id)?;
            info!("Passed open-auth for id `{}`", id.to_string());
        }

        let store = self.store.read().await;
        match q {
            Query::Path(ref path) => store.get_path(id, path).map(|rec| QueryResult::Single(rec)),
            Query::Tag(query) => {
                let tc = self.tag_cache.read().await;

                match query {
                    SetQuery::Intersect(ref tags) => tc.get_paths(id, |o| o.intersect(tags)),
                    SetQuery::Subset(ref tags) => tc.get_paths(id, |o| o.subset(tags)),
                    SetQuery::Equals(ref tags) => tc.get_paths(id, |o| o.equality(tags)),
                    SetQuery::Not(ref tags) => tc.get_paths(id, |o| o.not(tags)),
                }
                .iter()
                .map(|p| store.get_path(id, p))
                .collect::<Result<Vec<_>>>()
                .map(|vec| QueryResult::Many(vec))
            }
            _ => unimplemented!(),
        }
    }

    /// Create an iterator from a database query
    ///
    /// The primary difference between this function and `query()` is
    /// that no records are returned or loaded immediately from the
    /// database.  Instead a query is stored, sized and estimated at
    /// the time of querying and can then be stepped through.  This
    /// allows for fetching only a range of objects, limiting memory
    /// usage.
    ///
    /// Paths that are inserted after the `QueryIterator` was
    /// constructed aren't automatically added to it, because it's
    /// internal state is atomic for the time it was created.  If you
    /// want to get updates to the database as they happen, consider a
    /// `Subscription` instead.
    ///
    /// Following is an example for an iterator query, mirroring most
    /// of the `query()` usage quite closely.
    ///
    /// ```
    /// # use alexandria::{GLOBAL, Builder, Library, error::Result, utils::{Tag, TagSet, Path}, query::Query};
    /// # async fn foo() -> Result<()> {
    /// # let tmp = tempfile::tempdir().unwrap();
    /// # let lib = Builder::new().offset(tmp.path()).build();
    /// # let tag1 = Tag::new("tag1", vec![1, 3, 1, 2]);
    /// # let tag2 = Tag::new("tag2", vec![13, 12]);
    /// let tags = TagSet::from(vec![tag1, tag2]);
    /// let iter = lib
    ///     .query_iter(GLOBAL, Query::tags().equals(tags))
    ///     .await?;
    /// iter.skip(5);
    /// let rec = iter.next().await;
    /// # Ok(()) }
    /// ```
    ///
    /// ## Garbage collection
    ///
    /// By default, garbage collection isn't locked for paths that are
    /// included in an iterator.  What this means is that any `delete`
    /// call can remove records that will at some point be accessed by
    /// the returned iterator, resulting in an `Err(_)` return.  To
    /// avoid this race condition, you can call `lock()` on the
    /// iterator, which blocks alexandria from cleaning the iternal
    /// record representation for items that are supposed to be
    /// accessed by the iterator.
    ///
    /// **Note:** `query` may still return "No such path" for these
    /// items, since they were already deleted from the tag cache.
    /// And a caveat worth mentioning: if the program aborts before
    /// the Iterator `drop` was able to run, the items will not be
    /// cleaned from disk and reloaded into cache on restart.
    #[tracing::instrument(skip(self), level = "info")]
    pub async fn query_iter<S>(self: &Arc<Self>, id: S, q: Query) -> Result<QueryIterator>
    where
        S: Into<Session> + Debug,
    {
        let id = id.into();
        if let Session::Id(id) = id {
            self.users.read().await.is_open(id)?;
            info!("Passed open-auth for id `{}`", id.to_string());
        }

        Ok(QueryIterator::new(
            id,
            match q {
                Query::Path(ref p) => vec![p.clone()],
                Query::Tag(ref tq) => {
                    let tc = self.tag_cache.read().await;
                    match tq {
                        SetQuery::Intersect(ref tags) => tc.get_paths(id, |o| tags.intersect(o)),
                        SetQuery::Subset(ref tags) => {
                            // FIXME: I don't really know why this
                            // operation needs to be asymptotic, but
                            // some operations seem to be backwards?
                            // In either case, this works but we
                            // should figure out why this is.
                            tc.get_paths(id, |o| tags.subset(o) || o.subset(tags))
                        }
                        SetQuery::Equals(ref tags) => tc.get_paths(id, |o| tags.equality(o)),
                        SetQuery::Not(ref tags) => tc.get_paths(id, |o| tags.not(o)),
                    }
                }
                _ => unimplemented!(),
            },
            Arc::clone(self),
            q,
        ))
    }

    /// Subscribe to future database updates via a query filter
    ///
    /// When querying repeatedly isn't an option, or would lead to
    /// decreased performance, it's also possible to register a
    /// subscription.  They use the same mechanism as Queries to
    /// filter through tags and paths, but return a type that can be
    /// async-polled for updates.
    ///
    /// This doesn't give immediate access to the data, only the path
    /// that was changed, but can then be used to make a real query
    /// into the database to get an updated set of data.
    ///
    /// ```
    /// # use alexandria::{GLOBAL, Builder, Library, error::Result, utils::{Tag, TagSet, Path}, query::{Query, SetQuery}};
    /// # async fn foo() -> Result<()> {
    /// # let tmp = tempfile::tempdir().unwrap();
    /// # let lib = Builder::new().offset(tmp.path()).build();
    /// # let my_tag = Tag::new("tag1", vec![1, 3, 1, 2]);
    /// let tags = TagSet::from(vec![my_tag]);
    /// let sub = lib.subscribe(GLOBAL, Query::tags().subset(tags)).await?;
    ///
    /// let path = sub.next().await;
    /// let new_data = lib.query(GLOBAL, Query::Path(path)).await?;
    /// # Ok(()) }
    /// ```
    #[tracing::instrument(skip(self), level = "info")]
    pub async fn subscribe<S>(&self, id: S, q: Query) -> Result<Subscription>
    where
        S: Into<Session> + Debug,
    {
        let id = id.into();
        if let Session::Id(id) = id {
            self.users.read().await.is_open(id)?;
            info!("Passed open-auth for id `{}`", id.to_string());
        }

        Ok(self.subs.add_sub(q).await)
    }

    /// Check if a path exists for a particular session
    ///
    /// When inserting data, sometimes it's useful to check the path
    /// that was just inserted, or was inserted by a previous
    /// operation.  Because this is such a common operation, this
    /// utility function aims to make this workflow easier.
    ///
    /// If you want the actual type of a path node, use `query()` instead!
    pub async fn path_exists<S>(&self, id: S, p: Path) -> Result<bool>
    where
        S: Into<Session> + Debug,
    {
        self.query(id, Query::path(p)).await.map(|res| res.single())
    }
}
