# Mesh Router Exchange Protocol (MREP)

Status: *pre-draft*
First revision: *2022-04-05*
Latest revision: *2024-09-18*

The Irdest router Ratman exchanges user packets as well as routing metadata with neighbouring routers.  This communication is facilitated through the "mesh router exchange protocol".  It has three scopes.

1. Exchange user data packets
2. Collect connection metrics and transmission metadata
3. Perform routing decisions according to these metrics

The key words "_MUST_", "_MUST NOT_", "_REQUIRED_", "_SHALL_", "_SHALL NOT_", "_SHOULD_", "_SHOULD NOT_", "_RECOMMENDED_", "_MAY_", and "_OPTIONAL_" in this document are to be interpreted as described in [RFC2119]

## Basics

The Mesh Router Exchange Protocol specifies different mechanisms for multiple routers to communicate with each other in order to facilitate the flow of messages across a shared network.

Routers are connected with each other via different communication channels (or backplanes), meaning that the routing logic is decoupled from the connection logic.  This specification does not make assumptions on the API between a router and the connection logic, but does place requirements and limitations on the information exchange that a valid implementation MUST provide between the two component layers.

Simultaneously an Irdest router MUST allow connections from client applications, which can register addresses on the network.  A client application can register as many addresses as it needs or wants.  From a network perspective the relationship between an address and a device can't necessarily be proven.

The terms "Address" and "Id" may be used interchangeably in this specification draft.


### A word on encoding

This specification uses Rust-like pseudo-code to express datastructures.  For over-the-network communication MREP uses a low-overhead encoding format called "encoding frames".

The "encoding frames" format supports the following data types.  Specific message types are documented in [Appendix A]:

- `u8`, `u16`, `u32`, and `u64`, encoded as 1, 2, 4, or 8 big-endian bytes
- `bool`, encoded as 1 byte which is either `0`, or `1`
- `Option<T>`, encoded as a zero-byte
- `[u8]`, encoded as a 2-byte big-endian length indicator, followed by the raw data buffer
- `CString`, encoded as a c-style string and zero-byte terminator
- `Ident32`, is a short-hand for a fixed-size 32-byte buffer which can contain either an address or content ID, and is not length-prepended

Encoding frames are not self-documenting and instead message types MUST use a versioning byte which is then used to switch between older and newer type implementations.


### The anatomy of a message

When sending a piece of data this is called a "stream", based on the fact that the data is _streamed_ from a connected client to its local router, which encodes the content into [eris] blocks, which are streamed to the local journal.

From that point routes are selected and the blocks to send are sliced into "frames".  A frame is analogous to a single network packet.  A frame contains metadata such as the sender and recipients, signatures, and ordering information that allows each block to be re-assembled on the receiving end.

Intermediary routers can see which frames are associated with a block, but not which blocks make up a full stream.  Because the eris encoding creates a tree of blocks, only the root reference (+ additional metadata) are required to re-assemble the stream.  This message type is called a "manifest", which MUST be additionally encrypted to avoid eavesdropping by intermediary parties.

The transport encryption secret is calculated via a diffie-hellman key exchange between the private sender key and the public address key of the recipient (either a single address, or a namespace).


## Address announcements

An address in an Irdest network is a 32-byte ed25519 public key, backed by a corresponding private key, which is not shared outside of the router the address belongs to.  The private key MUST be encrypted with some kind of user-facing secret.

For new addresses to spread across the network Irdest uses a gossip announcement approach.  Announcements are re-broadcast periodically (currently every 2 seconds) and have a cryptographically signed timestamp, which MUST be verified by any receiving router.  This way the authenticity of an announced route can be guaranteed.  Announcements with a timestamp outside a particular window of validity MAY be dropped, although this specification does not currently indicate when this should be the case.

Announcements MUST NOT be broadcast to the sender channel of the announcement to avoid infinite replication and an announcement ID that has been seen before by a router MUST be dropped.


