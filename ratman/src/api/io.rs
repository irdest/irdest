use async_std::{
    channel::{self, Receiver, Sender},
    future::timeout,
    net::TcpStream,
    sync::{Arc, Mutex},
    task,
};
use libratman::{
    types::{api::ApiMessage, parse_message, write_with_length},
    NonfatalError, RatmanError, Result,
};
use once_cell::sync::Lazy;
use std::{
    sync::atomic::{AtomicUsize, Ordering},
    time::Duration,
};

/// A simple counter for generating linear IDs
pub struct StreamHandle(AtomicUsize, Receiver<Vec<u8>>);

impl StreamHandle {
    const fn default_const() -> Lazy<(Arc<Self>, Sender<Vec<u8>>)> {
        Lazy::new(|| {
            let (tx, rx) = channel::bounded(64);
            let this = Arc::new(Self(0.into(), rx));
            (this, tx)
        })
    }

    fn next(&self) -> usize {
        self.0.fetch_add(1, Ordering::AcqRel)
    }
}

/// I/O socket abstraction for a client application
///
/// We use this indirection here to allow future sockets to use
/// different formats (for example seqpack unix sockets).  Use
/// `[as_io()](Self::as_io) to get access to the underlying `Read +
/// Write` stream.
#[derive(Clone)]
pub enum Io {
    Tcp(Arc<Mutex<TcpStream>>),
}

impl Io {
    pub async fn read_message(&self) -> Result<ApiMessage> {
        let mut ctr = 0;
        while ctr < 7 {
            match self.read_message_inner().await {
                Ok(msg) => return Ok(msg),
                Err(RatmanError::Nonfatal(NonfatalError::NoRead)) => {
                    ctr += 1;
                    continue;
                }
                Err(e) => return Err(e),
            }
        }

        // Drop the connection if a read fails for the final time
        Err(RatmanError::ClientApi(
            libratman::ClientError::ConnectionLost,
        ))
    }

    pub async fn read_message_inner(&self) -> Result<ApiMessage> {
        match self {
            Self::Tcp(ref stream) => {
                match timeout(Duration::from_millis(79), async move {
                    let mut reader = stream.lock().await;
                    parse_message(&mut *reader).await
                })
                .await
                {
                    Ok(Ok(msg)) => Ok(msg),
                    Ok(Err(e)) => {
                        warn!("failed reading from TcpStream: {}", e);
                        Err(e)
                    }
                    Err(_) => Err(NonfatalError::NoRead.into()),
                }
            }
        }
    }

    pub async fn send_to(&self, message: Vec<u8>) {
        match self {
            Self::Tcp(ref stream) => {
                static COUNTER: Lazy<(Arc<StreamHandle>, Sender<Vec<u8>>)> =
                    StreamHandle::default_const();
                debug!("Hand out TcpStream {} ...", COUNTER.0.next());

                {
                    let future_this = Arc::clone(&COUNTER.0);
                    let future_io = Arc::clone(&stream);
                    task::spawn(async move {
                        while let Ok(to_send) = future_this.1.recv().await {
                            if let Err(e) =
                                write_with_length(&mut *future_io.lock().await, &to_send).await
                            {
                                warn!("Failed to send to API socket: {}", e);
                            }
                        }
                    });
                }

                COUNTER.1.send(message).await;
            }
        }
    }
}
