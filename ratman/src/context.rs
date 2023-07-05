// SPDX-FileCopyrightText: 2023 Katharina Fey <kookie@spacekookie.de>
//
// SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

use crate::{
    api::ConnectionManager, config::ConfigTree, core::Core, crypto::Keystore, protocol::Protocol,
    util::runtime_state::RuntimeState,
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
        let mut runtime_state = RuntimeState::start_initialising();

        let protocol = Protocol::new();
        let core = Arc::new(Core::init());
        let keys = Arc::new(Keystore::new());
        let clients = ConnectionManager::new();

        todo!()
    }

    /// Register metrics with a Prometheus registry.
    #[cfg(feature = "dashboard")]
    pub fn register_metrics(&self, registry: &mut prometheus_client::registry::Registry) {
        self.inner.register_metrics(registry);
        self.proto.register_metrics(registry);
    }
}
