// SPDX-FileCopyrightText: 2023 Katharina Fey <kookie@spacekookie.de>
//
// SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

// Project internal imports
use crate::{
    api::{self, ConnectionManager},
    config::{
        helpers, netmods::initialise_netmods, peers::PeeringBuilder, ConfigTree, CFG_RATMAND,
    },
    dispatch::BlockCollector,
    journal::Journal,
    links::LinksMap,
    procedures,
    protocol::Protocol,
    routes::RouteTable,
    storage::MetadataDb,
    util::{self, codes, setup_logging, Os, StateDirectoryLock},
};
use libratman::{
    rt::new_async_thread,
    tokio::{sync::mpsc::channel, task::spawn_local},
    Result,
};

// External imports
use atomptr::AtomPtr;
use fjall::Config;
use std::{net::SocketAddr, sync::Arc};
use tripwire::Tripwire;

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
    /// Keep track of blocks, frames, incomplete messages and seen IDs
    pub(crate) journal: Arc<Journal>,
    /// Keep track of network and router metadata
    pub(crate) meta_db: Arc<MetadataDb>,
    /// Current routing table state and query interface for resolution
    pub(crate) routes: Arc<RouteTable>,
    /// Protocol state machines
    pub(crate) protocol: Arc<Protocol>,
    /// Local client connection handler
    pub(crate) clients: Arc<ConnectionManager>,
    /// Indicate the current run state of the router context
    pub(crate) tripwire: Tripwire,
    /// Atomic state directory lock
    ///
    /// If None, ratman is running in ephemeral mode and no data will
    /// be saved this session.  This is usually the case in test
    /// scenarious, but may also be the case on low-power devices.
    _statedir_lock: Arc<AtomPtr<Option<StateDirectoryLock>>>,
}

impl RatmanContext {
    /// Create the in-memory Context, without any initialisation
    // todo: return errors here to allow the journal init to fail gracefully
    pub(crate) async fn new(config: ConfigTree) -> Result<Arc<Self>> {
        let (tripwire, tw_worker) = Tripwire::new_signals();
        let protocol = Protocol::new();

        spawn_local(tw_worker);

        // Initialise storage systems
        let journal_fjall = Config::new(Os::match_os().data_path().join("journal.fjall")).open()?;
        let meta_fjall = Config::new(Os::match_os().data_path().join("metadata.fjall")).open()?;
        let journal = Arc::new(Journal::new(journal_fjall)?);
        let meta_db = Arc::new(MetadataDb::new(meta_fjall)?);

        let links = LinksMap::new();
        let routes = RouteTable::new();

        let collector = BlockCollector::new(Arc::clone(&journal), Arc::clone(&meta_db)).await?;
        let clients = Arc::new(ConnectionManager::new());

        Ok(Arc::new(Self {
            config,
            collector,
            links,
            journal,
            meta_db,
            routes,
            protocol,
            clients,
            tripwire,
            _statedir_lock: Arc::new(AtomPtr::new(None)),
        }))
    }

    /// Create and start a new Ratman router context with a config
    pub async fn start(cfg: ConfigTree) {
        let this = match Self::new(cfg).await {
            Ok(t) => t,
            Err(e) => util::elog(
                format!("failed to initialise/ restore journal state: {e:?}"),
                codes::FATAL,
            ),
        };

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
                        "failed to acquire state directory lock!  Is another ratmand instance running?",
                        codes::FATAL,
                    );
                }
            }
        }

        // This never fails, we will have a map of netmods here, even if it is empty
        initialise_netmods(&this.config, &this.links).await;

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
                let mut peer_builder = PeeringBuilder::new(Arc::clone(&this.links));
                for peer in peers {
                    if let Err(e) = peer_builder.attach(peer.as_str()).await {
                        error!("failed to add peer: {}", e);
                    }
                }

                // If we made it to this point we don't need the
                // peering builder or driver map anymore, so we
                // dissolve both and add everything to the routing
                // core.
                // fixme: integrate this with spawn plans, etc etc
                info!("Driver initialisation complete!");
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
            let _dashboard_bind = ratmand_config
                .get_string_value("dashboard_bind")
                .unwrap_or_else(|| "localhost:8090".to_owned());

            let mut registry = prometheus_client::registry::Registry::default();
            // this.core.register_metrics(&mut registry);
            this.protocol.register_metrics(&mut registry);

            // if let Err(e) = crate::web::start(this.clone(), registry, dashboard_bind) {
            //     error!("failed to start web dashboard server: {}", e);
            // }
        }

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

        // todo: setup management machinery to handle result events
        if let Err(e) = api::start_api_thread(Arc::clone(&this), api_bind_addr).await {
            // todo: setup tripwire here
            util::elog(
                format!("failed to start client handler: {e}"),
                util::codes::FATAL,
            );
        }

        // Setup the ingress system, responsible for collecting all blocks
        // contained in a manifest back into a complete message streams
        let ingress_tx = {
            let (ingress_tx, rx) = channel(32);

            let this_ = Arc::clone(&this);
            let thrx = new_async_thread("ratmand-ingress", 1024 * 32, async move {
                procedures::exec_ingress_system(this_, rx).await;
                Ok(())
            });

            ingress_tx
        };

        // Start the switches and off we go
        {
            let links = Arc::clone(&this.links);

            // todo: make this configurable
            let batch_size = 32;

            // todo: use the configurable netmod runtime here instead
            for (name, ep, id) in links.get_with_ids().await {
                let this_ = Arc::clone(&this);
                let ingress_tx = ingress_tx.clone();

                new_async_thread::<String, _, ()>(
                    format!("ratmand-switch-{name}"),
                    1024,
                    async move {
                        procedures::exec_switching_batch(
                            id,
                            batch_size,
                            &this_.routes,
                            &this_.links,
                            &this_.journal,
                            &this_.collector,
                            this_.tripwire.clone(),
                            (&name, &ep),
                            ingress_tx,
                            // #[cfg(feature = "dashboard")]
                            // todo!()
                        )
                        .await;
                        Ok(())
                    },
                );
            }
        }

        this.tripwire.clone().await;
        info!("Ratmand core shutting down...");
    }

    /// Test whether Ratman is capable of writing anything to disk
    pub fn ephemeral(&self) -> bool {
        self._statedir_lock.get_ref().is_none()
    }
}
