// SPDX-FileCopyrightText: 2023 Katharina Fey <kookie@spacekookie.de>
//
// SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

// Project internal imports
use crate::{
    api::{self, ConnectionManager},
    config::{
        helpers, netmods::initialise_netmods, peers::PeeringBuilder, ConfigTree, CFG_RATMAND,
    },
    core::{Journal, LinksMap, RouteTable},
    crypto::Keystore,
    dispatch::{BlockCollector, BlockSlicer, StreamSlicer},
    protocol::Protocol,
    util::{self, codes, runtime_state::RuntimeState, setup_logging, Os, StateDirectoryLock},
};
use libratman::{
    frame::{
        carrier::{CarrierFrameHeader, CarrierFrameHeaderV1, ManifestFrame, ManifestFrameV1},
        FrameGenerator,
    },
    rt::AsyncSystem,
    tokio::sync::mpsc::Receiver,
    types::{Address, InMemoryEnvelope, Letterhead, Recipient, SequenceIdV1},
    Result,
};

// External imports
use async_eris::{BlockSize, ReadCapability};
use atomptr::AtomPtr;
use std::{net::SocketAddr, sync::Arc};

/// Top-level Ratman router state handle
///
/// This type is responsible for starting and owning various types
/// that control client and driver connections, and internal coherency
/// tasks.
// #[derive(Clone)]
pub struct RatmanContext {
    /// Keep a version of the launch configuration around
    pub(crate) config: ConfigTree,
    /// Responsible for collecting individual frames back into blocks
    pub(crate) collector: Arc<BlockCollector>,
    /// Runtime management of connected network drivers
    pub(crate) links: Arc<LinksMap>,
    /// Keeps track of undeliverable blocks, frames, incomplete
    /// messages, message IDs and other network activities.
    pub(crate) journal: Arc<Journal>,
    /// Current routing table state and query interface for resolution
    pub(crate) routes: Arc<RouteTable>,
    /// Protocol state machines
    pub(crate) protocol: Arc<Protocol>,
    /// Cryptographic store for local address keys
    pub(crate) keys: Arc<Keystore>,
    /// Local client connection handler
    pub(crate) clients: Arc<ConnectionManager>,
    /// Indicate the current run state of the router context
    // TODO: change this to be an AtomPtr
    runtime_state: RuntimeState,
    /// Atomic state directory lock
    ///
    /// If None, ratman is running in ephemeral mode and no data will
    /// be saved this session.  This is usually the case in test
    /// scenarious, but may also be the case on low-power devices.
    _statedir_lock: Arc<AtomPtr<Option<StateDirectoryLock>>>,
}

impl RatmanContext {
    /// Create the in-memory Context, without any initialisation
    pub(crate) fn new_in_memory(config: ConfigTree) -> (Arc<Self>, Receiver<Letterhead>) {
        let runtime_state = RuntimeState::start_initialising();
        let protocol = Protocol::new();
        let (collector, links, journal, routes, lh_notify_recv) = crate::core::exec_core_loops();
        let keys = Arc::new(Keystore::new());
        let clients = Arc::new(ConnectionManager {});

        (
            Arc::new(Self {
                config,
                collector,
                links,
                journal,
                routes,
                protocol,
                keys,
                clients,
                runtime_state,
                _statedir_lock: Arc::new(AtomPtr::new(None)),
            }),
            lh_notify_recv,
        )
    }

