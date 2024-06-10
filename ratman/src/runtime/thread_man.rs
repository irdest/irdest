use std::time::Duration;

use libratman::{
    tokio::{
        sync::mpsc::{channel, Receiver, Sender},
        task::{spawn_local, yield_now},
        time::sleep,
    },
    Result,
};

/// Keep track of error outputs from various async threads
///
/// If a thread encounters an error or panic it will get logged by this task.
/// Technically this is not a full runtime component but instead runs on the
/// main ratmand thread system
#[derive(Clone)]
pub struct AsyncThreadManager {
    cmd: Sender<Receiver<(String, Result<()>)>>,
}

impl AsyncThreadManager {
    pub fn start() -> Self {
        let (tx, mut rx) = channel::<Receiver<(String, Result<()>)>>(4);

        spawn_local(async move {
            let mut receivers = vec![];
            loop {
                // Check if there are new receivers to add to the check-set
                if let Ok(new_recv) = rx.try_recv() {
                    receivers.push(new_recv);
                }

                // todo: replace with select_all(...)
                for rx in &mut receivers {
                    if let Ok((label, res)) = rx.try_recv() {
                        match res {
                            Ok(()) => info!("async thread '{label}' completed successfully!"),
                            Err(e) => error!("async thread '{label}' encountered an error: {e:?}"),
                        }
                    }
                }

                // Pause and yield to avoid creating a nastly busy loop
                sleep(Duration::from_millis(110)).await;
                yield_now().await;
            }
        });

        Self { cmd: tx }
    }

    pub async fn add_receiver(&self, rx: Receiver<(String, Result<()>)>) {
        let _ = self.cmd.send(rx).await;
    }
}
