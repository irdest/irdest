use crate::{
    daemon::{env_xdg_data, parse},
    Router,
};
use async_std::{
    io::Result,
    net::{Incoming, TcpListener, TcpStream},
    stream::StreamExt,
    sync::{Arc, Mutex},
    task::{block_on, spawn_blocking},
};
use directories::ProjectDirs;
use identity::Identity;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{File, OpenOptions},
    io::{Read, Write},
    path::PathBuf,
};

pub(crate) type OnlineMap = Arc<Mutex<BTreeMap<Identity, Option<Io>>>>;

#[derive(Clone)]
pub(crate) enum Io {
    Tcp(TcpStream),
}

impl Io {
    pub(crate) fn as_io(&mut self) -> &mut (impl async_std::io::Write + async_std::io::Read) {
        match self {
            Self::Tcp(ref mut stream) => stream,
        }
    }
}

async fn load_users(router: &Router, path: PathBuf) -> Vec<Identity> {
    debug!("Loading registered users from file {:?}", path);
    let mut f = match File::open(path) {
        Ok(f) => f,
        Err(_) => return vec![],
    };

    let mut json = String::new();
    match f.read_to_string(&mut json) {
        Ok(_) => {}
        Err(_) => return vec![],
    }

    match serde_json::from_str::<Vec<Identity>>(&json) {
        Ok(vec) => {
            for addr in &vec {
                trace!("Loading addr {}", addr);
                let e1 = router.add_user(*addr).await;
                let e2 = router.online(*addr).await;

                if e1.is_err() || e2.is_err() {
                    warn!("Failed to load address: {}", addr);
                }
            }

            vec
        }
        Err(_) => vec![],
    }
}

fn data_path(dirs: &ProjectDirs) -> PathBuf {
    let data_dir = env_xdg_data()
        .map(|path| PathBuf::new().join(path))
        .unwrap_or_else(|| dirs.data_dir().to_path_buf());
    trace!("Ensure data directory exists: {:?}", data_dir);
    let _ = std::fs::create_dir(&data_dir);
    PathBuf::new().join(data_dir).join("users.json")
}

/// Keep track of current connections to stream messages to
pub(crate) struct DaemonState<'a> {
    router: Router,
    online: OnlineMap,
    listen: Incoming<'a>,
    dirs: ProjectDirs,
}

impl<'a> DaemonState<'a> {
    pub(crate) fn new(l: &'a TcpListener, router: Router) -> Self {
        let dirs = ProjectDirs::from("org", "irdest", "ratmand")
            .expect("Failed to initialise project directories");

        let path = data_path(&dirs);
        let r2 = router.clone();
        let online = block_on(async move {
            load_users(&r2, path)
                .await
                .into_iter()
                .map(|id| (id, None))
                .collect()
        });

        Self {
            online: Arc::new(Mutex::new(online)),
            listen: l.incoming(),
            router,
            dirs,
        }
    }

    /// Call this function after new user registrations to ensure we
    /// remember them next time
    pub(crate) async fn sync_users(&self) -> Result<()> {
        fn sync_blocking(path: PathBuf, users: Vec<Identity>) -> Result<()> {
            let mut f = OpenOptions::new()
                .create(true)
                .truncate(true)
                .write(true)
                .read(true)
                .open(path)?;
            let mut map = BTreeSet::new();

            users.iter().for_each(|id| {
                map.insert(id);
            });
            let json = serde_json::to_string_pretty(&map)?;

            f.write_all(json.as_bytes())?;
            Ok(())
        }

        let path = data_path(&self.dirs);
        let ids: Vec<_> = self.online.lock().await.iter().map(|(k, _)| *k).collect();
        spawn_blocking(move || sync_blocking(path, ids)).await?;
        Ok(())
    }

    pub(crate) async fn get_online(&self) -> OnlineMap {
        Arc::clone(&self.online)
    }

    /// Listen for new connections on a socket address
    pub(crate) async fn listen_for_connections(&mut self) -> Result<Option<Io>> {
        while let Some(stream) = self.listen.next().await {
            let mut stream = stream?;

            let (id, _) = match parse::handle_auth(&mut stream, &self.router).await {
                Ok(Some(pair)) => {
                    debug!("Successfully authenticated: {:?}", pair.0);
                    pair
                }
                // An anonymous client doesn't need an entry in the
                // lookup table because no message will ever be
                // addressed to it
                Ok(None) => return Ok(Some(Io::Tcp(stream))),
                Err(e) => {
                    error!("Encountered error during auth: {}", e);
                    break;
                }
            };

            let io = Io::Tcp(stream);
            self.online.lock().await.insert(id, Some(io.clone()));

            if let Err(e) = self.sync_users().await {
                error!("Failed to sync known addresses: {}", e);
            }

            return Ok(Some(io));
        }

        Ok(None)
    }
}
