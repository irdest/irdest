# Mesh Router Exchange Protocol (MREP)

Status: *pre-draft*
Start date: *2022-04-05*
Latest revision: *2022-10-16*

The Irdest router Ratman exchanges user packets as well as routing metadata with neighbouring routers.  This communication is facilitated through the "mesh router exchange protocol".  It has three scopes.

1. Exchange user data packets
2. Collect connection metrics and transmission metadata
3. Perform routing decisions according to these metrics

The key words "_MUST_", "_MUST NOT_", "_REQUIRED_", "_SHALL_", "_SHALL NOT_", "_SHOULD_", "_SHOULD NOT_", "_RECOMMENDED_", "_MAY_", and "_OPTIONAL_" in this document are to be interpreted as described in [RFC2119]

## Basics

The Mesh Router Exchange Protocol specifies different mechanisms for multiple routers to communicate with each other in order to facilitate the flow of messages across a shared network.

Routers are connected with each other via different communication channels (or backplanes), meaning that the routing logic is decoupled from the connection logic.  For Ratman, this is done via the *Netmod* abstraction.  Each netmod allows routers to peer with each other over a variety of backplanes, with each requiring different platform specific configuration.

Simultaneously an Irdest router MUST allow connections from client applications, which can register addresses on the network.  *An address on an Irdest network is an [ed25519] public key (32 bytes long).*

A client application can register as many addresses as it needs.  From a network perspective the relationship between an address and a device can't necessarily be proven.

The terms "Address" and "Id" may be used interchangeably in this specification draft.


### Tangent: Ratman architecture

While this specification doesn't explicitly define the interface between an Irdest router and the way it connects networking channels between different instances, several of the mechanisms in this specification require communication between the wire layer and the routing layer.  This section shortly outlines the required function endpoints and the data they accept and provide.

(In future this section should become the base for a second specification.)

A networking endpoint is defined via the following trait:

```rust
#[async_trait]
trait Endpoint {
    fn msg_size_hint(&self) -> usize;

    async fn peer_mtu(&self, target: Option<Target>) -> Option<usize>;

    async fn send(&self, frame: Frame, target: Target, exclude: Option<u16>) -> Result<()>;

    async fn next(&self) -> Result<(Frame, Target)>;
}
```

- `fn msg_size_hint` should return the largest recommended message to be sent over this link at a time.  This metric is used to constrain traffic over low-bandwidth connections.  If no limit exists, return `0`.
- `fn peer_mtu` queries the MTU value for the peer represented by a target ID.  If no target is given the immediate/ broadcast MTU is queried.  If no MTU could be determined, return `None`.
- `fn send` takes a frame, a target ID, and an exclusion parameter (used to de-duplicate flood messages on certain backplanes).  Returns an error if the frame is too big or the peer is busy.
- `fn next` will be polled by Ratman on an async task to grab the next incoming frame from a particular endpoint
  
## Announcements

Irdest is a gossip based network, meaning that participant addresses need to announce themselves, and rely on intermediary nodes to propagate messages.

Announcements fundamentally allow different network participants to learn of each other's existence without a management intermediary.


### Router announcements

Every router on the Irdest network has a unique address.  Messages sent to this address MUST conform to the MREP Message specification ([Appendix A]).

Routers announce their address to other routers they are immediately connected to.  However unlike regular Irdest addresses, these announcements *SHOULD NOT* be propagated, unless explicitly instructed to do so by the sending router.  This functionality may be added in a later version of this specification.

Routers announce each other via a specific mechanism in the netmod API.  On broadcast backplanes this should happen at the same frequency as the address announcement loop.

On connection (or p2p) backplanes this should only be done when initialising a connection, or whenever a connection was re-established/ recovered.

Every router keeps track of the set of routers connected to it, to enable future queries specified in this protocol draft.


### Address announcements

When registering an address, this generates an [ed25519] keypair.  The private key is stored in the router, whereas the public key is used to announce the address on the network.  Ratman does not use address compressions at this time.

The default announcement rate is **2 seconds**.  Following is an example announcement payload.

```rust
Announcement {
  origin: {
    timestamp: "2022-09-19 23:40:27+02:00",
  },
  origin_sign: [binary data],
  
  peer: {
    ...
  },
  peer_sign: [binary data],
  
  route: {
    mtu: 1211,
  },
}
```

The announcement structure itself does not need to contain the announced address, since it is itself contained in a delivery frame, which contains the sending address.

Three metadata sections are used to communicate authenticity, reliability and scale of connections across a network.

