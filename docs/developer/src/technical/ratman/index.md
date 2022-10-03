![](../../assets/ratman-banner.png)

This section outlines some internal concepts that are in use by the
Ratman routing daemon.

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


### Announcement metadata

As part of the announcement protocol nodes may include metadata in the
announcement.  According to the current specification draft, it looks
as follows:

```
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

For the following section the `announcement.route.mtu` parameter is
especially important!


## Message slicing & streaming

An incoming message in Ratman is sliced twice: once into cryptographic
blocks via the [ERIS](https://inqlab.net/projects/eris/) encoding, and
then again for transport according to the path MTU outlined in the
previous metadata section.

When slicing ERIS blocks, frames should be filled completely, with
non-overlapping block boundries.  This makes sure to not send frames
that contain a lot of zero-padding.

This should be implemented as an iterator/ stream, which consumes at
iterator/ stream of eris blocks.


```
ERIS block size = 4 bytes.
Frame size = 5 bytes.

Eris blocks: [1 2 3 4][5 6 7 8][9 10 11]
Frames:      [1 2 3 4 5][6 7 8 9 10][11]
```

```
ERIS block size = 12.
Frame size = 5.

Eris blocks: [1 2 3 4 5 6 7 8 9 A B C][D E F 10 11 12 13 14 15 16 17 18]
Frames:      [1 2 3 4 5][6 7 8 9 A][B C D E F][10 11 12 13 14][15 16 17 18]
```


### Selecting frame sizes

The two block sizes supported by ERIS by default are 1kB and 32kB.
For small messages these wil create a significant amount of overhead,
especially on low-MTU connections.

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


## WIP specification

There is a work-in-progress specification available [in the
wiki](https://hedgedoc.irde.st/X2JBI2rrQ8Oh9Q4yEO7sTQ)!
