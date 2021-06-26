use alexandria::{api::Library, error::Result};
use std::path::PathBuf;
use tracing_subscriber::{filter::LevelFilter, fmt, EnvFilter};

#[macro_use]
extern crate tracing;

pub(crate) fn parse_log_level() {
    let filter = EnvFilter::try_from_env("IRDEST_LOG")
        .unwrap_or_default()
        .add_directive(LevelFilter::TRACE.into())
        .add_directive("async_std=error".parse().unwrap())
        .add_directive("async_io=error".parse().unwrap())
        .add_directive("polling=error".parse().unwrap())
        .add_directive("mio=error".parse().unwrap());

    // Initialise the logger
    fmt::fmt().with_env_filter(filter).init();
}

#[async_std::main]
async fn main() -> Result<()> {
    parse_log_level();
    info!("Initialising alexandria database!");

    let path = PathBuf::new().join("test.ax");

    let lib = Library::create(path)?;

    // let lib = Builder::new().offset(path.as_path()).build();
    // lib.sync().await?;

    // let s = lib.sessions().create(Id::random(), "peterpan").await?;

    // lib.insert(
    //     s,
    //     "/:bar".into(),
    //     TagSet::empty(),
    //     Diff::map().insert("test key", "test value"),
    // )
    // .await?;

    // lib.ensure();

    Ok(())
}
