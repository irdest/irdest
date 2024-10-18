use crate::{context::RatmanContext, procedures};
use libratman::{
    types::{Address, Ident32},
    Result,
};
use std::{sync::Arc, time::Duration};

pub struct AnycastProbeHandler {
    pub namespace_id: Ident32,
}

impl AnycastProbeHandler {
    pub async fn execute(self, ctx: Arc<RatmanContext>, timeout: Duration) -> Result<Vec<Address>> {
        procedures::flood_frame(&ctx.routes, &ctx.links, todo!(), None).await?;

        todo!()
    }
}
