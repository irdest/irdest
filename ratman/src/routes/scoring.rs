use crate::{context::RatmanContext, storage::route::RouteData};
use async_trait::async_trait;
use libratman::{
    endpoint::NeighbourMetrics, frame::carrier::AnnounceFrame, types::Address, Result,
};
use std::{collections::BTreeMap, sync::Arc};

use super::EpNeighbourPair;

#[derive(Default)]
pub struct ScorerConfiguration {
    pub trust: BTreeMap<Address, u32>,
    pub available_bw: BTreeMap<EpNeighbourPair, NeighbourMetrics>,
}

#[async_trait]
pub trait RouteScorer {
    /// Provide a mechanism to pre-configure a route scoring module
    ///
    /// A global scorer configuration is kept in memory for the scorer to use for
    /// trust scores and other metadata that can be added in a future revision
    /// of this API.
    ///
    /// This function provides a default no-op implementation because it is
    /// optional for scoring modules which do not keep internal state
    async fn configure(&self, _: &Arc<RatmanContext>, _: &mut ScorerConfiguration) -> Result<()> {
        Ok(())
    }

    /// Capture live announcements to update the scorer module internal state
    ///
    /// This function provides a default no-op implementation because it is
    /// optional for scoring modules which do not keep internal state
    async fn irq_live_announcement(
        &self,
        _: &AnnounceFrame,
        _: &mut ScorerConfiguration,
    ) -> Result<()> {
        Ok(())
    }

    async fn compute(
        &self,
        _stream_size: usize,
        _cfg: &ScorerConfiguration,
        _meta: &RouteData,
    ) -> Result<EpNeighbourPair>;
}

/// The default route selection strategy in use whenever a live link exists
///
/// This strategy mainly uses the captured ping time to a given address, with
/// the available bandwidth as a tie-breaker for connections that have very
/// similar pings (~10% of each other).
pub struct DefaultScorer;

#[async_trait]
impl RouteScorer for DefaultScorer {
    async fn compute(
        &self,
        // Currently unused
        _stream_size: usize,
        _cfg: &ScorerConfiguration,
        _meta: &RouteData,
    ) -> Result<EpNeighbourPair> {
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
    async fn compute(
        &self,
        _stream_size: usize,
        _cfg: &ScorerConfiguration,
        _meta: &RouteData,
    ) -> Result<EpNeighbourPair> {
        todo!()
    }
}
