use crate::{
    core::{Dispatch, DriverMap, Journal, RouteTable, Switch},
    dispatch::BlockCollector,
};
use std::sync::Arc;

/// The core is a set of modules that densely depend on each other
///
/// The core itself exists only as a type container.  All
/// functionality is mapped via the `crate::core::` procedure modules
pub(crate) struct Core {
    pub(crate) collector: Arc<BlockCollector>,
    pub(crate) dispatch: Arc<Dispatch>,
    pub(crate) drivers: Arc<DriverMap>,
    pub(crate) journal: Arc<Journal>,
    pub(crate) routes: Arc<RouteTable>,
}

pub(crate) async fn start_core() -> Core {
    let drivers = DriverMap::new();
    let routes = RouteTable::new();
    let journal = Journal::new();

    let (jtx, jrx) = async_std::channel::bounded(16);
    let collector = BlockCollector::new(jtx);

    let dispatch = Dispatch::new(
        Arc::clone(&routes),
        Arc::clone(&drivers),
        Arc::clone(&collector),
    );

    // Dispatch the runners
    async_std::task::spawn(Arc::clone(&journal).run_block_acceptor(jrx));
    async_std::task::spawn(Arc::clone(&journal).run_message_assembler());

    Core {
        collector,
        dispatch,
        drivers,
        journal,
        routes,
    }
}
