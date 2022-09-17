//! Endpoint abstraction module

use crate::{Frame, Result, Target};
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
    /// Return a maximum frame size in bytes
    ///
    /// Despite the function name, **this is not a hint** and your
    /// netmod driver should select the actual
    fn size_hint(&self) -> usize;

    /// Dispatch a `Frame` across this link
    ///
    /// Sending characteristics are entirely up to the implementation.
    /// As mentioned in the `size_hint()` documentation, this function
    /// **must not** panic on a `Frame` for size reasons, instead it
    /// should return `Error::FrameTooLarge`.
    ///
    /// The target ID is a way to instruct a netmod where to send a
    /// frame in a one-to-many mapping.  When implementing a
    /// one-to-one endpoint this ID can be ignored (set to 0).
    ///
    /// Optionally an exclusion target can be provided.  This is used
    /// to prevent endless replication of flood messages.  When
    /// implementing a one-to-one endpoint, the frame MUST be dropped
    /// when exclude contains any value!
    async fn send(&self, frame: Frame, target: Target, exclude: Option<u16>) -> Result<()>;

    /// Poll for the next available Frame from this interface
    ///
    /// It's recommended to return transmission errors, even if there
    /// are no ways to correct the situation from the router's POV,
    /// simply to feed packet drop metrics.
    async fn next(&self) -> Result<(Frame, Target)>;
}

#[async_trait]
impl<T: Endpoint + Send + Sync> Endpoint for Arc<T> {
    fn size_hint(&self) -> usize {
        T::size_hint(self)
    }

    async fn send(&self, frame: Frame, target: Target, exclude: Option<u16>) -> Result<()> {
        T::send(self, frame, target, exclude).await
    }

    async fn next(&self) -> Result<(Frame, Target)> {
        T::next(self).await
    }
}
