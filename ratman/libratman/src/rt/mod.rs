//! Shared sync & async runtime utilities
//!
//! ## A WORD OF WARNING!
//!
//! tokio::spawn is FORBIDDEN in this module!  Only ever use
//! tokio::spawn_local!

use crate::Result;
use std::{
    future::Future,
    sync::{
        mpsc::{sync_channel, Receiver, SyncSender},
        Arc,
    },
};
use tokio::{
    runtime::{Builder, Runtime},
    task::LocalSet,
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
/// chunk => 8 garbage chunks.  1K chunk => 8192 garbage chunks.
pub const fn size_commonbuf_t<const T: usize>() -> usize {
    (1024 * 1024 * 8) / T
}

/// Encapsulates a single threaded async system
pub struct AsyncSystem {
    label: String,
    rt: Runtime,
    set: LocalSet,
    irq: (SyncSender<()>, Receiver<()>),
}

impl AsyncSystem {
    fn new(label: String, stack_mb: usize) -> Arc<Self> {
        Arc::new(Self {
            rt: Builder::new_current_thread()
                .thread_name(&label)
                .thread_stack_size(1024 * 1024 /* MibiByte */ * stack_mb)
                .build()
                .expect("failed to start async thread!"),
            set: LocalSet::new(),
            label,
            irq: sync_channel(4),
        })
    }

    pub fn async_interrupt(self: &Arc<Self>) {
        self.irq.0.send(());
    }

    fn root_exec(&self, f: impl Future<Output = Result<()>> + Send + 'static) -> Result<()> {
        let _g = self.rt.handle().enter();
        let ok = self.rt.handle().block_on(f);
        drop(_g);
        ok
    }
}

/// Spawn new worker thread with an async system launcher
pub fn new_async_thread(
    label: String,
    stack_mb: usize,
    f: impl Future<Output = Result<()>> + Send + 'static,
) {
    std::thread::spawn(move || {
        let system = AsyncSystem::new(label, stack_mb);
        match system.root_exec(f) {
            Ok(_) => info!("Worker thread {} completed successfully!", system.label),
            Err(e) => error!("Worker thread {} encountered an error: {}", system.label, e),
        }
    });
}
