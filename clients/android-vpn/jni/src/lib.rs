#[cfg(target_os = "android")]
mod _impl;

#[cfg(target_os = "android")]
pub use _impl::*;
