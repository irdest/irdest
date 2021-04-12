use crate::{
    messages::MsgUtils,
    users::{Announcer, TAG_PROFILE},
    Irdest,
};
use alexandria::utils::Tag;
use async_std::task;
use ratman::{netmod::Recipient, Router};
use std::sync::Arc;
use tracing::{debug, info, warn};

/// A thread-detached discovery service running inside irdest-core
///
/// ## Required data
///
/// This internal service needs access to both the rest of the `Irdest`
/// structure to access external service registries and user stores,
/// as well as the underlying `Router` of a platform to send messages
/// to and receive from.
///
/// ## Startup
///
/// Startup procedure works pretty closely to how a `Router` is
/// initialised in `ratman`, where initialisation spawns threads, and
/// returns channel endpoints to send messages to the Discovery service.
///
/// Available messages are encoded in the DiscCmd enum.
#[derive(Clone)]
pub(crate) struct Discovery;

impl Discovery {
    /// Start a discovery service running inside irdest-core
    pub(crate) fn start(irdest: Arc<Irdest>, router: Arc<Router>) {
        // Incoming message handler
        Self::inc_handler(Arc::clone(&irdest), Arc::clone(&router));

        // Handle new users
        task::spawn(async move {
            loop {
                let id = router.discover().await;
                debug!(id = id.to_string().as_str(), "Received announcement!");

                if !irdest.users.known_remote().await.contains(&id) {
                    info!(id = id.to_string().as_str(), "Discovered new user!");
                    irdest.users
                        .insert_profile(id, vec![Tag::empty(TAG_PROFILE)])
                        .await;
                }
            }
        });
    }

    /// Spawns a thread that listens to incoming messages
    #[tracing::instrument(skip(irdest, router), level = "info")]
    fn inc_handler(irdest: Arc<Irdest>, router: Arc<Router>) {
        task::spawn(async move {
            loop {
                let msg = router.next().await;
                let sender = msg.sender;

                info!("Receiving message by `{}`...", sender);
                let recp = match msg.recipient {
                    Recipient::User(id) => Some(id),
                    Recipient::Flood => None,
                };

                // Filter internal status messages
                // panic!("Getting an announcer message!!!!");
                if let Some(profile) = Announcer::check_message(&msg) {
                    // If we had a previous version, generate diffs for update
                    if let Some(old) = irdest.users.get(msg.sender).await.ok() {
                        let diff = old.generate_updates(profile);
                        irdest.users.modify(msg.sender, diff).await.unwrap();
                    }

                    continue;
                }

                let msg = match MsgUtils::process(msg, &irdest.users).await {
                    Ok(msg) => Arc::new(msg),
                    Err(_) => {
                        warn!("Skipping malformed message by `{}`", sender);
                        continue;
                    }
                };

                irdest.messages.insert_remote(recp, Arc::clone(&msg)).await;
                info!("Finished processing incoming message!");
            }
        });
    }
}
