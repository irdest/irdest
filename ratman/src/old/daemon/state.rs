// SPDX-FileCopyrightText: 2019-2022 Katharina Fey <kookie@spacekookie.de>
//
// SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

use crate::{
    daemon::{env_xdg_data, parse},
    storage::addrs::LocalAddress,
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
use std::env::consts::OS;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{File, OpenOptions},
    io::{Read, Write},
    path::PathBuf,
};
use types::Address;

pub(crate) type OnlineMap = Arc<Mutex<BTreeMap<Address, Option<Io>>>>;

async fn load_users(router: &Router, path: PathBuf) -> Vec<Address> {
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

    match serde_json::from_str::<Vec<LocalAddress>>(&json) {
        Ok(vec) => {
            for LocalAddress { ref id, .. } in &vec {
                trace!("Loading addr {}", id);
                let e1 = router.add_existing_user(*id).await;
                let e2 = router.online(*id).await;

                let key_data = [0]; // FIXME
                router.load_address(*id, &key_data).await.unwrap();

                if e1.is_err() || e2.is_err() {
                    warn!("Failed to load address: {}", id);
                }
            }

            vec.into_iter().map(|l| l.id).collect()
        }
        Err(_) => vec![],
    }
}

/// Keep track of current connections to stream messages to
pub(crate) struct DaemonState<'a> {
    router: Router,
    online: OnlineMap,
    listen: Incoming<'a>,
}

impl<'a> DaemonState<'a> {
    pub(crate) fn new(l: &'a TcpListener, router: Router) -> Self {
        let path = Os::match_os().data_path();
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
        }
    }

    /// Call this function after new user registrations to ensure we
    /// remember them next time
    pub(crate) async fn sync_users(&self) -> Result<()> {
        fn sync_blocking(path: PathBuf, users: Vec<LocalAddress>) -> Result<()> {
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

        let path = Os::match_os().data_path();

        let addrs = self.router.local_addrs().await;
        // let ids: Vec<_> = self.online.lock().await.iter().map(|(k, _)| *k).collect();
        spawn_blocking(move || sync_blocking(path, addrs)).await?;
        Ok(())
    }

    pub(crate) async fn get_online(&self) -> OnlineMap {
        Arc::clone(&self.online)
    }

    /// Listen for new connections on a socket address
    pub(crate) async fn listen_for_connections(&mut self) -> Result<Option<(Address, Io)>> {
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
                Ok(None) => return Ok(Some((Address::random(), Io::Tcp(stream)))),
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

            return Ok(Some((id, io)));
        }

        Ok(None)
    }
}