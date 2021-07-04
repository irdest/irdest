# Technical Documentation

This is an introduction to the irdest technical documentation.
Following is an outline of the entire system, meant to give you a
broad overview of what irdest does, and what it aims to achieve.

This manual is relevant for both **irdest hackers**, and **irdest
application developers**.

**Note to future editors**: make sure every major component has an
`internals` section, so to make it convenient to skip.


## Introduction

Fundamentally, irdest is a highly distributed system.  It is not
accountable to a single set of rules across all devices that are part
of this system.  Each device can be home to many users, that can all
send messages to each other, but have separate states.  Furthermore,
connections in this system are not always real time.

A lot of traditional networking infrastructe is built up in layers
(see [OSI model][osi]).  Similarly, the irdest project replicates some
of these layers.

**Note** that the layers between the OSI model and irdest don't map
perfectly onto each other and are rather meant to illustrate
difference in scope!

To give you a better understanding of what parts of the irdest project
do what, here's an overview of the layers in a full irdest application
stack.  Each layer has a short description below.

[osi]: https://en.wikipedia.org/wiki/OSI_model

| Layer                         | Component(s)                 |
|-------------------------------|--------------------------------|
| Network drivers (Layer 1 & 2) | `netmod`, `netmod-tcp`, ...    |
| Irdest Router (Layer 3 & 4)   | `ratman`                       |
| Service API core (Layer 5\*)  | `irdest-core`                  |
| Irdest RPC/ SDK (N/A)         | `irdest-sdk`, `irpc-broker`    |
| Services (Layer 5\* & 6)      | `irdest-chat`, `irdest-groups` |
| Clients (Layer 6 & 7)         | `irdest-hubd`, `irdest-gtk`    |


### Network drivers

Network drivers establish and manage connections with peers via
different underlying transport mechanisms (for example TCP
connections, but also more low-level protocols such as PPP).  A driver
(in the irdest jargon called a "netmod") is initialised and bound to
the process running the irdest router, and long-running.

Many different drivers can be active on the same device, as long as
they are connected to the same router.  In the OSI model, this maps to
layers 1 & 2.


### Irdest router

A decentralised packet router that uses cryptographic public keys as
addresses for users.  One router is unique to a device and manages
both I/O with network drivers, as well as message requests to the
layers above.

The irdest router announces itself to other routers on the network via
the Mesh Router Command Protocol (MRCP), and updates its internal
routing table according to a chosen routing strategy, neighbour network
topology and link quality.

In the OSI model, this maps to layer 3 and 4.


### Service API core

The main irdest state handler, managing an encrypted database, user
profiles, and service state.  It handles the user web of trust, and
interactions of applications that use irdest as a networking backend.

In the OSI model, this partially maps to layer 5.


### Irdest RPC core & SDKs

A generic RPC protocol to connect different irdest services together.
Since there can only be one router running per device, it needs to
be possible for different applications to interact with this router at
the same time.  This is done by running the irdest core and router in
a background daemon and using an RPC protocol/ interface (called irpc)
to issue commands and get live updates from the network.

This layer is not represented in the OSI model and an implementation
detail of the irdest userspace design.


### Services

A service is a micro-application (or [actor]) that communicates with
other services via the irpc bus.  The scope of a service should be
small, and re-usable.  For example, `irdest-chat` implements a very
simple encrypted group chat system that can also easily be used by
third-party services that wish to use messaging in their user
experience.

Functionality should be, if possible, implemented as a service so that
other application developers can depend on it in their applications,
instead of having to re-write the functionality.  The distinction
between a service and a general development library is that a service
has access, and actively uses the irdest network and the irdest-core
API to perform tasks.

A service that does not expose an API of its own for other services to
use is called a "client service".  All end-user applications that use
irdest are client services.

In the OSI model this partially maps to layer 5 and 6.

[actor]: https://en.wikipedia.org/wiki/Actor_(programming_language)


### Clients/ Applications

User facing applications that use a collection of services to provide
users with a particular experience and implement different features.

The irdest project develops and ships a set of core services and
clients to demonstrate the functionality of the networking stack that
we are developing, and to act as a test bed to tweak performance
parameters in the lower layers.

But this should by no means be taken as the final scope of what irdest
can do.  Ideally, any application that requires some form of networked
communication with other users of the application can be ported to run
natively over an irdest network.

For "legacy" applications, we also provide a TCP/IP [proxy service]()
(work in progress).


## What next?

### For irdest application developers

We recommend you familiarise yourself with the irdest SDKs and RPC
system.  If you have further questions on how your users can see and
interact with each other, the following sections of this manual may
also be of interest to you.

- [Irdest APIs](./api/)
- [Irdest RPC overview](./api/rpc.html)
- [Routing basics](./ratman/basics.html)


### For irdest hackers

Depending on what part of the stack you want to work on, you should
read the introduction to that chapter, as well as the internals
section of each component.

If you have further questions, please do not hesitate to contact us!
