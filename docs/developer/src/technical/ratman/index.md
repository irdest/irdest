![](../../assets/ratman-banner.png)

A modular userspace packet router, providing decentralised distance
vector routing and message delay tolerance.  Ratman is both a Rust
library and stand-alone and zero-config routing daemon.

Ratman provides a _256-bit address space via ed25519 public keys_.
This means that messages are end-to-end encrypted and authenticated at
the transport level.  An address is announced via gossip protocol
routing to other network participants.

Ratman somewhat exists outside the OSI layer scopes.  On one side (the
"bottom") it binds to _different transport layers_ such as IP, or BLE
via "net module" drivers.

On the other side (the "top") it provides a protobuf API on a local
TCP socket for _applications to interact with this network_.  One of
the core principles of Ratman is to make network roaming easier,
building a general abstraction over a network, leaving it up to
drivers to interface with implementation specifics.

This also means that connections in a Ratman network can be ephemeral
and can sustain long periods of inactivity because of the
delay-tolerant nature of this routing approach.


## Gossip announcements

Ratman operates on the [gossip protocol] approach, where each address
on the network repeatedly announces itself to other network
participants.  Based on these announcements the routing tables of
passing devices and peers will be updated as needed.  This means that
no single device will ever have a full view of the network state, but
will always know the "direction" a packet needs to be sent in order to
make progress towards its destination.  This is similar to how
existing routing protocols such as [BATMAN] and [BGP] work.

[gossip protocol]: https://en.wikipedia.org/wiki/Gossip_protocol
[BATMAN]: https://en.wikipedia.org/wiki/B.A.T.M.A.N.
[BGP]: https://en.wikipedia.org/wiki/Border_Gateway_Protocol


### A short example

To illustrate this capability, let's look at this simple network graph:

```
  +--------- [ D ] ----------+
  |                          |
[ A ] ------ [ B ] ------- [ C ]
```

Node `A` sends announcements to `B` an `D`, which will both proxy it
to `C`. The router at `C` will use various metrics to decide which
link is more stable, and declare it the "primary" for peer `A`.

When `C` wants to route a packet to `A`, it looks up the local
interface over which it thinks it can reach `A` the best (for example
`D`). It then dispatches the packet to `D`, knowing that this node
must be closer to the destination to deliver the packet.


## Delay tolerance

This section will be expanded when the implementation of delay
tolerance becomes more stable and usable.  But in short: messages can
be buffered by various nodes across the network when the destination
is not reachable.  This means that different networks can communicate
with each other even when no stable connections between them exist
(for example via a [sneaker net]).  This does however require
applications to be aware of long delays and handle them gracefully!

[sneaker net]: https://en.wikipedia.org/wiki/Sneakernet

## Roaming

Because of the distinction between network channels and routing it is
easy to roam across network boundries.  As long as a channel can be
established (even just one-way), packets can be sent through the
Irdest network with no knowledge of these network bounds.

It also means that a network can easily be composed of different
routing channels.  Local UDP discovery, TCP links across the existing
internet, Wireless antenna communities, and even phones.
