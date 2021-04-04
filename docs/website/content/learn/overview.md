---
Title: Networking overview
layout: page
---

irdest implements a heterogenious, fully decentralised mesh network.
What this means is that transmission channels can change between
device links: the network is made up of roaming network shards.  To
allow routing across network boundries, irdest uses a public key
address space, to avoid having to deal with network collisions, and
NAT.

For links that rely on regular IP spaces (v4 or v6), a local lookup
table is kept in each node that operates an overlay endpoint.

This way irdest can run on unpriviledged devices, because it doesn't
rely on modifying kernel routing table parameters to create a DHT
(distributed hash table); all routing is done entirely in userspace.


## Breadcrumb routing

The router behind irdest is called "ratman" (route and transmission
manager), which works via a gossip announcement protocol.  Each node
on a network periodically announces itself to it's peers, letting them
know over which of their local interfaces they can reach this node.
Because of the coupling between ratman IDs and irdest users, this
corresponds to announcing a user identity on the network for irdest,
but different applications using ratman can bind these IDs to
different semantics.

Let's look at a small example:

```

  +--------> [ D ] <---------+
  v                          v
[ A ] <----> [ B ] <-----> [ C ]

```

Node `A` sends announcements to `B` an `D`, who will both proxy it to
`C`.  The router in `C` will use various metrics to decide which link is
more stable, and declare it the "primary" for peer `A`.  When `C` now
wants to route a message to `A`, it looks up the local interface over
which it thinks it can reach `A` the best (for example `D`).  It then
dispatches the message to `D`, hoping that this node will know where
to deliver it.

No node in the network knows the full network topology, meaning that
it can dynamically change, without greatly impacting routing
performance.

There are several corner cases for what to do when a loop is
encountered, or when a network becomes too big that it needs to be
segmented.  But this covers the basics of how ratman and irdest do
routing.


## Automatic roaming

Because irdest is primarily designed to run on unpriviledged mobile
devices, a network automatically needs to be able to roam between
transmission channels.  In fact, because of technical limitations on
mobile platforms connections between devices can't be long lived and
need to be broken up after a transmission, to sync with the next peer.

To make this handshake faster, ratman can sync the undelivered message
buffers (called message journal) via a Merkel tree.  This way mobile
devices round-robin their connection states, each syncing undelivered
packets to a neighbour, before moving on to the next peer.
