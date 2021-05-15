//! I/O persistence module

pub(crate) mod format;

mod sync;
pub use sync::Sync;

mod cfg;
pub use cfg::Config;
pub use cfg::legacy as versions;
