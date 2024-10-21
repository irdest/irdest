use crate::{
    api::{socket_v2::RawSocketHandle, SubscriptionHandle},
    types::{error::UserError, AddrAuth, Address, Ident32, LetterheadV1, Namespace, Recipient},
    ClientError, Result,
};
use async_trait::async_trait;
use std::{net::SocketAddr, sync::Arc, time::Duration};
use tokio::{io::AsyncRead, sync::MutexGuard};

use super::types::{PeerEntry, RouterStatus, ServerPing};

#[async_trait]
pub trait RatmanIpcExtV1 {
    /// Create a new Ratman IPC interface given a
    async fn start(bind: SocketAddr) -> Result<Arc<Self>>;

    //
    // (@^_^@) Address commands
    //

    /// List available local addresses
    async fn addr_list(self: &Arc<Self>) -> Result<Vec<Address>>;

    /// Create a new address for an existing client token
    ///
    /// Optionally you may give this address a name.  It won't be
    /// shared with any other network participant or client and purely
    /// serves as a human identifier.
    ///
    /// A WEIRD QUIRK in this API: to limit scope I didn't implement a namespace
    /// management API yet, which means listening to messages addressed to a
    /// namespace becomes very hard.  To deal with this and to make namespaces
    /// (for subscriptions!) persistent between restarts you can optionally
    /// supply the namespace private key material.  It's suggested to include a
    /// copy of this key in your application's source code rather than
    /// configuration.
    async fn addr_create<'n>(
        self: &Arc<Self>,
        name: Option<&'n String>,
    ) -> Result<(Address, AddrAuth)>;

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
    // (@^_^@) Peers commands
    //

    async fn peers_list(self: &Arc<Self>) -> Result<Vec<PeerEntry>>;

    //
    // (@^_^@) Status commands
    //

    async fn router_status(self: &Arc<Self>) -> Result<RouterStatus>;

    //
    // (@^_^@) Contact commands
    //

    // /// Create a new contact entry for an address
    // ///
    // /// Each client has its own contact book.  Currently there's no
    // /// way to share contacts between clients.
    // async fn contact_add(
    //     self: &Arc<Self>,
    //     auth: AddrAuth,
    //     addr: Address,
    //     note: Option<String>,
    //     tags: BTreeMap<String, String>,
    //     trust: u8,
    // ) -> Result<Ident32>;

    // /// Apply a simple change across one or multiple contact entries
    // async fn contact_modify(
    //     self: &Arc<Self>,
    //     auth: AddrAuth,

    //     // Selection filter section
    //     addr_filter: Vec<Address>,
    //     note_filter: Option<String>,
    //     tags_filter: BTreeMap<String, String>,

    //     // Modification section
    //     note_modify: Modify<String>,
    //     tags_modify: Modify<(String, String)>,
    // ) -> Result<Vec<Ident32>>;

    // /// Delete existing contact entries via filters
    // async fn contact_delete(self: &Arc<Self>, auth: AddrAuth, addr: Address) -> Result<()>;

    //
    // (@^_^@) Subscription commands
    //

    /// Check which subscriptions are currently available on the router
    async fn subs_available(
        self: &Arc<Self>,
        auth: AddrAuth,
        addr: Address,
    ) -> Result<Vec<Ident32>>;

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
        addr: Address,
        recipient: Recipient,
    ) -> Result<SubscriptionHandle>;

    /// Restore a previously created subscription
    async fn subs_restore(
        self: &Arc<Self>,
        auth: AddrAuth,
        addr: Address,
        sub_id: Ident32,
    ) -> Result<SubscriptionHandle>;

    /// Delete a subscription, invalidating any previous subscription handles
    async fn subs_delete(
        self: &Arc<Self>,
        auth: AddrAuth,
        addr: Address,
        subsciption_id: Ident32,
    ) -> Result<()>;
}

#[async_trait]
pub trait RatmanStreamExtV1: RatmanIpcExtV1 {
    /// Send a message stream to a single address on the network
    ///
    /// A send action needs a valid authentication token for the address that it
    /// is being sent from.  The letterhead contains metadata about the stream:
    /// what address is sending where, and how much.
    ///
    /// Optionally you can call `.add_send_time()` on the letterhead before
    /// passing it to this function to include the current time in the stream
    /// for the receiving client.
    async fn send_to<I: AsyncRead + Unpin + Send>(
        self: &Arc<Self>,
        auth: AddrAuth,
        letterhead: LetterheadV1,
        data_reader: I,
    ) -> Result<()>;

    /// Send the same message stream to multiple recipients
    ///
    /// Most of the Letterhead
    async fn send_many<I: AsyncRead + Unpin + Send>(
        self: &Arc<Self>,
        auth: AddrAuth,
        letterheads: Vec<LetterheadV1>,
        data_reader: I,
    ) -> Result<()>;

