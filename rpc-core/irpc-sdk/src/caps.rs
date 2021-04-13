/// Bitmask representing utf8 text json encoding
pub const ENCODING_JSON: u8 = 0x1;
/// BItmask representing binary message pack encoding
pub const ENCODING_MSGPACK: u8 = 0x3;

/// Express a set of capabilities between services and the broker
///
/// The only required field is `encoding` which indicates to the
/// broker which transport encodings can be accepted by the service.
/// It's recommended to use the `basic_json()` and `json()`
/// constructors.  It is however also possible to construct an
/// encoding bitfield manually.
///
/// ```rust
/// # use irpc_sdk::{Capabilities, ENCODING_JSON, ENCODING_MSGPACK};
/// Capabilities {
///     encoding: ENCODING_JSON | ENCODING_MSGPACK,
///     functions: vec![]
/// };
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Capabilities {
    /// Select which encoding to use
    pub encoding: u8,
    /// Set of supported functions
    pub functions: Vec<String>,
}

impl Capabilities {
    /// Create a basic json encoding capability set
    pub fn basic_json() -> Self {
        Self {
            encoding: ENCODING_JSON,
            functions: vec![],
        }
    }

    /// Create a capability set using the json encoding
    pub fn json<F: Into<Vec<String>>>(functions: F) -> Self {
        Self {
            encoding: ENCODING_JSON,
            functions: functions.into(),
        }
    }
}
