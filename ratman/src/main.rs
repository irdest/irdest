//! Ratman

#[macro_use]
extern crate tracing;

mod daemon;
pub(crate) use ratman::*;

#[async_std::main]
async fn main() {
    let _r = Router::new();
}
