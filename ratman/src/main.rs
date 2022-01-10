//! Ratman

#[macro_use]
extern crate tracing;

mod daemon;
pub(crate) use ratman::*;

#[async_std::main]
async fn main() {
    let r = Router::new();
    if let Err(e) = daemon::run(r, "0.0.0.0:9020".parse().unwrap()).await {
        error!("Ratmand suffered fatal error: {}", e);
    }
}