Origin metadata is provided by the origin of the announcement, and *MUST* thus be signed (via the `origin_sign` field).  As per this specification it *MUST* contain a timestamp of when the announcement was generated.

Additional announcement origin metadata MAY be included by the originator in the `origin` section.  Routers *MUST* NOT strip this metadata, since it would break the authenticity verification for the announcement.  **However** announcements with origin metadata fields larger than 1024 bytes *MUST* be discarded.  This limit may be reduced in a future version of this specification.

Peer metadata is provided by the immediate peer that an announcement was propagated via.  Let's look at the following example.

```
A  ->  B  ->  C
```

In the above example, router A is connected to router B, which is connected to both A and C, and router C, which is only connected to router B.  Messages can flow from A to C via B bidirectionally.

An announcement from A to B will contain origin metadata from A, as well as peer metadata from A.  However when B replicates the announcement to C, origin metadata will still be from A, but peer metadata from B.

Peer metadata is used to provide additional data about a network address, which may not be generally relevant to the network and instead only spread for a single hop.  Peer metadata is signed by the router's address key. For every incoming announcement, routers *MUST* replace the previous peer metadata section with their own.  If no metadata is provided both the `peer` and `peer_sign` fields *MUST* be filled with zeros.

Lastly an announcement contains route metadata, which is not signed and can not be trusted.  This field allows routers along the way to deposit helpful markers to other routers in the downstream of a particular announcement.

This may be related to the reliability of connections, the limitation of bandwidth, or that a uni-directional network boundary was crossed.  Currently only the `mtu` field is specified.  Additional fields will be added by later revisions of this specification.

Additionally, the route metadata section *MUST* be pruned if it surpasses 512 bytes in length.  Pruning means that only recognized fields will be kept, while unknown keys are discarded.


## Route scoring

Consider the following scenario:

```
A -  B  - C
 \ D - E /
```

Router A is connected to `B` and `D`, router `B` is connected to `A` and `C`, router `C` is connected to `B` and `E`, etc.

Announcements from `A` will reach `C` both via the link from `B` to `C` and from `E` to `C`.  Thus, when `C` wants to send a message to `A` it needs to consider multiple routes for sending messages.  This is called "route scoring" and is implemented via the Route scoring API (see Appendix C).

There are many different approaches that can be taken for route scoring, especially for loop avoidance strategies.  By making this component modular, it means that future additions can radically change or overhaul the way that Ratman performs this scoring mechanism, without needing large amounts of re-engineering in the core of Ratman.

This also allows network researchers to integrate their research code into an existing networking and testing platform.


### Default greedy strategy

This is the default routing strategy in Ratman.

For any connection/ message pair not qualifying for a small message optimisation strategy, ...

*Outline*

