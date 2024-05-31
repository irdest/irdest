# Ratman client protocol

External applications can connect to the Ratman routing daemon via a Tcp socket connection (by default `localhost:9020`, but this can be changed via the user configuration).  This page outlines the protocol in use for this connection.

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
| Status    | 0x7         | Query general status information                              |
| Sub       | 0x8         | Setup asynchronous stream subscriptions                       |
| Client    | 0x9         | Configure client connection                                   |

The following methods are availble:

| Name    | Byte-offset | Description                                                                                                                                       |
|---------|-------------|---------------------------------------------------------------------------------------------------------------------------------------------------|
| Create  | 0x1         | Create a new resource locally                                                                                                                     |
| Destroy | 0x2         | Destroy an existing local resource permanently                                                                                                    |
| Up      | 0x3         | Mark an existing resource as "up", which will start announcing it to the network.  This could be an address or a pre-cached stream (file sharing) |
| Down    | 0x4         | Mark an existing resource as "down", which will stop announcing it to the network, but not delete its local state                                 |
| Add     | 0x5         | Insert a new record into any data storage endpoint.  This operation is indempotent (applying an operation for the second time is a no-op)         |
| Delete  | 0x6         | Delete a record from any data storage endpoint.  This does not by default delete any associated data on disk (but can be opted into)              |
| Modify  | 0x7         | Modify an existing data storage record in place                                                                                                   |
| List    | 0x10        | List available records for a given namespace and storage endpoint                                                                                 |
| Query   | 0x11        | Run a query for specific data in the storage engine                                                                                               |
| One     | 0x12        | "One-to-one" mode when sending or receiving data, which locks a stream to a single destination address                                            |
| Many    | 0x13        | "One-to-many" mode when sending or receiving data, which allows message streams from and to multiple destination addresses                        |
| Flood   | 0x14        | "Flood" mode when sending, which sends a message stream to all network participants listening to a given flood address namespace                  |
| Fetch   | 0x15        | Pull cached message streams (partial or completed) from the router storage                                                                        |
| System  | 0x16        | ???                                                                                                                                               |
| Op addr | 0x17        | I genuinely forgot what this does                                                                                                                 |
| Op link | 0x18        | I'll read the source and fill this in later but lol                                                                                               |