## Router announcements

Every router on the Irdest network also has a unique address which is not shared with client applications.  Messages sent to this address MUST conform to the MREP Message specification ([Appendix A]).

Routers announce their address to other routers they are immediately connected to.  However unlike regular Irdest addresses, these announcements SHOULD NOT be propagated, unless explicitly instructed to do so by the sending router.  However this specification does not currently indicate when this would be the case.

Router announcements are re-broadcast periodically (currently every 30 seconds) to all immediately connected routers, via every connection channel.  The receiving router of such an announcement MUST keep track of the connection channel and specific "target ID" of the connection in memory, but SHOULD NOT persist any of the announcement data, unless explicitly indicated by the protocol.  Currently this specification does not indicate when this would be the case.


## Namespaces

A special kind of address exists called a "namespace".  While a regular address uses an internal private key, a namespace uses a private key provided by a client application.  This allows multiple applications to share the same encryption and verification key for a given namespace to share information amongst different instances of itself across the network.


## Route selection / scoring

Because Irdest is a mesh network the selection of a route for any given frame is done by every router that handles it along the way.  This is also due to the fact that no one network participant can have a full picture of the network topology and is thus dependent on peers to forward frames to whichever of their neighbours is best suited to deliver a particular frame.

A neighbour in Irdest is a device that is connected to the current router via some communication channel (netmod).  The terms "netmod", "connection", and "neighbour" are used interchangably.

Irdest uses two different route selection (or "route scoring") mechanisms.


### Live route scoring

When a live connection exists this scorer is used.  A connection is considered "live" when the router has received an address announcement from the recipient address from a given neighbour in the last 30 seconds.

Because announcements are re-broadcast every 2 seconds this gives lenience to "network wobbles" and temporary connection drop-outs.  When announcements stop being received from a neighbour it is marked as `Idle` and skipped when doing route selection via the live scorer.

The live scorer uses both ping latency (calculated based on the signed timestamp in an announcement) and a measured available bandwidth approximation of a given connection.

When a router only has a single link available to reach a peer, this link MUST be used.  When there are multiple routes to a target the links are sorted by their ping times and the lowest ping link MUST be used.

