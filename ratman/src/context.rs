// SPDX-FileCopyrightText: 2023 Katharina Fey <kookie@spacekookie.de>
//
// SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

use crate::{
    api::ConnectionManager,
    config::{helpers, netmods::initialise_netmods, ConfigTree, CFG_RATMAND},
    core::Core,
    crypto::Keystore,
    protocol::Protocol,
    util::{runtime_state::RuntimeState, setup_logging},
};
use async_std::sync::Arc;

/// Top-level Ratman router state handle
///
/// This type is responsible for starting and owning various types
/// that control client and driver connections, and internal coherency
/// tasks.
pub struct RatmanContext {
    /// Abstraction over the internal routing logic
    pub(crate) core: Arc<Core>,
    /// A protocol state machine
    pub(crate) protocol: Arc<Protocol>,
    /// Cryptographic store for local address keys
    pub(crate) keys: Arc<Keystore>,
    /// Local client connection handler
    pub(crate) clients: ConnectionManager,
    /// Indicate the current run state of the router context
    // TODO: change this to be an AtomPtr
    runtime_state: RuntimeState,
}

impl RatmanContext {
    /// Create and start a new Ratman router context with a config
    pub async fn start(cfg: ConfigTree) -> Arc<Self> {
        let runtime_state = RuntimeState::start_initialising();

        let protocol = Protocol::new();
        let core = Arc::new(Core::init());
        let keys = Arc::new(Keystore::new());
        let clients = ConnectionManager::new();

        let this = Self {
            core,
            protocol,
            keys,
            clients,
            runtime_state,
        };

        let ratmand_config = cfg.get_subtree(CFG_RATMAND).expect("no 'ratmand' tree");

        // Before we do anything else, make sure we see logs
        setup_logging(&ratmand_config);

        // This never fails, we will have a map of netmods here, even if it is empty
        let driver_map = initialise_netmods(&cfg).await;

        // let verbose = ratmand_config.get_value("verbosity");
        // println!("{:#?}", verbose);

        // Get the initial set of peers from the configuration.
        // Either this is done via the `peer_file` field, which is
        // then read and parsed, or via the `peers` list block.  In
        // either way we have to check for encoding problems.
        //
        // FIXME: At this point the peer syntax also hasn't been
        // validated yet!
        let _peers = match ratmand_config
            .get_string_value("peer_file")
            .and_then(|path| helpers::load_peers_file(path).ok())
            .or(ratmand_config.get_string_list_block("peers"))
        {
            Some(peers) => {
                // Load the given file and parse it
                false
            }
            _ => false,
        };

        todo!()
    }

    /// Register metrics with a Prometheus registry.
    #[cfg(feature = "dashboard")]
    pub fn register_metrics(&self, registry: &mut prometheus_client::registry::Registry) {
        self.inner.register_metrics(registry);
        self.proto.register_metrics(registry);
    }
}
