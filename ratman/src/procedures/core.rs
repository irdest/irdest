use crate::{
    core::{Dispatch, Journal, LinksMap, RouteTable},
    dispatch::BlockCollector,
};
use libratman::tokio::{sync::mpsc, task::spawn_local};
use std::sync::Arc;

/// The core is a set of modules that densely depend on each other
///
/// The core itself exists only as a type container.  All
/// functionality is mapped via the `crate::core::` procedure modules
pub(crate) struct Core {
    pub(crate) collector: Arc<BlockCollector>,
    pub(crate) dispatch: Arc<Dispatch>,
    pub(crate) drivers: Arc<LinksMap>,
    pub(crate) journal: Arc<Journal>,
    pub(crate) routes: Arc<RouteTable>,
}

pub(crate) async fn start_core() -> Core {
    let drivers = LinksMap::new();
    let routes = RouteTable::new();
    let journal = Journal::new();

    let (jtx, jrx) = mpsc::channel(16);
    let collector = BlockCollector::new(jtx);

    let dispatch = Dispatch::new(
        Arc::clone(&routes),
        Arc::clone(&drivers),
        Arc::clone(&collector),
    );

    // Dispatch the runners
    spawn_local(Arc::clone(&journal).run_block_acceptor(jrx));
    spawn_local(Arc::clone(&journal).run_message_assembler());

    Core {
        collector,
        dispatch,
        drivers,
        journal,
        routes,
    }
}
