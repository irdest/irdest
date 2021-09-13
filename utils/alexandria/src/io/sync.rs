use crate::{
    delta::{Delta, DeltaType},
    dir::Dirs,
    error::{Error, Result},
    io::format,
    utils::Path,
    Library,
};
use async_std::{sync::Mutex, task};
use std::{
    collections::VecDeque,
    fs::{self, OpenOptions as Open},
    io::Write,
    path::PathBuf,
    sync::Arc,
    time::Duration,
};

/// Persistence synchronisation module
///
/// Each of the following modules needs to be synced
///
/// - `UserTags`
/// - `TagCache`
/// - `Store` (both data, and GC lock state)
///
/// In the future we must also consider the data caches!
pub struct Sync {
    dirs: Dirs,
    dirty: Mutex<VecDeque<Delta>>,
}

impl Sync {
    pub(crate) fn new(dirs: Dirs) -> Arc<Self> {
        Arc::new(Self {
            dirs,
            dirty: Default::default(),
        })
    }

    /// Start the synchronisation engine and related sub-tasks
    pub(crate) async fn start(self: Arc<Self>, lib: Arc<Library>) -> Result<()> {
        if let Some(meta) = match fs::metadata(&self.dirs.root()) {
            Ok(meta) => Some(meta),
            _ => {
                warn!("Unable to get offset path metadata; permission problems may occur!");
                None
            }
        } {
            // Reject directories that can't be written into
            if meta.permissions().readonly() {
                return Err(Error::SyncFailed {
                    msg: "Library directory not writable!".into(),
                });
            }
        }

        // Start a task to sync the store
        {
            let sync = Arc::clone(&self);
            let lib = Arc::clone(&lib);
            task::spawn(async move {
                info!("Starting synchronisation task...");
                loop {
                    {
                        let mut vec = sync.dirty.lock().await;
                        trace!("Syncing {} records", vec.len());

                        if let Err(e) = write_store(&lib, &sync.dirs, &vec).await {
                            error!("Sync::write_store failed: {}", e);
                        };
                        vec.clear();
                    }

                    task::sleep(Duration::from_millis(25)).await;
                }
            });
        }

        Ok(())
    }

    /// Queue a delta to be synced
    pub(crate) fn queue(self: &Arc<Self>, d: &Delta) {
        let this = Arc::clone(self);
        let d = d.clone();
        task::spawn(async move {
            let mut dirty = this.dirty.lock().await;
            debug!("Queuing delta: {:?} {}", d.action, d.path);
            dirty.push_back(d);
        });
    }

    /// Block the current flow of runtime until the sync engine is empty
    pub(crate) fn block_on_clear(self: &Arc<Self>) {
        let this = Arc::clone(self);

        task::block_on(async move {
            loop {
                match this.dirty.lock().await.len() {
                    0 => break,
                    _ => task::sleep(Duration::from_millis(50)).await,
                }
            }
        })
    }
}

async fn write_store(l: &Arc<Library>, dirs: &Dirs, deltas: &VecDeque<Delta>) -> Result<()> {
    for ref d in deltas {
        trace!("Syncing path {}", d.path);
        let path = d.path.clone();
        let fs_path = format::path(dirs, d.rec_id.unwrap());

        // Match on the delta type
        let buf = match d.action {
            DeltaType::Insert | DeltaType::Update => {
                // let store = l.store.read().await;
                // let key = l.users.read().await.get_key(dbg!(d.user.id()).unwrap())?;

                // let e = store.get_encrypted(key, d.user, &d.path)?;
                // Some(format::encode(e))
                todo!()
            }
            DeltaType::Delete => None,
        };

        SyncWriter { fs_path, buf }.run(path.clone())
    }

    Ok(())
}

/// An asynchronous update writer
pub struct SyncWriter {
    fs_path: PathBuf,
    buf: Option<Vec<u8>>,
}

impl SyncWriter {
    fn run(self, path: Path) {
        task::spawn(async move {
            if let Err(e) = match self.buf {
                Some(_) => self.write(),
                None => self.delete(),
            } {
                error!("Error occured while syncing path `{}`: {}", path, e);
            }
        });
    }

    fn write(self) -> Result<()> {
        let buf = self.buf.unwrap();
        let mut f = Open::new()
            .truncate(true)
            .write(true)
            .create(true)
            .open(self.fs_path)
            .unwrap();
        Ok(f.write_all(&buf).unwrap())
    }

    fn delete(self) -> Result<()> {
        Ok(fs::remove_file(self.fs_path)?)
    }
}

// /// This test is still broken -- persistence is unsupported!
// ///
// /// Fixing this test is part of the effort documented in #4, #10, and
// /// #12 (general refactoring of the Alexandria internals)
// #[ignore]
// #[async_std::test]
// async fn write_and_load() -> Result<()> {
//     use tracing_subscriber::{filter::LevelFilter, fmt, EnvFilter};

//     let filter = EnvFilter::try_from_env("IRDEST_LOG")
//         .unwrap_or_default()
//         .add_directive(LevelFilter::TRACE.into())
//         .add_directive("async_std=error".parse().unwrap())
//         .add_directive("async_io=error".parse().unwrap())
//         .add_directive("polling=error".parse().unwrap())
//         .add_directive("mio=error".parse().unwrap());

//     // Initialise the logger
//     fmt().with_env_filter(filter).init();

//     use crate::{
//         query::Query,
//         utils::{Diff, TagSet},
//         Builder, Library,
//     };
//     let tmp = tempfile::tempdir().unwrap();
//     let lib = Builder::new().offset(tmp.path()).build();
//     lib.sync().await?;

//     let id = crate::utils::Id::random();
//     let sess = lib.sessions().create(id, "abcdefg").await.unwrap();

//     lib.insert(
//         sess.clone(),
//         "/:bar".into(),
//         TagSet::empty(),
//         Diff::map().insert("test key", "test value"),
//     )
//     .await?;

//     lib.ensure();
//     drop(lib);

//     let lib = Library::load(tmp.path(), "")?;

//     let rec = lib.query(GLOBAL, Query::Path("/:bar".into())).await?;
//     assert_eq!(
//         rec.as_single().kv().get("test key"),
//         Some(&Value::from("test value"))
//     );

//     Ok(())
// }
