use crate::{context::RatmanContext, storage::route::RouteEntry};
use async_trait::async_trait;
use libratman::{frame::carrier::AnnounceFrame, Result};
use std::sync::Arc;

#[allow(unused)] // not part of the current development sprint
#[async_trait]
pub trait RouteScorer {
    async fn configure(&self, ctx: &Arc<RatmanContext>) -> Result<()>;
    async fn irq_live_announcement(&self, a: &AnnounceFrame) -> Result<()>;
    async fn compute(&self, msg_size: usize, meta: [&RouteEntry]) -> Result<()>;
}
