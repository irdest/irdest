// SPDX-FileCopyrightText: 2023 Katharina Fey <kookie@spacekookie.de>
//
// SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

use std::net::SocketAddr;

use crate::{
    api::{self, ConnectionManager},
    config::{
        helpers, netmods::initialise_netmods, peers::PeeringBuilder, ConfigTree, CFG_RATMAND,
    },
    core::Core,
    crypto::Keystore,
    dispatch::{new_carrier_v1, BlockSlicer, StreamSlicer},
    protocol::Protocol,
    util::{self, codes, runtime_state::RuntimeState, setup_logging, Os, StateDirectoryLock},
};
use async_eris::BlockSize;
use async_std::{sync::Arc, task::block_on};
use atomptr::AtomPtr;
use libratman::{
    netmod::InMemoryEnvelope,
    types::{
        frames::{CarrierFrame, FrameGenerator, ManifestFrame, ManifestFrameV1},
        Address, ApiRecipient, Message, Recipient,
    },
    Result,
};

/// Top-level Ratman router state handle
///
/// This type is responsible for starting and owning various types
/// that control client and driver connections, and internal coherency
/// tasks.
#[derive(Clone)]
pub struct RatmanContext {
    /// Abstraction over the internal routing logic
    pub(crate) core: Arc<Core>,
    /// A protocol state machine
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
    pub(crate) fn new_in_memory() -> Arc<Self> {
        let runtime_state = RuntimeState::start_initialising();
        let protocol = Protocol::new();
        let core = Arc::new(Core::init());
        let keys = Arc::new(Keystore::new());
        let clients = Arc::new(ConnectionManager::new());

        Arc::new(Self {
            core,
            protocol,
            keys,
            clients,
            runtime_state,
            _statedir_lock: Arc::new(AtomPtr::new(None)),
        })
    }

    /// Create and start a new Ratman router context with a config
    pub fn start(cfg: ConfigTree) -> Arc<Self> {
        let this = Self::new_in_memory();

        // Parse the ratmand config tree
        let ratmand_config = cfg.get_subtree(CFG_RATMAND).expect("no 'ratmand' tree");

        // Before we do anything else, make sure we see logs
        setup_logging(&ratmand_config);

        // If ratmand isn't set up to run ephemerally (for tests) try
        // to lock the state directory here and crash if we can't.
        if ratmand_config.get_bool_value("ephemeral").unwrap_or(false) {
            warn!("ratmand is running in ephemeral mode: no data will be persisted to disk");
            warn!("State directory locking is unimplemented");
            warn!("Take care that peering hardware is not used from multiple drivers!");
        } else {
            match block_on(Os::lock_state_directory(None)) {
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
        block_on(this.clients.load_users(&this));

        // This never fails, we will have a map of netmods here, even if it is empty
        let driver_map = block_on(initialise_netmods(&cfg));

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
                    if let Err(e) = block_on(peer_builder.attach(peer.as_str())) {
                        error!("failed to add peer: {}", e);
                    }
                }

                // If we made it to this point we don't need the
                // peering builder or driver map anymore, so we
                // dissolve both and add everything to the routing
                // core.
                for (name, ep) in peer_builder.consume() {
                    let _ep_id = block_on(this.core.add_ep(name, ep));
                }
            }

            // If no peers exist, check if there are alternative
            // peering mechanisms (currently either
            // 'accept_uknown_peers' or having 'lan' discovery
            // enabled).  We print a warning in this case
            None if !ratmand_config
                .get_bool_value("accept_unknown_peers")
                .unwrap_or(false)
                && cfg
                    .get_subtree("lan")
                    .and_then(|tree| tree.get_bool_value("enable"))
                    .unwrap_or(false) =>
            {
                warn!("No peers were provided, but no alternative peering mechanism was detected!")
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
            this.core.register_metrics(&mut registry);
            this.protocol.register_metrics(&mut registry);

            if let Err(e) = block_on(crate::web::start(this.clone(), registry, dashboard_bind)) {
                error!("failed to start web dashboard server: {}", e);
            }
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
        if let Err(e) = block_on(api::run(Arc::clone(&this), api_bind_addr)) {
            // If we returned an error, the API module has crashed
            error!("API connector crashed with error: {}", e);
            this.runtime_state.kill();
        }

        // If we reach this point the router is shutting down
        // (allegedly?)
        this.runtime_state.terminate();

        this
    }

    /// Create a new address
    ///
    /// This function creates a new keypair, inserts the address part
    /// into the local routing table, and starts announcing it to the
    /// rest of the network.
    pub async fn create_new_address(&self) -> Result<Address> {
        let addr = self.keys.create_address().await;
        self.core.add_local_address(addr).await?;
        self.online(addr).await?;
        Ok(addr)
    }

    // TODO: this function must handle address key decryption
    pub async fn load_existing_address(&self, addr: Address, key_data: &[u8]) -> Result<()> {
        self.keys.add_address(addr, key_data).await;
        self.core.add_local_address(addr).await?;
        self.online(addr).await?;
        Ok(())
    }

    async fn online(&self, addr: Address) -> Result<()> {
        // This checks whether the address actually exists first
        self.core.known(addr, true).await?;

        // Then start a new protocol handler task for the address
        Arc::clone(&self.protocol)
            .online(addr, Arc::clone(&self.core))
            .await
    }

    /// Test whether Ratman is capable of writing anything to disk
    pub fn ephemeral(&self) -> bool {
        self._statedir_lock.get_ref().is_none()
    }

    // TODO: should this require some kind of cryptographic challenge maybe ??
    pub async fn set_address_offline(&self, addr: Address) -> Result<()> {
        self.core.known(addr, true).await?;
        self.protocol.offline(addr).await
    }

    /// Dispatch a high-level message
    pub async fn send(self: &Arc<Self>, msg: Message, block_size: BlockSize) -> Result<()> {
        let sender = msg.sender;

        // remap the recipient type because the client API sucks ass
        let recipient = match msg.recipient {
            ApiRecipient::Standard(ref t) => {
                Recipient::Target(*t.first().expect("no recipient in message!"))
            }
            ApiRecipient::Flood(ref t) => Recipient::Flood(*t),
        };

        //let (read_capability, blocks) = BlockSlicer::slice(self, msg, block_size).await?;
        todo!()

        //     let data_frames = StreamSlicer::slice(self, recipient, sender, blocks.into_iter())?;

        //     // Iterate over all the data frames and send them off
        //     for carrier in data_frames {
        //         let mut buffer = vec![];
        //         let meta = carrier.as_meta();

        //         carrier.generate(&mut buffer)?;
        //         let envelope = InMemoryEnvelope { meta, buffer };
        //         self.core.dispatch_frame(envelope).await?;
        //     }

        //     // Create a final manifest frame from the ReadCapability
        //     {
        //         let mut inner_buffer = vec![];
        //         let manifest = ManifestFrame::V1(ManifestFrameV1::from(read_capability));
        //         manifest.generate(&mut inner_buffer)?;

        //         let mut manifest = new_carrier_v1(Some(recipient), sender, None);
        //         manifest.set_payload_checked(1300, inner_buffer)?;

        //         let manifest_carrier = CarrierFrame::V1(manifest);
        //         let meta = manifest_carrier.as_meta();

        //         let mut buffer = vec![];
        //         manifest_carrier.generate(&mut buffer)?;
        //         let manifest_env = InMemoryEnvelope { meta, buffer };

        //         // Finally send off the manifest
        //         self.core.dispatch_frame(manifest_env).await
        //     }
    }
}
