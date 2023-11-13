use libratman::netmod::{InMemoryEnvelope, Target};

use crate::core::GenericEndpoint;
use std::sync::Arc;

/// A full runtime system thread handling netmod traffic
///
/// This type can be initialised either for a single netmod, two, or
/// all of them.
pub struct NetmodRuntime {
    state: NetmodState,
}

enum NetmodState {
    Single(Arc<GenericEndpoint>),
    Binary(Arc<GenericEndpoint>, Arc<GenericEndpoint>),
    Complete(Vec<GenericEndpoint>),
}

impl NetmodState {
    pub async fn send_to(
        &self,
        nmodid: usize,
        target: Target,
        envelope: InMemoryEnvelope,
        exclude: Option<u16>,
    ) -> Result<()> {
        match self {
            Self::Single(ep) => ep.send(envelope, target, exclude).await,
            _ => todo!(),
        }
    }
}
