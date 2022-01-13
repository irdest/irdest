use crate::{daemon::parse, Router};
use async_std::{
    io::Result,
    net::{Incoming, TcpListener, TcpStream},
    stream::StreamExt,
    sync::{Arc, Mutex},
};
use identity::Identity;
use std::collections::BTreeMap;

pub(crate) type OnlineMap = Arc<Mutex<BTreeMap<Identity, Io>>>;

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

/// Keep track of current connections to stream messages to
pub(crate) struct DaemonState<'a> {
    router: Router,
    online: OnlineMap,
    listen: Incoming<'a>,
}

impl<'a> DaemonState<'a> {
    pub(crate) fn new(l: &'a TcpListener, router: Router) -> Self {
        Self {
            online: Default::default(),
            listen: l.incoming(),
            router,
        }
    }

    pub(crate) async fn get_online(&self) -> OnlineMap {
        Arc::clone(&self.online)
    }

    /// Listen for new connections on a socket address
    pub(crate) async fn listen_for_connections(&mut self) -> Result<Option<Io>> {
        while let Some(stream) = self.listen.next().await {
            let mut stream = stream?;

            let (id, _) = match parse::handle_auth(&mut stream, &self.router).await {
                Ok(pair) => {
                    debug!("Successfully authenticated: {:?}", pair.0);
                    pair
                }
                Err(e) => {
                    error!("Encountered error during auth: {}", e);
                    break;
                }
            };

            let io = Io::Tcp(stream);
            self.online.lock().await.insert(id, io.clone());
            return Ok(Some(io));
        }

        Ok(None)
    }
}