    /// Create and start a new Ratman router context with a config
    pub async fn start(cfg: ConfigTree) -> Arc<Self> {
        let (this, lh_notify_recv) = Self::new_in_memory(cfg);

        // Parse the ratmand config tree
        let ratmand_config = this
            .config
            .get_subtree(CFG_RATMAND)
            .expect("no 'ratmand' tree");

        // Before we do anything else, make sure we see logs
        setup_logging(&ratmand_config);

        // If ratmand isn't set up to run ephemerally (for tests) try
        // to lock the state directory here and crash if we can't.
        if ratmand_config.get_bool_value("ephemeral").unwrap_or(false) {
            warn!("ratmand is running in ephemeral mode: no data will be persisted to disk");
            warn!("State directory locking is unimplemented");
            warn!("Take care that peering hardware is not used from multiple drivers!");
        } else {
            match Os::lock_state_directory(None).await {
                Ok(Some(lock)) => {
                    this._statedir_lock.swap(Some(lock));
                }
                Ok(None) => {}
                Err(_) => {
                    util::elog(
                        "failed to acquire state directory lock!  terminating...",
                        codes::FATAL,
                    );
                }
            }
        }

        // Load existing client/address relations
        // this.clients.load_users(&this).await;

        // This never fails, we will have a map of netmods here, even if it is empty
        let driver_map = initialise_netmods(&this.config).await;

        // Get the initial set of peers from the configuration.
        // Either this is done via the `peer_file` field, which is
        // then read and parsed, or via the `peers` list block.  In
        // either way we have to check for encoding problems.
        //
        // FIXME: At this point the peer syntax also hasn't been
        // validated yet!
        match ratmand_config
            .get_string_value("peer_file")
            .and_then(|path| helpers::load_peers_file(path).ok())
            .or(ratmand_config.get_string_list_block("peers"))
        {
            // If peers exist, add them to the drivers
            Some(peers) => {
                let mut peer_builder = PeeringBuilder::new(driver_map);
                for peer in peers {
                    if let Err(e) = peer_builder.attach(peer.as_str()).await {
                        error!("failed to add peer: {}", e);
                    }
                }

                // If we made it to this point we don't need the
                // peering builder or driver map anymore, so we
                // dissolve both and add everything to the routing
                // core.
                for (name, ep) in peer_builder.consume() {
                    let _ep_id = this.links.add(name, ep).await;
                    // fixme: integrate this with spawn plans, etc etc
                }
            }

            // If no peers exist, check if there are alternative
            // peering mechanisms (currently either
            // 'accept_uknown_peers' or having 'lan' discovery
            // enabled).  We print a warning otherwise
            None if !ratmand_config
                .get_bool_value("accept_unknown_peers")
                .unwrap_or(false)
                & /* and */ this
                    .config
                    .get_subtree("lan")
                    .and_then(|tree| tree.get_bool_value("enable"))
                    .unwrap_or(false) =>
            {
                warn!("No peers were provided, but no alternative peering mechanism was detected!  You won't be able to talk to anyone.")
            }

            // If no peers exist, but other peering mechanisms exist
            _ => {}
        };

        // If the dashboard feature and configuration is enabled
        #[cfg(feature = "dashboard")]
        if let Some(true) = ratmand_config.get_bool_value("enable_dashboard") {
            let dashboard_bind = ratmand_config
                .get_string_value("dashboard_bind")
                .unwrap_or_else(|| "localhost:8090".to_owned());

            let mut registry = prometheus_client::registry::Registry::default();
            // this.core.register_metrics(&mut registry);
            this.protocol.register_metrics(&mut registry);

            // if let Err(e) = crate::web::start(this.clone(), registry, dashboard_bind) {
            //     error!("failed to start web dashboard server: {}", e);
            // }
        }

        // At this point we can mark the router as having finished initialising
        this.runtime_state.finished_initialising();

        // Finally, we start the machinery that accepts new client
        // connections.  We hand it a complete (atomic reference) copy
        // of the router state context.
        let api_bind = ratmand_config
            .get_string_value("api_bind")
            .unwrap_or_else(|| format!("localhost:9020"))
            // FIXME: there must be a better way to do this lol
            .replace("localhost", "127.0.0.1");

        let api_bind_addr: SocketAddr = match api_bind.parse() {
            Ok(bind) => bind,
            Err(e) => {
                util::elog(
                    format!("failed to parse API bind address '{}': {}", api_bind, e),
                    util::codes::INVALID_PARAM,
                );
            }
        };

        // We block execution on running the API module
        // if let Err(e) = api::run(Arc::clone(&this), api_bind_addr) {
        //     // If we returned an error, the API module has crashed
        //     error!("API connector crashed with error: {}", e);
        //     this.runtime_state.kill();
        // }

        libratman::tokio::time::sleep(std::time::Duration::from_secs(2)).await;

        // If we reach this point the router is shutting down
        // (allegedly?)
        this.runtime_state.terminate();

        this
    }

