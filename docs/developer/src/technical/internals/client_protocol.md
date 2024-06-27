# Ratman client protocol

External applications can connect to the Ratman routing daemon via a Tcp socket connection (by default `localhost:5852`, but this can be changed via the user configuration).  This page outlines the protocol in use for this connection.

If `libratman` SDK bindings exist for your language (currently only Rust) we highly recommend you use those instead of implementing the protocol from scratch.

## Microframe

Messages in the Ratman client protocol use a very simple framing mechanism specific to Irdest called `Microframe`.  Every Microframe message is split into a header and a body.

- `modes: u16`: encode the message command and payload types
- `auth: Option<ClientAuth>`: provide a previously registered auth token; field may be blank for initial connections
- `payload_size: u32`: encode the length of the main payload, up to the configured server maximum


The mode field is split into two parts: the namespace and the method.  A namespace specifies the internal API in use for a given command.  Not all namespace-command combinations are valid.  In `libratman` the mode is thus constructed as follows:

```rust
pub const fn make(ns: u8, op: u8) -> u16 {
    ((ns as u16) << 8) as u16 | op as u16
}
```

The following namespaces are available:

| Name      | Byte-offset | Description                                                   |
|-----------|-------------|---------------------------------------------------------------|
| Intrinsic | 0x0         | For internal use only                                         |
| Addr      | 0x1         | Local addresses and keys                                      |
| Contact   | 0x2         | Address-specific contact book for external addresses and keys |
| Link      | 0x3         | Hardware interfaces to connect to other Ratman instances      |
| Peer      | 0x4         | Other network participants without any relation metadata      |
| Recv      | 0x5         | Configure Ratman to receive a file                            |
| Send      | 0x6         | Sending data to other peers and the network                   |
| Stream    | 0x7         | Incoming streams namespace, separate from explicity receiving |

The following methods are availble:

| Name    | Byte-offset | Description                                                                                                                                       |
|---------|-------------|---------------------------------------------------------------------------------------------------------------------------------------------------|
| Create  | 0x1         | Create a new resource locally                                                                                                                     |
| Destroy | 0x2         | Destroy an existing local resource permanently                                                                                                    |
| Sub     | 0x3         | Subscribe to a particular resource (currently only streams are supported)                                                                         |
| Resub   | 0x4         | Restore a previously held subscription after a router restart                                                                                     |
| Unsub   | 0x5         | Unsubscribe from a previously subscribed resource                                                                                                 |
| Up      | 0x10        | Mark an existing resource as "up", which will start announcing it to the network.  This could be an address or a pre-cached stream (file sharing) |
| Down    | 0x11        | Mark an existing resource as "down", which will stop announcing it to the network, but not delete its local state                                 |
| Add     | 0x20        | Insert a new record into any data storage endpoint.  This operation is indempotent (applying an operation for the second time is a no-op)         |
| Delete  | 0x21        | Delete a record from any data storage endpoint.  This does not by default delete any associated data on disk (but can be opted into)              |
| Modify  | 0x22        | Modify an existing data storage record in place                                                                                                   |
| List    | 0x30        | List available records for a given namespace and storage endpoint                                                                                 |
| Query   | 0x31        | Run a query for specific data in the storage engine                                                                                               |
| One     | 0x32        | "One-to-one" mode when sending or receiving data, which locks a stream to a single destination address                                            |
| Many    | 0x33        | "One-to-many" mode when sending or receiving data, which allows message streams from and to multiple destination addresses                        |
| Status  | 0x34        | Get status updates on various components.  Currently only "Peer" and "Intrinsic" are supported                                                    |

Since the microframe header can have various sizes it is length-prepended with a 4-byte integer.  A full API protocol transmission thus looks as follows:

```
[ 4 byte header size ]
[ 2 byte mode indicator ]
[ 32 bytes auth token ][ 1 byte placeholder (0) ]
[ 4 byte payload length ]
[ N byte payload ]
```

## Command payload encoding

The available commands are described in `libratman/src/api/types`.  More documentation to be added.  Please feel free to ask if you run into any issues or have general questions.