- For "large" MTUs (> 1200 bytes?) select path of smallest ETD
- Otherwise balance an MTU curve vs ETD curve (don't reward high MTU or low ETD at the cost of the other)


### Small message optimisation strategy

This routing strategy is activated when sending messages over a connection that fall under the message/ MTU metric requirements of [Low bandwidth modes](#Low-bandwidth-modes).

*Outline*

- Deny messages that are too big
- Use pull routing
- Send the manifest first
- Use a store & forward protocol (up to a limit?)


## Security

<small>(I am not a cryptographer and this section will have to be expanded/ reviewed in the future)</small>

All user messages sent through Irdest are encrypted via the [ChaCha20](https://en.wikipedia.org/wiki/Salsa20#ChaCha_variant) stream cipher, provided by the [ERIS] block slicing specification which is used to encode user payloads.

An address in Irdest is an public key (also called "address key"), backed by a corresponding secret key.  Keys are generated on the [ed25519] twisted edwards curve.

Because ChaCha20 is a *stream cipher* it requires a symmetric secret to work.  For this purpose we convert both secret and public keys from the edwards curve representation to the montgomery curve representation, and use this for the *x25519 diffie-hellman* handshake between the sending address secret key and the recipient address key.

There are two layers of signatures in Irdest.  The base layer Frame (defined in [Appendix A]) contains space for an ed25519 signature.  All message payloads are signed by the sending address edward's curve key (currently using `ed25519-dalek`) and can be verified by any node a message traverses (since the sending address is visible to any network participant).

For user payloads, ERIS guarantees message integrity by verifying block content hashes against the recorded versions in the manifest.  This manifest message is also signed via the basic Frame delivery mechanism.  Thus user message integrity can be guaranteed.


## Message encoding & delivery

User payloads are encoded via [ERIS], sliced into delivery Frames, and sent towards their destination (see [Appendix A] on details).

This uses two basic message types: `DataFrame` and `ManifestFrame`.  Messages in Irdest are sessionless streams, meaning that data is streamed between different Irdest routers, but buffered into complete messages before being exposed to the recipient application.

ERIS specifies a *"Read Capability"* which for the purposes of Irdest and this spec we are calling the *"Manifest"*.

For a `DataFrame` the payload of the underlying delivery Frame is entirely filled with content from a re-aligned block stream.  Frames MUST NOT be padded with trailing zeros to fill the target MTU.

A `ManifestFrame` contains a binary encoded version of the "Read Capability".  If this manifest is too large for the containing delivery frame, it is split into multiple frames (see [Appendix A: Manifest Frame](#Manifest-Frame))


## Journal sync

Irdest allows devices to connect to each other via short-lived (or "ephemeral") connections.  One such application is Android phones, where p2p WiFi connections can only be established with a single other party at a time.  Bluetooth mesh groups are possible, but are also significantly limited in the number of active connections.

For this purpose we introduce the "journal sync" mechanism.

An Irdest router MUST contain a journal of content-addressed blocks of data (see [Appendix B](#Appendix-B-message-routing)).  Messages are indexed via their content hashes, as well as the recipient information.  A journal sync is a uni-directional operation, which should be applied in both directions of the link.  What that means is that journals are not so much synced, but propagated.

Let's look at an example to demonstrate the process.

Routers A and B are connected to each other via an ephemeral connection (`req_ephemeral_connection` is called by a netmod driver which has established the connection).

First both routers exchange a list of known addresses.  Future versions of this specification MAY implement some kind of compression or optimisation for this transfer, since routing tables may get quite large.

```rust
SyncScopeRequest {
  addrs: BTreeSet<Address>,
}
```




*Outline:*

- Exchange list of known addresses (with an optimisation for "last recently used")
- Forward blocks addressed to any of the known addresses
- How to avoid re-transmit loops in a group of phones?
- How to avoid having to send too much data?
- Loops between people who both infrequently see the same peer address?  Who gets the frames? Both? (probably)


## AGPL compliance

Ratman is licensed under the AGPL-3.0 license and as such needs to be able to provide its own source code.

It is not possible to query the source of a node more than *one router edge* away from your own since router address announcements do not propagate across the network.

A router MAY at any time send a source request to a connected router.  The request is time-stamped to avoid repeated and duplicate requests.

```rust
SourceRequest {
  date: "2022-09-22 03:18:32"
}
```

As a response, the recipient router MUST send a `SourceResponse` reply.  The response doesn't contain the source code.  Instead it describes the source that is running.  A `SourceResponse` MUST contain the `source_urn` field.  Every other field is optional, but a router SHOULD still provide them.  The `note` field SHOULD contain a list of patch-names that have been applied to the node, if the `number_of_patches` is not zero.  Otherwise this field SHOULD remain empty.

```rust
SourceResponse {
  version: "0.5",
  number_of_patches: 0,
  source_url: "https://git.irde.st/we/irdest",
  source_urn: "b5bb9d8014a0f9b1d61e21e796d78dccdf1352f23cd32812f4850b878ae4944c",
  note: "Hier könnte Ihre Werbung stehen",
}
```

A recipient of this `SourceResponse` can now check whether the source code their node is running is the same as the router that responded, by checking the `source_urn` against their own source version (TODO: specify how this URN is generated).

In case the recipient doesn't already have this source code they can now send a `PullRequest` to the sending node:

```rust
PullRequest {
  urn: "b5bb9d8014a0f9b1d61e21e796d78dccdf1352f23cd32812f4850b878ae4944c",
}
```

## Low bandwidth modes

Some links in an Irdest network may be extremely low bandwidth, for example when using `netmod-lora` for long range communication.  This severely constricts the maximum transfer size (< 255 bytes), on a < 1% duty cycle.  This means that the *maximum incoming message size* MUST be constricted as well.

In these cases the "Small Message Optimisation" (SMO) MUST be used.  Following is a table that outlines the selection of encoding block sizes based on the determined path MTU and size-hint (via `announcement.route.mtu` and `announcement.route.size-hint`)

For these cases we should have small-message optimisations, based on the size of the message, and the path MTU to the recipient.

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

A frame may encounter a netmod link that doesn't allow for a sufficiently sized MTU

In some cases, the path MTU information on the sending node was incorrect, and a set of frames will encounter a link that is too low-bandwidth to support their size.  In this case the "leap-frogging" protocol should be used.

The first frame in a series that is too large to transmit over a connection will be prepended with this metadata section:

```rust
LinkLeapRequest {
  seq_id: "1D90-C2AB-E50D-A4EC-F88C-BD9E-818B-7006-7D32-BED0-4EEC-83F0-756E-D856-40AA-B611",
  inc_mtu: 1222,
  look_ahead: false,
}
```

- `seq_id` :: the sequence ID for the incoming set of frames.  This identifier is used to determine which frames need to be re-sliced
- `inc_mtu` is the size of incoming frames

MTU leap-frogging performs a single step of look-ahead.  This means that a router receiving a `LinkLeapRequest` MUST perform an MTU look-ahead if `request.look_ahead` is set to `true` (and subsequently set it to `false`).  This means that up to two link MTU limitations can be "skipped over" before having to re-collect into the original frame size and re-slicing.

For an incoming `LinkLeapRequest` a router MUST spawn a `LeapFrameCollector`



## Appendix A: MREP Message specification

This section of the specification outlines the way that MREP container messages are encoded.

The basic container type of any message in an Irdest network is a delivery frame, which has the following structure:

```rust
Frame {
  modes: [u8; 2],
  recipient: [u8; 32] (optional),
  sender: [u8; 32],
  seq_id: [u8; 16] (optional),
  signature: [u8; 32] (optional),
  pl_size: u16,
  payload: &[u8]
}
```

This message structure is **byte aligned**.

- `modes` :: a bitfield that specifies what type of content is encoded into the payload
- `recipient` :: (Optional) recipient address key.  May be replaced with a single zero byte if the frame is not addressed (see below).
- `sender` :: mandatory sender address key
- `seq_id` :: (Optional) sequence ID for push messaging payloads, mtu-leap protocol, etc
- `signature` :: (Optional) payload signature, generated by the sending key.  May be replaced with a single zero byte if the frame has a payload-internal signature (see below).
- `pl_size` :: 16 bit unsigned integer indicating the size of the data section.  Frame payloads larger than 32kiB are not supported!
- `payload` :: variable length payload.  Encoding is protocol protocol specific and payload MUST be prepended with an 2 byte size indicator.

Following is a (*work in progress*) overview of valid bitfields.  If a field is _not_ listed it is _invalid_!  Routers that encounter an invalid message MUST discard it.


| Bitfield              | Frame type descriptor |
|-----------------------|-----------------------|
| `0000 0000 0000 0000` | Address Announcement  |
| `0000 0000 0000 0010` | Data frame            |
| `0000 0000 0000 0011` | Manifest frame        |
| `0000 0000 0000 0100` | Router Announcement   |
| `0000 0000 0000 1xxx` | (Reserved)            |
| `0000 0000 0010 0001` | SyncScopeRequest      |
| `0000 0000 001x xxx0` | (Reserved)            |
| `0000 0000 0100 0001` | SourceRequest         |
| `0000 0000 0010 0010` | SourceResponse        |
| `0000 0000 01xx xx11` | (Reserved)            |
| `0000 0000 1000 0001` | PushNotice            |
| `0000 0000 1000 0010` | DenyNotice            |
| `0000 0000 1000 0011` | PullRequest           |
| `0xxx xxx1 xxxx xxxx` | (Reserved)            |
| `1000 0000 0000 0001` | LinkLeapNotice        |

Message payloads are encoded via their own Protobuf schemas (**TODO**: turn pseudo-code schemas from this specification into protobuf type definitions).

### Announcement

`Announcement` frames are special in that they MUST set the `recipient` and `signature` field to a single zero byte.  This is because announcements are not addressed, and contain a payload-internal signature system.  All other message types handled by this specification MUST include both a recipient and signature!

```protobuf
message Announcement {
    OriginData origin = 1;
    bytes origin_sign = 2;

    PeerData peer = 3;
    bytes peer_sign = 4;

    RouteData route = 5;
}

message OriginData {
    string timestamp = 1;
}

message PeerData { /* tbd */ }

message RouteData {
    uint32 mtu = 1;
    uint32 size_hint = 2;
}
```

### Data Frame

A data frame is already explicitly sliced to fit into a delivery frame (see "MTU leap-frogging" for how to handle exceptions to this).  Therefore the payload content can simply be encoded as a set of bytes.

The delivery frame knows the size of the payload.  Thus no special encoding for data frames is required.


### Manifest Frame

Message manifests SHOULD generally fit into a single delivery frame.  This may not be the case on low-bandwidth connections.  Additionally, because the manifest has no well-defined representation in the [ERIS] spec, we need to wrap it in our own encoding schema.

```protobuf
message Manifest {
    string root_urn = 1;
    string root_salt = 2;
}
```

### Router Announcement

```protobuf
message RouterAnnouncement {

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

- **Push routing** is used by default when an active connection to a peer is present.
- **Pull routing** is used whenever there is no active connection, or for particularly large or static payloads.


### Push routing

This is the default routing mode.  It is used whenever an active connection is present, or if the sending application didn't provide any additional instructions.

A message stream is encoded into [ERIS] blocks which are encoded, encrypted, and content addressed.  Each block is saved in the Router journal.  Lastly a message manifest is generated and signed by the sending key.  Blocks are sent to the recipient as they are generated, avoiding having to save the entire message in memory.

Lastly a message manifest is generated which contains the content hashes of each previous block.  This manifest is signed by the sending key and also sent to the recipient.

On the receiving side blocks are kept in the journal until the manifest is received, then the message can be verified and decoded for the receiving application.

For messages larger than `N` MB (tbd), a sending router MUST generate a `PushNotice` before the final message manifest has been generated.

```rust
PushNotice {
  sender: <Address>,
  recipient: <Address>,
  estimate_size: usize, // size in bytes
}
```

A receiving router MAY accept this notice by simply not responding, or MAY reject the incoming message (for example via an automatic filtering rule).  The sequence ID can be obtained from the containing delivery frame.

```rust
DenyNotice {
  id: <Sequence Id>
}
```

When receiving a `DenyNotice` a sending router MUST immediately terminate encoding and transmission.  Any intermediary router that encounters a `DenyNotice`, which holds frames in its journal associated with a stream ID MUST remove these frames from their journals.


### Pull routing

An incoming message stream is still turned into [ERIS] blocks which are encoded, encrypted, and content addressed.  Each block is saved in the journal as it is generated, but not dispatched.  Once the manifest has been created it will be sent towards the recipient peer.

This message routing mode will be used either:

1. When a sending client marks a message as a "Pull Payload"
2. When an active sending stream is interrupted by a broken connection

When the recipient router receives the signed message manifest it MAY generate a set of pull request messages for the sender.

```rust
PullRequest {
  urn: "25660fc21c9b25b7fde985b8ae61b36dedcb8b0192e691eda60dff7c0e5ff00a"
}
```


## Appendix C: route scoring API

Consider the following scenario:

```
A -  B  - C
 \ D - E /
```

When routing a message from `C` to `A` first the routing table will return a set of available routes, with associated metadata:

```rust
RouteData {
  peer: [0, 2],
  meta: {
    mtu: 1222,
    size_hint: None,
    etd: "00:00:31.110",
  }
}
```

- `peer` :: the tuple of `endpoint` and `target` identifiers that identify a routing "direction"
- `meta.mtu` :: maximum transfer unit of the route
- `meta.size_hint` :: maximum total message size of the route.  None means there is no imposed limit
- `meta.etd` :: "estimated transfer delay" uses the incoming announcement timestamp and calculates a delay to the current system time.  While this metric can be _very inaccurate_ due to different time sync mechanisms or badly configured timezones, a route scoring system may still access it.

The API for route scoring is defined via the `RouteScore` trait:

```rust
trait RouteScore {
    async fn configure(&self, r: &Router) -> Result<()>; 

    async fn irq_live_announcement(&self, a: &Announcement) -> Result<(), RouteScoreError>;
    
    async fn compute(&self, msg_size: usize, meta: [&RouteData]) -> Result<usize, RouteScoreError>;
}
```

Via the `configure` flag the router can be set-up to send live announcements to the given route score module by calling `Router::req_live_announce(&route_scorer)`.  For any incoming announcement Ratman will then call the `irq_live_announcement` endpoint with a given announcement frame.

```rust
enum RouteScoreError {
    UpdateFailed(String),
    
    ReSelect(enum Branch {
        Small,
        Delay,
        Trust,
        Neighbour,
    })
}
```

[RFC2119]: http://www.ietf.org/rfc/rfc2119.txt
[ERIS]: <https://eris.codeberg.page/spec/>
[ed25519]: <https://cr.yp.to/ecdh/curve25519-20060209.pdf>
[Appendix A]: #Appendix-A-MREP-Message-specification
