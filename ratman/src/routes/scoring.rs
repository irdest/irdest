use crate::{
    context::RatmanContext,
    storage::route::{RouteData, RouteState},
};
use async_trait::async_trait;
use libratman::{
    endpoint::NeighbourMetrics, frame::carrier::AnnounceFrame, types::Address, NonfatalError,
    RatmanError, Result,
};
use std::{collections::BTreeMap, sync::Arc};

use super::EpNeighbourPair;

#[derive(Default)]
pub struct ScorerConfiguration {
    /// Calculate trust scores for an address
    #[allow(unused)]
    pub trust: BTreeMap<Address, u32>,
    /// Available measured bandwidth for a connection
    pub available_bw: BTreeMap<EpNeighbourPair, NeighbourMetrics>,
    /// Available advertised buffer space for a connection
    pub available_buffer: BTreeMap<EpNeighbourPair, u64>,
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
    #[inline]
    async fn compute(
        &self,
        // Currently unused
        _stream_size: usize,
        cfg: &ScorerConfiguration,
        meta: &RouteData,
    ) -> Result<EpNeighbourPair> {
        // If there's no route available we return an error.  This case SHOULD
        // never occur but you never know
        if meta.link_id.len() == 0 {
            return Err(RatmanError::Nonfatal(NonfatalError::NoAvailableRoute));
        }

        if meta.link_id.len() == 1 {
            // We know there is at least one available link, but we need to check if
            // it's active.  If not we must fail-over into the next scorer
            let link = meta.link_id.get(0).unwrap();
            let link_data = meta.link_data.get(link).unwrap();

            if link_data.state == RouteState::Active {
                return Ok(*link);
            } else {
                return Err(RatmanError::Nonfatal(NonfatalError::NoAvailableRoute));
            }
        }
        // There are at least two links, so we look at both, compare the ping
        // times and use measured bandwidth as a tie-breaker
        else {
            let nb_a = meta.link_id.get(0).unwrap();
            let nb_b = meta.link_id.get(1).unwrap();

            let link_a = meta.link_data.get(nb_a).unwrap();
            let link_b = meta.link_data.get(nb_b).unwrap();

            // Check that both links are, in fact, active
            match (link_a.state, link_b.state) {
                (RouteState::Idle, _)
                | (_, RouteState::Idle)
                | (RouteState::Lost, _)
                | (_, RouteState::Lost) => {
                    return Err(RatmanError::Nonfatal(NonfatalError::NoAvailableRoute));
                }
                _ => {}
            }

            // Calculate a 10% bound around the ping time in ms.  If the values
            // are close, the available bandwidth is used as a tie-breaker
            if link_b.ping.as_millis() >= (link_a.ping.as_millis() * 90 / 100)
                && link_b.ping.as_millis() <= (link_a.ping.as_millis() * 110 / 100)
            {
                let link_a_bw = cfg.available_bw.get(nb_a);
                let link_b_bw = cfg.available_bw.get(nb_b);

                match (link_a_bw, link_b_bw) {
                    (Some(a_bw), Some(b_bw)) if b_bw.write_bandwidth > a_bw.write_bandwidth => {
                        Ok(*nb_b)
                    }
                    _ => Ok(*nb_a),
                }
            }
            // Otherwise: just return the first link
            else {
                Ok(*nb_a)
            }
        }
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
        // Currently unused
        _stream_size: usize,
        _cfg: &ScorerConfiguration,
        meta: &RouteData,
    ) -> Result<EpNeighbourPair> {
        // If there's no route available we return an error.  This case SHOULD
        // never occur but you never know
        if meta.link_id.len() == 0 {
            return Err(RatmanError::Nonfatal(NonfatalError::NoAvailableRoute));
        }

        if meta.link_id.len() == 1 {
            // We know there is at least one available link, but we need to check if
            // it's active.  If not we must fail-over into the next scorer
            let link = meta.link_id.get(0).unwrap();
            let link_data = meta.link_data.get(link).unwrap();
            return Ok(*link);
        } else {
            let nb_a = meta.link_id.get(0).unwrap();
            let nb_b = meta.link_id.get(1).unwrap();

            let link_a = meta.link_data.get(nb_a).unwrap();
            let link_b = meta.link_data.get(nb_b).unwrap();
        }
    }
}
