//! Module only loaded when Ratman is running as a daemon

mod parse;
mod transform;

pub(crate) fn elog<S: Into<String>>(msg: S, code: u16) -> ! {
    tracing::error!("{}", msg.into());
    std::process::exit(code.into());
}
