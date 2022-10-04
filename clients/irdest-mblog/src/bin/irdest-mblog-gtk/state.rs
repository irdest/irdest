use anyhow::Result;
use ratman_client::RatmanIpc;

/// Central app state type which handles connection to Ratman
pub struct AppState {
    ipc: RatmanIpc,
}

impl AppState {
    pub async fn new() -> Result<Self> {
        let addr = irdest_mblog::load_or_create_addr().await?;
        let ipc = RatmanIpc::default_with_addr(addr).await?;

        Ok(Self { ipc })
    }
}
