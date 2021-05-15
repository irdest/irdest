use crate::{
    delta::Delta,
    dir::Dirs,
    error::{Error, Result},
    io::format,
    utils::Path,
    Library, Session,
};
use async_std::{sync::Mutex, task};
use std::{collections::VecDeque, fs, path::PathBuf, sync::Arc, time::Duration};
use tracing::{error, warn};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum SyncItem {
    Path(Path),
}

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
                loop {
                    {
                        let mut vec = sync.dirty.lock().await;
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
            dirty.push_back(d);
        });
    }
}

async fn write_store(l: &Arc<Library>, dirs: &Dirs, deltas: &VecDeque<Delta>) -> Result<()> {
    for ref d in deltas {
        let fs_path = format::path(dirs, &d.user, &d.path);

        //
    }

    Ok(())
}

/// An asynchronous update writer
pub struct SyncWriter {
    fs_path: PathBuf,
    sess: Session,
    path: Path,
}
