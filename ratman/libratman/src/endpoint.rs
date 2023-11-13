//! Endpoint abstraction module

use crate::{
    types::{CurrentStatus, EpTarget, InMemoryEnvelope},
    Result,
};
use async_trait::async_trait;
use std::sync::Arc;

/// The main trait describing a Ratman networking interface
///
/// All functions work without mutability because an endpoint is
/// expected to implement some access multiplexing or rely on atomic
/// operations to ensure thread safety.  This is because it's not
/// reasonable for an endpoint driver to rely purely on Rust's
/// ownership and mutability model, because it will inevitably have to
/// interact with system components, other buffers that push into a
/// queue, or similar.
///
/// This interface doesn't care about the implementation details of
/// these endpoints, and so, to make matters simpler for the router,
/// and to make it obvious that internal mutability needs to be used,
/// this interface is immutable by default.
#[async_trait]
pub trait Endpoint {
    /// Return a string identifier for this netmodk instance
    ///
    /// The identifier logic MUST handle multi-instancing, if it
    /// supports this.
    fn identifier(&self) -> String {
        "<unknown>".into()
    }

    /// Return a status digest for the current instance
    fn status(&self) -> CurrentStatus {
        CurrentStatus::Unknown
    }

    /// Start a peering session with a remote address
    ///
    /// The formatting of this address is specific to the netmod
    /// implementation, meaning that different netmods can rely on
    /// fundamentally different address schemas to establish their
    /// connections.  For example, the `inet` netmod simply uses IPv6
    /// socket addresses, while the `lora` netmod relies on
    /// cryptographic IDs of nearby gateways.
    ///
    /// The identifier returned must be a unique peer identifier,
    /// similar to the `EpTarget` abstraction that is used by `send`
    /// and `next`.  Currently this API doesn't consider stopping a
    /// peering intent (i.e. even if a connection drops, the netmod
    /// should always attempt to re-establish the connection).  The
    /// returned peer identifier can be used in the future to
    /// disconnect two routers from each other without having to
    /// restart all other connections.
    async fn start_peering(&self, _addr: &str) -> Result<u16> {
        unimplemented!()
    }

    /// Return a maximum frame size in bytes
    ///
    /// Despite the function name, *this is not a hint* and your
    /// netmod MAY reject frame envelopes that exceed the maximum
    /// transfer size of the respective link.
    ///
    /// If your communication channel supports external slicing
    /// mechanisms an implementation MAY also return 0, which will
    /// disable block-sub-slicing, meaning that full 1K or 32K payload
    /// frames will be dispatched directly.
    fn size_hint(&self) -> usize;

    /// Send a frame envelope to a target over this link
    ///
    /// Sending characteristics are entirely up to the implementation.
    /// As mentioned in the `size_hint()` documentation, this function
    /// **must not** panic on a frame envelope for size reasons,
    /// instead it should return `Error::FrameTooLarge`.
    ///
    /// The target ID is a way to instruct a netmod where to send a
    /// frame in a one-to-many mapping.  When implementing a
    /// one-to-one endpoint this ID can be ignored (set to 0).
    ///
    /// Optionally an exclusion target can be provided.  This is used
    /// to prevent sending flood messages back to their original
    /// recipients.  *When implementing a one-to-one endpoint*, the
    /// frame MUST be dropped when exclude contains any value!
    async fn send(
        &self,
        envelope: InMemoryEnvelope,
        target: EpTarget,
        exclude: Option<u16>,
    ) -> Result<()>;

    /// Poll for the next available Frame from this interface
    ///
    /// The errors returned by this netmod can currently not be
    /// returned to the sending application, but they can be logged
    /// for statistics purposes.
    ///
    /// **Note**: the implementation of this future MUST be
    /// cancellation safe!  Please consult the `tokio::select`
    /// documentation on what exactly that means for your Netmod
    /// implementation!  *Serious data loss* can occur if special care
    /// is not taken!
    async fn next(&self) -> Result<(InMemoryEnvelope, EpTarget)>;
}

#[async_trait]
impl<T: Endpoint + Send + Sync> Endpoint for Arc<T> {
    fn size_hint(&self) -> usize {
        T::size_hint(self)
    }

    async fn send(
        &self,
        envelope: InMemoryEnvelope,
        target: EpTarget,
        exclude: Option<u16>,
    ) -> Result<()> {
        T::send(self, envelope, target, exclude).await
    }

    async fn next(&self) -> Result<(InMemoryEnvelope, EpTarget)> {
        T::next(self).await
    }
}
