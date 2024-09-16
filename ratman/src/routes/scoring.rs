use crate::{
    context::RatmanContext,
    storage::route::{RouteData, RouteEntry},
};
use async_trait::async_trait;
use libratman::{frame::carrier::AnnounceFrame, types::Address, Result};
use std::sync::Arc;

use super::EpNeighbourPair;

pub struct ScorerConfiguration {
    pub trust: BTreeMap<Address, u32>,
}

#[async_trait]
trait RouteScorer {
    async fn configure(&self, _: &Arc<RatmanContext>, _: &mut ScorerConfiguration) -> Result<()> {
        Ok(())
    }

    async fn irq_live_announcement(&self, a: &AnnounceFrame) -> Result<()> {
        Ok(())
    }

    async fn compute(&self, _stream_size: usize, _meta: &[&RouteData]) -> Result<EpNeighbourPair>;
}

/// The default route selection strategy in use whenever a live link exists
///
/// This strategy mainly uses the captured ping time to a given address, with
/// the available bandwidth as a tie-breaker for connections that have very
/// similar pings (~10% of each other).
pub struct DefaultScorer;

#[async_trait]
impl RouteScorer for DefaultScorer {
    async fn compute(&self, _stream_size: usize, _meta: &[&RouteData]) -> Result<EpNeighbourPair> {
        todo!()
    }
}

/// A fallback route selection strategy for when no live link exists
///
/// This strategy uses the available bandwdith as well as advertised available
/// storage space in neighbouring nodes to select the next connection.  It is
/// possible that this scorer fails to determine a route, if the next hop is
/// full, at which point routing is paused until a link can be established
/// again.
pub struct StoreForwardScorer;

#[async_trait]
impl RouteScorer for StoreForwardScorer {
    async fn compute(&self, _stream_size: usize, _meta: &[&RouteData]) -> Result<EpNeighbourPair> {
        todo!()
    }
}
