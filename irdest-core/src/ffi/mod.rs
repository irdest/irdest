//! Various foreign-function interface wrappers for irdest-core
//!
//! These interfaces enable you to use irdest-core from different
//! programming languages and build setups.  In most cases it's
//! recommended to use the `irpc` stack that enables
//! inter-process communication (via various protocols and channels),
//! but sometimes binding to irdest-core and various other libraries (such
//! as the services) directly for integration).
//!
//! For example, while the main UI might use the `irpc` +
//! `irdest-http` interface to manage call states in `irdest-voice`,
//! your app might bind directly to `irdest-voice` to save the overhead
//! of streaming voice data via http calls and to make integrations
//! easier.
//!
//! The following modules are disabled by default and can be enabled
//! via compile-time feature flags given to cargo.

#[cfg(feature = "ffi-java")]
pub mod java;