    /// Block this task/ socket to wait for a single incoming message stream
    ///
    /// This function returns a single stream letterhead (which indicates the
    /// sender, receiver, and metadata such as stream length) and an async
    /// reader, which can then be used to read the stream to some buffer or
    /// writer stream.
    ///
    /// Reading more bytes than `letterhead.payload_length` indicates is
    /// undefined behaviour!  You **must** drop the `ReadStream` after you're
    /// done reading to make the socket available to other API exchanges again!
    ///
    /// If you need to receive data more consistently consider setting up a
    /// subscription!
    async fn recv_one<'s>(
        self: &'s Arc<Self>,
        auth: AddrAuth,
        addr: Address,
        to: Recipient,
    ) -> Result<(LetterheadV1, ReadStream<'s>)>;

    /// Return an iterator over a stream of letterheads and read streams
    ///
    /// This function returns an iterator over incoming letterheads and read
    /// handles, which MUST be dropped at the end of your iterator closure to
    /// avoid deadlocking the next receive.  Reading more bytes than
    /// `letterhead.payload_length` is undefined behaviour!
    ///
    /// If you need to receive data more consistently consider setting up a
    /// subscription!
    async fn recv_many<'s>(
        self: &'s Arc<Self>,
        auth: AddrAuth,
        addr: Address,
        to: Recipient,
        num: Option<u32>,
    ) -> Result<StreamGenerator<'s>>;
}

pub struct ReadStream<'a>(pub(crate) MutexGuard<'a, RawSocketHandle>);

impl<'a> ReadStream<'a> {
    pub fn as_reader(&mut self) -> &mut impl AsyncRead {
        &mut *self.0.stream()
    }

    pub async fn drop(mut self) -> Result<()> {
        let (_, ping) = self.0.read_microframe::<ServerPing>().await?;
        match ping? {
            ServerPing::Ok => Ok(()),
            ServerPing::Error(e) => Err(e.into()),
            ping => Err(ClientError::Internal(format!("{ping:?}")).into()),
        }
    }
}

pub struct StreamGenerator<'a> {
    pub(crate) limit: Option<u32>,
    pub read: u32,
    pub inner: ReadStream<'a>,
}

impl<'a> StreamGenerator<'a> {
    pub async fn wait_for_manifest(&mut self) -> Result<LetterheadV1> {
        match self.limit {
            Some(limit) if limit > self.read => {
                let (_, lh) = self.inner.0.read_microframe::<LetterheadV1>().await?;
                self.read += 1;
                lh
            }
            None => {
                let (_, lh) = self.inner.0.read_microframe::<LetterheadV1>().await?;
                self.read += 1;
                lh
            }
            Some(_) => Err(UserError::RecvLimitReached.into()),
        }
    }
}

#[async_trait]
pub trait NamespaceAnycastExtV1: RatmanIpcExtV1 {
    /// Register a new namespace with the router
    ///
    /// To create a space key, you can either use the `ratctl` CLI or call the
    /// function `generate_space_key()` and store its output in your application
    /// source code.
    ///
    /// The private key must be included in every instance of your application
    /// to allow for transport layer space signatures and encryption.
    async fn namespace_register(
        self: &Arc<Self>,
        auth: AddrAuth,
        space_pubkey: Address,
        space_privkey: Ident32,
    ) -> Result<()>;

    /// List all locally available namespaces that have previously been
    /// registered on this system
    async fn namespace_list(self: &Arc<Self>) -> Result<Vec<Namespace>>;

    /// Destroy all local data associated with a namespace
    ///
    /// Note: this does not stop another client from re-registering this
    /// namespace, however deleted data can not be recovered.
    async fn namespace_destroy(self: &Arc<Self>, pubkey: Namespace, privkey: Ident32)
        -> Result<()>;

    /// Mark a given namespace as "up" for a given application
    ///
    /// This is different from a stream subscription, which listens to messages
    /// sent to a given namespace.  This function should be used by an
    /// application using a space for responding to various protocols that are
    /// targeted at this namespace.  It also enables the router to pre-cache
    /// messages sent to the namespace, even if no active message subscription
    /// exists
    ///
    /// Namespace subscriptions can be maintained independent of whether the
    /// namespace is up or down.
    async fn namespace_up(
        self: &Arc<Self>,
        client_address: Address,
        auth: AddrAuth,
        space_pubkey: Address,
    ) -> Result<()>;

    /// Mark a given namespace as "down" for a given application
    ///
    /// After this operation the router will no longer respond to anycast probes
    /// and other namespace protocols, as well as no longer cache incoming
    /// messages addressed to the namespace.  Namespace subscriptions can be
    /// maintained independent of whether the namespace is up or down.
    async fn namespace_down(
        self: &Arc<Self>,
        client_address: Address,
        auth: AddrAuth,
        space_pubkey: Address,
    ) -> Result<()>;

    /// Perform an anycast probe for a given namespace
    ///
    /// The anycast ping is sent out across the whole network and any client
    /// instance subscribed to this namespace will reply.  Any address which
    /// responds within the timeout is returned by this function, ordered by
    /// lowest to highest ping times.
    async fn namespace_anycast_probe(
        self: &Arc<Self>,
        client_address: Address,
        auth: AddrAuth,
        space_pubkey: Address,
        timeout: Duration,
    ) -> Result<Vec<(Address, Duration)>>;
}
