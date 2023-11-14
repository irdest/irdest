//! Shared sync & async runtime utilities
//!
//! ## A WORD OF WARNING!
//!
//! tokio::spawn is FORBIDDEN in this module!  Only ever use
//! tokio::spawn_local!

use crate::{
    rt::writer::{write_u32, AsyncWriter},
    Result,
};
use rand::RngCore;
use std::{
    future::Future,
    sync::{
        mpsc::{sync_channel, Receiver as SyncReceiver, SyncSender},
        Arc,
    },
    time::Duration,
};
use tokio::{
    net::TcpStream,
    runtime::{Builder, Runtime},
    sync::mpsc,
    task::{spawn_local, LocalSet},
    time::{timeout, Timeout},
};

pub mod reader;
pub mod writer;

/// An arbitrary buffer scheme size called "commonbuf"
///
/// Standardises the size of channel buffers based on a common scheme
/// of sub-diving chunk/ block sizes.  This provides a unified
/// mechanism to limit memory size.
///
/// Completely arbitrarily: 8MB divided by the size of a chunk, so 1M
/// chunk => 8 buffer slots.  1K chunk => 8192 buffer slots.
pub const fn size_commonbuf_t<const T: usize>() -> usize {
    (1024 * 1024 * 8) / T
}

/// Encapsulates a single threaded async system
pub struct AsyncSystem {
    label: String,
    rt: Runtime,
    #[allow(unused)]
    set: LocalSet,
    irq: (SyncSender<()>, SyncReceiver<()>),
}

impl AsyncSystem {
    fn new(label: String, stack_mb: usize) -> Arc<Self> {
        Arc::new(Self {
            rt: Builder::new_current_thread()
                .thread_name(&label)
                .enable_io()
                .enable_time()
                .thread_stack_size(1024 * 1024 /* MibiByte */ * stack_mb)
                .build()
                .expect("failed to start async thread!"),
            set: LocalSet::new(),
            label,
            irq: sync_channel(4),
        })
    }

    pub fn async_interrupt(self: &Arc<Self>) {
        let _ = self.irq.0.send(());
    }

    fn root_exec<O: Sized + Send + 'static>(
        &self,
        f: impl Future<Output = Result<O>> + Send + 'static,
    ) -> Result<O> {
        self.rt.block_on(f)
    }
}

/// Spawn new worker thread with an async system launcher
pub fn new_async_thread<S, F, O>(label: S, stack_mb: usize, f: F) -> mpsc::Receiver<Result<O>>
where
    S: Into<String>,
    F: Future<Output = Result<O>> + Send + 'static,
    O: Sized + Send + 'static,
{
    let label = label.into();
    let (tx, rx) = mpsc::channel(8);

    std::thread::spawn(move || {
        let system = AsyncSystem::new(label, stack_mb);
        let res = system.root_exec(f);
        let label = system.label.clone();
        let _ = system.root_exec(async move {
            match res {
                Ok(_) => println!("Worker thread {} completed successfully!", label),
                Err(ref e) => println!("Worker thread {} encountered an error: {}", label, e),
            }

            match tx.send(res).await {
                Ok(_) => println!("Send successful!"),
                Err(e) => println!("Send failed because {}", e),
            }

            Ok(())
        });
    });

    rx
}

#[test]
fn simple_tcp_transfer() {
    use crate::rt::reader::{AsyncVecReader, LengthReader};
    use tokio::net::TcpListener;

    // Receiver
    let mut responder_rx = new_async_thread("tcp server", 32, async move {
        println!("TCP server bind()");
        let l = TcpListener::bind("localhost:5555").await.unwrap();
        println!("Waiting for accept()");
        let (mut stream, _addr) = l.accept().await.unwrap();

        println!("New connection, accepted!");
        let length = LengthReader::new(&mut stream).read_u32().await.unwrap();
        AsyncVecReader::new(length as usize, &mut stream)
            .read_to_vec()
            .await
            .map(|t| {
                println!("Read successful!");
                t
            })
    });

    let mut input_data = vec![0; 1024 * 8];
    rand::thread_rng().fill_bytes(&mut input_data);

    // Sender
    let to_send = input_data.clone();
    new_async_thread("tcp client", 32, async move {
        println!("tcp client :: run()");
        let to_send = to_send.clone();

        let mut stream = timeout(Duration::from_secs(2), TcpStream::connect("localhost:5555"))
            .await
            .unwrap()
            .unwrap();
        println!("Connection successful!");

        write_u32(&mut stream, to_send.len() as u32).await.unwrap();
        AsyncWriter::new(to_send.as_slice(), &mut stream)
            .write_buffer()
            .await
            .unwrap();

        Ok(())
    });

    let main = AsyncSystem::new("main".into(), 1);
    let received_data = main
        .root_exec(async move { responder_rx.recv().await.unwrap() })
        .unwrap();

    println!("DID WE GET DATA?? {:?}", received_data);
    assert_eq!(input_data, received_data);
}
