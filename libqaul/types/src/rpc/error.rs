
/// An error occured during RPC type conversions
pub enum ConvertError {
    /// An error occured while base decoding capnproto root
    BaseDecodeError(String),
    /// The conversion failed because some fields were missing
    MissingFields(Vec<String>),
}

impl From<capnp::Error> for ConvertError {
    fn from(e: capnp::Error) -> Self {
        Self::BaseDecodeError(e.to_string())
    }
}

pub(crate) fn try_read<T, E>(err: &mut Vec<String>, t: Result<T, E>, field: &str) -> Option<T> {
    match t.ok() {
        None => {
            err.push(field.into());
            None
        }
        some => some,
    }
}
