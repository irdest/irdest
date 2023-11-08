#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("invalid mode: (ns: {0}, op: {1})")]
    InvalidMode(u8, u8),
    #[error("failed to read a full microframe: timeout")]
    ReadTimeout,
    #[error("failed to read a valid cstring from input")]
    InvalidString,
    #[error("failed to parse type because of missing fields: {:?}", 0)]
    MissingFields(&'static [&'static str]),
}
