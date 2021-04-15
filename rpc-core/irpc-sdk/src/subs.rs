use std::{collections::BTreeMap, marker::PhantomData};
use async_std::sync::{Sender, Receiver};

/// A generic subscription type
///
/// Use this type in your component SDK to make it possible for users
/// to get updates for a particular stream of data.  Use
/// [`SubscriptionCmd`](crate::proto::SubscriptionCmd) to encode the
/// subscription creation handshake.  A subscription object is then
/// generic over the type returned by the subscription stream.
///
/// Following is an overview of the subscription message flow.
///
/// ```
/// [ Your Service ]                     [ Remote Service ]
///     Subscriptioncmd::Register ----------->
///            <------------- SdkReply::Identity
///
///     ...
///
///            <------------- SubscriptionCmd::Push
///            <------------- SubscriptionCmd::Push
///
///     ...
///
///     Subscriptioncmd::Unregister --------->
///            <------------- SdkReply::Ok
/// ```
///
/// ## Creating subscriptions
///
/// ```rust
///
/// ```
pub struct Subscription<T> {
    _type: PhantomData<T>,
    rx: Receiver<Vec<u8>>,
}


/// Map between an RPC connection and subscription objects
#[derive(Default)]
pub struct SubSwitch {
    map: BTreeMap<Identity, Sender<Vec<u8>>>,
}


impl SubSwitch {
    /// Create a new map for RPC subscriptions
    pub fn new() -> Self {
        Self::default()
    }
}
