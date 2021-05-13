use crate::{Identity, IrdestSdk};

pub use irpc_sdk::{
    default_socket_path,
    error::{RpcError, RpcResult},
    io::{self, Message},
    RpcSocket, Service, SubSwitch, Subscription, ENCODING_JSON,
};
pub use std::{str, sync::Arc};

use ircore_types::rpc::{Capabilities, Reply, ServiceCapabilities, ServiceReply};
use ircore_types::users::UserAuth;

pub struct ServiceRpc<'ir> {
    pub(crate) rpc: &'ir IrdestSdk,
}