    // /// Create a new address
    // ///
    // /// This function creates a new keypair, inserts the address part
    // /// into the local routing table, and starts announcing it to the
    // /// rest of the network.
    // pub async fn create_new_address(self: &Arc<Self>) -> Result<Address> {
    //     let addr = self.keys.create_address().await;
    //     self.core.add_local_address(addr).await?;
    //     self.online(addr).await?;
    //     Ok(addr)
    // }

    // // TODO: this function must handle address key decryption
    // pub async fn load_existing_address(
    //     self: &Arc<Self>,
    //     addr: Address,
    //     key_data: &[u8],
    // ) -> Result<()> {
    //     self.keys.add_address(addr, key_data).await;
    //     self.core.add_local_address(addr).await?;
    //     self.online(addr).await?;
    //     Ok(())
    // }

    // async fn online(self: &Arc<Self>, addr: Address) -> Result<()> {
    //     // This checks whether the address actually exists first
    //     self.core.known(addr, true).await?;

    //     // Then start a new protocol handler task for the address
    //     Arc::clone(&self.protocol)
    //         .online(addr, Arc::clone(self))
    //         .await
    // }

    /// Test whether Ratman is capable of writing anything to disk
    pub fn ephemeral(&self) -> bool {
        self._statedir_lock.get_ref().is_none()
    }

    // // TODO: should this require some kind of cryptographic challenge maybe ??
    // pub async fn set_address_offline(&self, addr: Address) -> Result<()> {
    //     self.core.known(addr, true).await?;
    //     self.protocol.offline(addr).await
    // }

    // /// Dispatch a high-level message
    // // todo: this function should do a few more things around the
    // // journal!
    // //
    // // - If sending fails, insert the frame into the journal frame queue
    // // - Insert created blocks into the journal for future store & forward
    // // - Slice manifest correctly if needed
    // pub async fn send(self: &Arc<Self>, mut msg: Message) -> Result<()> {
    //     let sender = msg.sender;

    //     // remap the recipient type because the client API sucks ass
    //     let recipient = match msg.recipient {
    //         ApiRecipient::Standard(ref t) => {
    //             Recipient::Target(*t.first().expect("no recipient in message!"))
    //         }
    //         ApiRecipient::Flood(ref t) => Recipient::Flood(*t),
    //     };

    //     // Turn the message into blocks, based on the size of the
    //     // message payload.  Currently the cut-off between 1K blocks
    //     // and 32K blocks is 14kb
    //     let selected_block_size = if msg.payload.len() > 14336 {
    //         BlockSize::_32K
    //     } else {
    //         BlockSize::_1K
    //     };

    //     let (read_capability, mut blocks) =
    //         BlockSlicer::slice(self, &mut msg, selected_block_size).await?;

    //     // Then turn the set of blocks into a series of MTU-sliced
    //     // carrier frames
    //     let carriers = StreamSlicer::slice(self, recipient, sender, blocks.clone().into_iter())?;
    //     info!(
    //         "Message ID {} resulted in {} blocks (of size {}), yielding {} carrier frames",
    //         msg.id,
    //         blocks.len(),
    //         match selected_block_size {
    //             BlockSize::_1K => "1K",
    //             _ => "32K",
    //         },
    //         carriers.len()
    //     );

    //     // Iterate over all the data frames and send them off
    //     for envelope in carriers {
    //         self.core.dispatch.dispatch_frame(envelope).await?;
    //     }

    //     // Finally create a Manifest frame and send that off last
    //     {
    //         let manifest_frame = ManifestFrame::V1(ManifestFrameV1::from(read_capability));
    //         let mut manifest_payload = vec![];
    //         manifest_frame.generate(&mut manifest_payload)?;

    //         // todo: currently we don't try to slice the manifest at
    //         // all.  If it gets too big it may not fit across every
    //         // transport channel.
    //         let m_header = CarrierFrameHeader::new_blockmanifest_frame(
    //             sender,
    //             recipient,
    //             SequenceIdV1 {
    //                 hash: msg.get_id(),
    //                 num: 1,
    //                 max: 1,
    //             },
    //             manifest_payload.len() as u16,
    //         );

    //         let m_envelope = InMemoryEnvelope::from_header_and_payload(m_header, manifest_payload)?;
    //         self.core.dispatch.dispatch_frame(m_envelope).await?;
    //         trace!("Sending message manifest was successful!");
    //     }

    //     Ok(())
    // }
}
