use crate::{
    api::SubscriptionHandle,
    types::{Address, AddrAuth, Ident32, Modify, Recipient},
    Result,
};
use async_trait::async_trait;
use std::{collections::BTreeMap, net::SocketAddr, sync::Arc};

#[async_trait]
pub trait RatmanIpcExtV1 {
    /// Create a new Ratman IPC interface given a
    async fn start(bind: SocketAddr) -> Result<Arc<Self>>;

    //
    // (@^_^@) Address commands
    //

    /// Create a new address for an existing client token
    ///
    /// Optionally you may give this address a name.  It won't be
    /// shared with any other network participant or client and purely
    /// serves as a human identifier.
    async fn addr_create(self: &Arc<Self>, name: Option<String>) -> Result<(Address, AddrAuth)>;

    /// Delete an address, optionally including all its linked data
    async fn addr_destroy(
        self: &Arc<Self>,
        auth: AddrAuth,
        addr: Address,
        force: bool,
    ) -> Result<()>;

    /// Mark a particular address as "up"
    async fn addr_up(self: &Arc<Self>, auth: AddrAuth, addr: Address) -> Result<()>;

    /// Mark a particular address as "down"
    async fn addr_down(self: &Arc<Self>, auth: AddrAuth, addr: Address) -> Result<()>;

    //
    // (@^_^@) Contact commands
    //

    /// Create a new contact entry for an address
    ///
    /// Each client has its own contact book.  Currently there's no
    /// way to share contacts between clients.
    async fn contact_add(
        self: &Arc<Self>,
        auth: AddrAuth,
        addr: Address,
        note: Option<String>,
        tags: BTreeMap<String, String>,
        trust: u8,
    ) -> Result<Ident32>;

    /// Apply a simple change across one or multiple contact entries
    async fn contact_modify(
        self: &Arc<Self>,
        auth: AddrAuth,

        // Selection filter section
        addr_filter: Vec<Address>,
        note_filter: Option<String>,
        tags_filter: BTreeMap<String, String>,

        // Modification section
        note_modify: Modify<String>,
        tags_modify: Modify<(String, String)>,
    ) -> Result<Vec<Ident32>>;

    /// Delete existing contact entries via filters
    async fn contact_delete(self: &Arc<Self>, auth: AddrAuth, addr: Address) -> Result<()>;

    //
    // (@^_^@) Subscription commands
    //

    /// Check which subscriptions are currently available on the router
    async fn subs_available(self: &Arc<Self>, auth: AddrAuth) -> Result<Vec<Ident32>>;

    /// Create a new subscription for a specific Recipient type
    ///
    /// The `subscription_recipient` can either be a single address
    /// (one of the client's registered addresses), or a flood
    /// namespace.  If subscribing to a namespace you MUST
    /// additionally add the associated namespace key!  See [todo] for
    /// details!
    ///
    /// When re-creating a subscription (for example after the client shuts
    /// down) it will be reused by the router and a new handle is constructed.
    ///
    /// To explicitly stop a subscription from the router call `unsubscribe`
    /// instead!
    // A subscription can optionally be synced, meaning that no messages are
    // accepted for the subscription while the client is offline (although no
    // guarantees are made about other clients -- relevant messages MAY still be
    // able to be queried via the journal if another client has added them).
    //
    // Subscriptions can also be auto-deleting, if a `timeout` Duration is
    // provided.
    async fn subs_create(
        self: &Arc<Self>,
        auth: AddrAuth,
        subscription_recipient: Recipient,
        // synced: bool,
        // timeout: Option<Duration>,
    ) -> Result<SubscriptionHandle>;

    /// Restore a previously created subscription
    async fn subs_restore(
        self: &Arc<Self>,
        auth: AddrAuth,
        sub_id: Ident32,
    ) -> Result<SubscriptionHandle>;

    /// Delete a subscription, invalidating any previous subscription handles
    async fn subs_delete(self: &Arc<Self>, auth: AddrAuth, subsciption_id: Ident32) -> Result<()>;
}
