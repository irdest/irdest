# Concepts & Ideas

Irdest is a decentralised networking project.  An Irdest network is
created via the Ratman router and other Irdest applications.
Different devices can be connected together locally (via WiFi,
ethernet, or long-range radio) or over the internet as a VPN-like
network.

## Network basics

An Irdest network is independent of traditional computer networking.
Irdest traffic can be routed via the existing internet, but creates
its own address space and routing rules.  **An Irdest network does not
use IP addresses!**

Instead an address is a cryptographic public key.  This allows all
network traffic to be encrypted and signed by default!

Because of the cryptographic nature of an address, a computer can also
have many addresses registered at the same time.  Different addresses
can either be used for different applications or for different
identities via the same application.  There is no central authority
for handing out addresses.  Every computer self-generates new
addresses as they are needed.  The chance of collision in a 32 byte
(256 bit) address space are extremely small.

## Ratman architecture

An Irdest network is created by the Ratman router daemon.  This is a
program running on your computer that connects with other instances of
Ratman and facilitates the exchange of messages by applications via
direct links and intermediary routes.

*basic irdest network outline*

Irdest is a mesh network, which means that anyone on the network can
communicate with each other by passing messages to participants in
between you and your recipient.



Two other important concepts for Ratman are netmod drivers and
clients.  Consider the following graphic.

*insert Irdest stack graphic*

Ratman is a userspace routing daemon, meaning that it works outside of
your kernel.  This is done to be more platform independent and support
a wider range of operating systems!  To handle the actual connections
with other Ratman routers (called "peering") we use
connection-specific drivers called "Netmods".


## Routing



## What next?


