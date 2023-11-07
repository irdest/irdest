# Concepts & Ideas

An Irdest network is created via the "Ratman" router, and other Irdest
applications.  Different devices can be connected together locally
(via WiFi, ethernet, or long-range radio) or over the internet, as a
virtual public network.


## Network basics

An Irdest network is independent of traditional computer networking.
Irdest traffic can be routed via the existing internet, but creates
its own address space and routing rules.  **An Irdest network does not
use IP addresses!**

Instead an address is a cryptographic public key.  This allows all
data sent through the network to be encrypted and verified by default.

Because of the cryptographic nature of an address, a computer can also
have many addresses online at the same time.  Different addresses can
either be used for different applications or for different identities
via the same application (if it supports this).  There is no central
authority for handing out addresses.  Every computer self-generates
new addresses as they are needed.


## Ratman architecture

The Ratman router is a program running on your computer that connects
with other router instances and facilitates the exchange of messages
by applications via this network.

![](irdest-network.png)

Connections between different Ratman instances are created via
specific connection drivers.  Each driver allows Ratman to connect (or
"peer") with other instances of Ratman via some network channel.  For
example `netmod-inet` allows Ratman to connect via the internet,
`netmod-lora` via a [LoRa wireless](../how-to/02_lora.md) modem, etc.

Since Ratman is not part of your operating system kernel and uses
different addresses, special applications are needed to interface it
with existing network infrastructure (the web, your favourite video
game, etc).

For this purpose Irdest comes with a set of plugin-applications.
Currently this is only `irdest-proxy`, which can tunnel IP traffic
through an Irdest network.  Additonal plug-ins can be written via the
Ratman developer tools.


## Routing

Irdest is a mesh network, which means that anyone on the network can
communicate with anyone else by passing messages to participants in
between you and your recipient.  **This also means that there is no
central authority on how packets is transported.**

When registering an address, Ratman starts announcing it to its
neighbours (Ratman instances that it is peered to).  These neighbours
can then send messages to the new address.

When a Ratman instance receives an announcement, it updates its own
routing table, and then passes the announcement onward to any
neighbours that it things haven't seen the announcement yet.

Via this mechanism **an address will sooner or later be known** by the
whole network.

When sending a message to a particular address a router checks which
neighbour the address was announced from, (picks the best one if there
are multiple), and then sends data to that neighbour.  This is
repeated by any intermediary router until the message reaches its
destination!

This means that no single network participant can know the layout (or
"topology") of the network, slow nodes in between two points will be
avoided if other routes exist, and when creating a new address, it
will take some time until other network participants will be able to
communicate with that address.