When two ping times are within 10% of each other (i.e. 10ms vs 11ms) the available bandwidth of a link is used as a tie-breaker.  When the bandwidth for each link is _also_ within a 2% window of each other, or one of the links has failed to measure a bandwidth (the announcement didn't contain it, or it was set to `0` for other reasons) only the ping time SHOULD be used to determine the route.

Because ping times are measured end-to-end the overall ping time to a target decreases as a frame gets closer to its destination.  Routing loops can be avoided, because even when a secondary route exists that may provide more bandwidth it is not advantageous for a router to "send back" a frame as it would increase the ping time.

Links that a frame was received on MUST be excluded from route selection!


### Store & Forward scoring

When no live link to a target address exists (i.e. not announcement has been received within the last 10 seconds) the routing behaviour of the live scorer is inverted:

If only one link towards a target address exists it MUST be used.  If multiple links exists, they are sorted by available bandwidth and the link with the highest bandwidth MUST be used.  When two bandwidth values are within a 10% window of each other the ping time to a target MAY be used.

When doing a store & forward routing strategy an implementation MAY rely on the fact that trusted addresses may spend more time in closer proximity to each other, meaning that a trust score can be used to rank them (see Bubble Rap routing in the bibliography).  This mechanism is currently not implemented.


## Security

<small>(I am not a cryptographer and this section will have to be
expanded/ reviewed in the future)</small>

All user messages sent through Irdest are encrypted via the
[ChaCha20](https://en.wikipedia.org/wiki/Salsa20#ChaCha_variant)
stream cipher, provided by the [ERIS] block slicing specification
which is used to encode user payloads.

An address in Irdest is an public key (also called "address key"),
backed by a corresponding secret key.  Keys are generated on the
[ed25519] twisted edwards curve.

Because ChaCha20 is a *stream cipher* it requires a symmetric secret
to work.  For this purpose we convert both secret and public keys from
the edwards curve representation to the montgomery curve
representation, and use this for the *x25519 diffie-hellman* handshake
between the sending address secret key and the recipient address key.

There are two layers of signatures in Irdest.  The base layer Frame
(defined in [Appendix A]) contains space for an ed25519 signature.
All message payloads are signed by the sending address edward's curve
key (currently using `ed25519-dalek`) and can be verified by any node
a message traverses (since the sending address is visible to any
network participant).

For user payloads, ERIS guarantees message integrity by verifying
block content hashes against the recorded versions in the manifest.
This manifest message is also signed via the basic Frame delivery
mechanism.  Thus user message integrity can be guaranteed.


## Message encoding & delivery

User payloads are encoded via [ERIS], sliced into carrier frames, and
sent towards their destination (see [Appendix A] on details).

This uses two basic message types: `DataFrame` and `ManifestFrame`.
Messages in Irdest are sessionless streams, meaning that data is
streamed between different Irdest routers, but buffered into complete
messages before being exposed to the recipient application.

ERIS specifies a *"Read Capability"* which for the purposes of Irdest
and this spec we are calling the *"Manifest"*.

For a `DataFrame` the payload of the underlying carrier frame is
entirely filled with content from a re-aligned block stream.  Frames
MUST NOT be padded with trailing zeros to fill the target MTU.

A `ManifestFrame` contains a binary encoded version of the "Read
Capability".  If this manifest is too large for the containing
carrier frame, it is split into multiple frames (see [Appendix A:
Manifest Frame](#Manifest-Frame))


## Journal sync

Irdest allows devices to connect to each other via short-lived (or
"ephemeral") connections.  One such application is Android phones,
where p2p WiFi connections can only be established with a single other
party at a time.  Bluetooth mesh groups are possible, but are also
significantly limited in the number of active connections.

For this purpose we introduce the "journal sync" mechanism.

An Irdest router MUST contain a journal of content-addressed blocks of
data (see [Appendix B](#Appendix-B-message-routing)).  Messages are
indexed via their content hashes, as well as the recipient
information.  A journal sync is a uni-directional operation, which
should be applied in both directions of the link.  What that means is
that journals are not so much synced, but propagated.

Let's look at an example to demonstrate the process.

Routers A and B are connected to each other via an ephemeral
connection (`req_ephemeral_connection` is called by a netmod driver
which has established the connection).

First both routers exchange a list of known addresses.  Future
versions of this specification MAY implement some kind of compression
or optimisation for this transfer, since routing tables may get quite
large.

```rust
SyncScopeRequest {
  addrs: BTreeSet<Address>,
}
```

*Outline:*

- Exchange list of known addresses (with an optimisation for "last
  recently used")
- Forward blocks addressed to any of the known addresses
- How to avoid re-transmit loops in a group of phones?
- How to avoid having to send too much data?
- Loops between people who both infrequently see the same peer
  address?  Who gets the frames? Both? (probably)


## AGPL compliance

Ratman is licensed under the AGPL-3.0 license and as such needs to be
able to provide its own source code.

It is not possible to query the source of a node more than *one router
edge* away from your own since router address announcements do not
propagate across the network.

A router MAY at any time send a source request to a connected router.
The request is time-stamped to avoid repeated and duplicate requests.

```rust
SourceRequest {
  date: "2022-09-22 03:18:32"
}
```

As a response, the recipient router MUST send a `SourceResponse`
reply.  The response doesn't contain the source code.  Instead it
describes the source that is running.  A `SourceResponse` MUST contain
the `source_urn` field.  Every other field is optional, but a router
SHOULD still provide them.  The `note` field SHOULD contain a list of
patch-names that have been applied to the node, if the
`number_of_patches` is not zero.  Otherwise this field SHOULD remain
empty.

```rust
SourceResponse {
  version: "0.5",
  number_of_patches: 0,
  source_url: "https://git.irde.st/we/irdest",
  source_urn: "b5bb9d8014a0f9b1d61e21e796d78dccdf1352f23cd32812f4850b878ae4944c",
  note: "Hier könnte Ihre Werbung stehen",
}
```

A recipient of this `SourceResponse` can now check whether the source
code their node is running is the same as the router that responded,
by checking the `source_urn` against their own source version (TODO:
specify how this URN is generated).

In case the recipient doesn't already have this source code they can
now send a `PullRequest` to the sending node:

```rust
PullRequest {
  urn: "b5bb9d8014a0f9b1d61e21e796d78dccdf1352f23cd32812f4850b878ae4944c",
}
```

## Low bandwidth modes

Some links in an Irdest network may be extremely low bandwidth, for
example when using `netmod-lora` for long range communication.  This
severely constricts the maximum transfer size (< 255 bytes), on a < 1%
duty cycle.  This means that the *maximum incoming message size* MUST
be constricted as well.

In these cases the "Small Message Optimisation" (SMO) MUST be used.
Following is a table that outlines the selection of encoding block
sizes based on the determined path MTU and size-hint (via
`announcement.route.mtu` and `announcement.route.size-hint`)

For these cases we should have small-message optimisations, based on
the size of the message, and the path MTU to the recipient.

| Message size | Path MTU    | Selected block size |
|--------------|-------------|---------------------|
| < 256 bytes  | -           | 64 bytes            |
| < 1 kB       | -           | 256 bytes           |
| < 32 kB      | < 1 kB      | 256 bytes           |
| < 2 kB       | < 256 bytes | 64 bytes            |
| > 1kB < 28kB | -           | 1 kB                |
| -            | -           | 32 kB               |

Messages **larger than 32 kB/ 2 kB on a path MTU of <1 kB/ <256 bytes
respectively** should be rejected by the sending router. We may want
to add another small message optimisation between 2kB and 32kB max
size messages.


## MTU leap-frogging

A frame may encounter a netmod link that doesn't allow for a
sufficiently sized MTU

In some cases, the path MTU information on the sending node was
incorrect, and a set of frames will encounter a link that is too
low-bandwidth to support their size.  In this case the "leap-frogging"
protocol should be used.

The first frame in a series that is too large to transmit over a
connection will be prepended with this metadata section:

```rust
LinkLeapRequest {
  seq_id: "1D90-C2AB-E50D-A4EC-F88C-BD9E-818B-7006-7D32-BED0-4EEC-83F0-756E-D856-40AA-B611",
  inc_mtu: 1222,
  look_ahead: false,
}
```

- `seq_id` :: the sequence ID for the incoming set of frames.  This
  identifier is used to determine which frames need to be re-sliced
- `inc_mtu` is the size of incoming frames

MTU leap-frogging performs a single step of look-ahead.  This means
that a router receiving a `LinkLeapRequest` MUST perform an MTU
look-ahead if `request.look_ahead` is set to `true` (and subsequently
set it to `false`).  This means that up to two link MTU limitations
can be "skipped over" before having to re-collect into the original
frame size and re-slicing.

For an incoming `LinkLeapRequest` a router MUST spawn a
`LeapFrameCollector`



## Appendix A: MREP Message specification

This section of the specification outlines the way that MREP container messages are encoded.  As per the "encoding frames" rules any optional field that is not present MUST be replaced with a zero byte.

The basic container type of any message in an Irdest network is a carrier frame, which consists of a header and optional payload.  The header has the following structure:

```rust
CarrierFrameHeader {
  version: u8,
  modes: u16,
  recipient: [u8; 32] (optional),
  sender: [u8; 32],
  seq_id: [u8; 34] (optional),
  signature: [u8; 32] (optional),
  payload_size: u16,
}
```

This message structure is **byte aligned**.

- `version` :: indicate which version of the carrier frame format should be parsed.  Currently only the value `0x1` is supported
- `modes` :: a bitfield that specifies what type of content is encoded into the payload
- `recipient` :: (Optional) recipient address key.  May be replaced with a single zero byte if the frame is not addressed (see below).
- `sender` :: mandatory sender address key
- `seq_id` :: (Optional) sequence ID for push messaging payloads, mtu-leap protocol, etc
- `signature` :: (Optional) payload signature, generated by the sending key.  May be replaced with a single zero byte if the frame has a payload-internal signature (see below).
- `payload_size` :: 16 bit unsigned integer indicating the size of the data section.  Frame payloads larger than 32kiB are not supported!

Importantly, the `CarrierFrame` does not include a transmission checksum to detect transport errors.  This is because some transport channels have a built-in checksum mechanism, and thus the effort would be duplicated.  It is up to any netmod to decide whether a transmission checksum is required.

Following is a (*work in progress!*) overview of valid bitfields.  If a field is _not_ listed it is _invalid_!  Routers that encounter an invalid message MUST discard it.


| Bitfield states | Frame type descriptor                    |
|-----------------|------------------------------------------|
| `000 01xx`      | Base address announcements               |
| `0000 1000`     | ERIS Data frame                          |
| `0000 1001`     | ERIS Manifest frame                      |
| `0000 1xxx`     | *(Reserved for future data frame types)* |
| `0001 xxxx`     | *(Reserved)*                             |
| `001x xxxx`     | Netmod/ Wire peering frames              |
| `01xx xxxx`     | Router to Router peering frames          |
| `1000 0001`     | Anycast Probe Request                    |
| `???? ????`     | SyncScopeRequest                         |
| `???? ????`     | SourceRequest                            |
| `???? ????`     | SourceResponse                           |
| `???? ????`     | PushNotice                               |
| `???? ????`     | DenyNotice                               |
| `???? ????`     | PullRequest                              |
| `???? ????`     | LinkLeapNotice                           |
| `xxxx xxxx`     | User specified packet type range         |


### Announcement

`Announcement` frames are special in that they MUST set the `recipient` and `signature` field to a single zero byte.  This is because announcements are not addressed, and contain a payload-internal signature system.  All other message types handled by this specification MUST include both a recipient and signature!

The announcement payload consists of multiple parts:

```rust
OriginData {
  timestamp: CString
}
```

The origin data is signed by the announcement sender and MUST not be modified.  Any announcement with an invalid origin data signature MUST be discarded.

```rust
PeerData { }
```

Peer data is refreshed at every hop and signed with the corresponding router address key.  This field is currently left blank for future expansion.

```rust
RouteData {
  available_bw: u32,
  available_mtu: u32,
}
```

The route data is conditionally modified on every hop and corresponds to the _fully traced_ route from an announced address to an arbitrary recipient on the network.  Both the "bandwidth" and "maximum transfer unit" fields MUST ONLY be updated when the measured value from a particular receiving link is _lower_ than the value that is already included in the route data.

For example, if the given announcement was received over a link that has a measured bandwidth of 32768 B/s (32KB/s) a router MUST update the field before re-broadcasting it to other peers.  For this reason the route data section SHOULD NOT be signed.

```rust
RouteData {
  available_bw: 65536
  available_mtu: 1300,
}
```


### Data frame

A data frame is already explicitly sliced to fit into a carrier frame
(see "MTU leap-frogging" for how to handle exceptions to this).
Therefore the payload content can simply be encoded as a set of bytes.

The carrier frame knows the size of the payload.  Thus no special
encoding for data frames is required.


### Manifest frame

Message manifests SHOULD generally fit into a single carrier frame.
This may not be the case on low-bandwidth connections.  Because the manifest has no well-defined representation in the [ERIS] spec, we need to wrap it in our own encoding schema.

Additionally the manifest contains some important metadata about the stream that is about to be received, including the from and to address, the full stream size, and any auxiliary metadata, which can be filled in by the sending application.

The manifest SHOULD be encrypted with a shared symmetric secret based on the sending and receiving address keys.  When receiving a manifest for an address that is not active, it SHOULD be stored until the address key for decryption becomes available.

```rust
struct Manifest {
    letterhead: Letterhead {
        from: Address,
        to: Recipient,
        stream_size: u64,
        auxiliary_data: Vec<(CString, CString)>,
    },
    block_size: u8,
    block_level: u8,
    root_reference: Ident32,
    root_key: Ident32,
}
```


### Netmod peering range

When establishing a peering relationship between two routers their
respective netmods MAY have to negotiate some state between them.  In
most cases this protocol should be simple.  In any case, the bitflag
range `0000 0000 001x xxxx` is reserved for such purposes (in decimal
numbers `32` to `63`).  CarrierFrame's in this modes range MUST not be
passed to the router.  Instead their payloads SHOULD be parsed by the
receiving netmod to influence peering decisions.

It is left up to the netmod implementation to specify how this range
is used.  Netmods that wish to interact with each other SHOULD
coordinate usage of the same frame type flags.


### Router peering range

Similar to the netmod peering protocol range, routers have the ability
to exchange data with their immediate peers about who they are, where
they can route, and any other information that may impact neighbour
routing decisions.  The bitblag range `0000 0000 01xx xxxx` is
reserved for such purposes (in decimal numbers `64` to `127`).
CarrierFrame's in this range MUST NOT be cached in the routing
journal, or forwarded to any other peer.

Currently the only specified message structure is the `RouterMeta` which is used for routers to exchange basic information about their state with their immediate neighbours.  Received router protocol messages MUST NOT be forwarded to other peers.

```rust
struct RouterMeta {
    key_id: Ident32,
    available_buffer: Option<u64>,
    known_peers: u32
}
```

## Anycast Probe Request

Namespaces allow applications to listen to the same address key across a network, allowing different instances to "find" each other.  The MREP protocol doesn't support real anycast, but can pre-compute route preferences across a namespace.

An anycast probe is addressed to a namespace address and contains no payload.

```rust
CarrierframeHeaderV1 {
  modes: 0b1000_0000,
  sender: [application address],
  recipient: Some([namespace address]),
  seq_id: None,
  auxiliary_data: None,
  signature_data: None,
  payload_length: 0,
}
```

In order for a router to respond to an anycast probe request it MUST have marked the given namespace as "up".  If this is the case a router switch MUST respond to an anycast probe request.

Responses MUST include a timestamp in the auxiliary_data section to differentiate a probe request and response in a router switch.  A router MAY collect anycast responses that arrive after the listed timeout for analytics or routing purposes, but MUST NOT be included in the API response to the requesting application.

```rust
CarrierFrameHeaderV1 {
  modes: 0x1000_000,
  sender: [application address],
  recipient: Some([anycast probe initiator address]),
  seq_id: None,
  auxiliary_data: Some([encoded UTC timestamp data]),
  signature_data: None,
  payload_length: 0,
}
```


### SyncScopeRequest

### SourceRequest

### SourceResponse

### PushNotice

### DenyNotice

### PullRequest

### LinkLeapNotice

## Appendix B: message routing

Ratman has two message sending capabilities: Push and Pull.

- **Push routing** is used by default when an active connection to a
  peer is present.
- **Pull routing** is used whenever there is no active connection, or
  for particularly large or static payloads.


### Push routing

This is the default routing mode.  It is used whenever an active
connection is present, or if the sending application didn't provide
any additional instructions.

A message stream is encoded into [ERIS] blocks which are encoded,
encrypted, and content addressed.  Each block is saved in the Router
journal.  Lastly a message manifest is generated and signed by the
sending key.  Blocks are sent to the recipient as they are generated,
avoiding having to save the entire message in memory.

Lastly a message manifest is generated which contains the content
hashes of each previous block.  This manifest is signed by the sending
key and also sent to the recipient.

On the receiving side blocks are kept in the journal until the
manifest is received, then the message can be verified and decoded for
the receiving application.

For messages larger than `N` MB (tbd), a sending router MUST generate
a `PushNotice` before the final message manifest has been generated.

```rust
struct PushNotice {
  sender: <Address>,
  recipient: <Address>,
  estimate_size: usize, // size in bytes
}
```

A receiving router MAY accept this notice by simply not responding, or
MAY reject the incoming message (for example via an automatic
filtering rule).  The sequence ID can be obtained from the containing
carrier frame.

```rust
struct DenyNotice {
  id: <Sequence Id>
}
```

When receiving a `DenyNotice` a sending router MUST immediately
terminate encoding and transmission.  Any intermediary router that
encounters a `DenyNotice`, which holds frames in its journal
associated with a stream ID MUST remove these frames from their
journals.


### Pull routing

An incoming message stream is still turned into [ERIS] blocks which
are encoded, encrypted, and content addressed.  Each block is saved in
the journal as it is generated, but not dispatched.  Once the manifest
has been created it will be sent towards the recipient peer.

This message routing mode will be used either:

1. When a sending client marks a message as a "Pull Payload"
2. When an active sending stream is interrupted by a broken connection

When the recipient router receives the signed message manifest it MAY
generate a set of pull request messages for the sender.

```rust
PullRequest {
  urn: "25660fc21c9b25b7fde985b8ae61b36dedcb8b0192e691eda60dff7c0e5ff00a"
}
```


## Appendix C: route scoring API

Currently this API is not exposed outside of the router and there is no mechanism to load external route scoring modules at runtime.  Nonetheless, additional route scorers can be added to the ratmand source tree, which comply to the following API:


```rust
#[async_trait]
trait RouteScorer {
    async fn configure(&self, 
        _ctx: &Arc<RatmanContext>, 
        _cfg: &mut ScorerConfiguration
    ) -> Result<()> {
        Ok(())
    }

    async fn irq_live_announcement(&self, 
        _a: &AnnounceFrame,
        _cfg: &mut ScorerConfiguration
    ) -> Result<()> {
        Ok(())
    }

    async fn compute(&self, 
        _stream_size: usize, 
        _cfg: &ScorerConfiguration, 
        _meta: &[&RouteData]
    ) -> Result<EpNeighbourPair>;
}

```

The implementation of `configure` and `irq_live_announcement` are optional, since the two default scoring mechanisms can run stateless.  Nonetheless configuration and "announcement capture" is supported via this API.

For any given route scoring module a `ScorerConfiguration` type is constructed and kept in memory along-side the actual scorer.  It's defined as follows:

```rust
struct ScorerConfiguration {
    trust_scores: BTreeMap<Address, u32>,
}
```

It contains a mapping of user-provided trust scores for a set of addresses  Trusted addresses indicate that the user of the device has manually verified the authenticity and validity of a given peer address (for example, that an address does really belong to a given user).  Addresses that have no associated trust score MUST NOT be included in the configuration set.  Currently trust scores are defined per-device.  It is left as an implementation detail to a scoring module how to compute trust scores based on the available router state.

When calling `compute` on the route scoring API a module SHOULD return a valid neighbour link and target ID (the link is a connection, the target is a selector for a specific peer on that sub-network).  In case no route could be selected this function MUST return `NonfatalError::NoAvailableRoute` which either fails-over into the next selection strategy or pause routing until a live link could be established again.


[RFC2119]: http://www.ietf.org/rfc/rfc2119.txt
[ERIS]: <https://eris.codeberg.page/spec/>
[ed25519]: <https://cr.yp.to/ecdh/curve25519-20060209.pdf>
[Appendix A]: #Appendix-A-MREP-Message-specification
