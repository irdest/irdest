use crate::{
    api::SubscriptionHandle,
    types::{AddrAuth, Address, Ident32, LetterheadV1, Modify, Recipient},
    Result,
};
use async_trait::async_trait;
use std::{collections::BTreeMap, net::SocketAddr, sync::Arc};
use tokio::{
    io::{AsyncRead, AsyncWrite},
    net::TcpStream,
    sync::MutexGuard,
};

use super::socket_v2::RawSocketHandle;

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
        space_private_key: Option<Ident32>,
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
    async fn recv_many<'s, I>(
        self: &'s Arc<Self>,
        auth: AddrAuth,
        addr: Address,
        to: Recipient,
        num: u32,
    ) -> Result<I>
    where
        I: Iterator<Item = (LetterheadV1, ReadStream<'s>)>;
}

pub struct ReadStream<'a>(pub(crate) MutexGuard<'a, RawSocketHandle>);

impl<'a> ReadStream<'a> {
    pub fn as_reader(&mut self) -> &mut impl AsyncRead {
        &mut *self.0.stream()
    }
}
