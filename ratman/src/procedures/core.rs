use crate::{
    core::{run_message_assembler, Journal, LinksMap, RouteTable},
    dispatch::BlockCollector,
};
use libratman::{
    tokio::{sync::mpsc, task::spawn_local},
    types::Letterhead,
};
use std::sync::Arc;

/// The core is a set of modules that densely depend on each other
///
/// The core itself exists only as a type container.  All
/// functionality is mapped via the `crate::core::` procedure modules
pub(crate) struct Core {
    pub(crate) collector: Arc<BlockCollector>,
    pub(crate) drivers: Arc<LinksMap>,
    pub(crate) journal: Arc<Journal>,
    pub(crate) routes: Arc<RouteTable>,
}

pub(crate) async fn start_core() -> (Core, mpsc::Receiver<Letterhead>) {
    let drivers = LinksMap::new();
    let routes = RouteTable::new();
    let (journal, m_notify_r) = Journal::new();

    let (jtx, jrx) = mpsc::channel(16);
    let (lh_notify_t, lh_notify_r) = mpsc::channel(16);

    let collector = BlockCollector::new(jtx);

    // Dispatch the runners
    spawn_local(Arc::clone(&journal).run_block_acceptor(jrx));
    spawn_local(run_message_assembler(
        Arc::clone(&journal),
        m_notify_r,
        lh_notify_t,
    ));

    (
        Core {
            collector,
            drivers,
            journal,
            routes,
        },
        lh_notify_r,
    )
}
