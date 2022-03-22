# Technical Documentation

Welcome to the Irdest technical documentation.  This tree of documents
is meant to give you an overview into the various components that make
up the Irdest project, as well as their inner workings.

This manual is relevant for both **Irdest hackers**, and **Irdest
application developers**.


## Introduction

Irdest is a distributed routing system, creating an address space over
_ed25519 keys_.  Each address on the network is a public key, backed
by a corresponding private key.  This means that encryption and
message authentication are built into the routing layer of the
network.  Each physical device can be _home to many addresses_, used
by different applications, and it is not possible from the outside to
re-associate a specific device with a specific address.

A lot of traditional networking infrastructures is built up in layers
(see [OSI model][osi]).  Similarly, the irdest project replicates some
of these layers.  Note however that the layers between the OSI model
and Irdest don't map perfectly onto each other and are only meant to
illustrate difference in hardware access, user access, and scope.

[osi]: https://en.wikipedia.org/wiki/OSI_model

Following is a short overview of layers in Irdest.

| OSI Layer            | Irdest Layer      | Component(s)                                 |
|----------------------|-------------------|----------------------------------------------|
| Physical & Data link | Network drivers   | `netmod-inet`, `netmod-lan`, ...             |
| Network & Transport  | Ratman            | `ratman` (and the `ratmand` daemon)          |
| Session              | Integration shims | `irdest-proxy`, `ratcat`, ...                |
| Application          | Clients           | `irdest-ping`, `irdest-mblog`, ... your app? |


### Network drivers

Network drivers establish and manage connections with peers via
different underlying transport mechanisms (for example TCP
connections, but also more low-level protocols such as PPP).  A driver
(in the irdest jargon called a "netmod") is initialised and bound to
the process running the irdest router, and long-running.

Many different drivers can be active on the same device, as long as
they are connected to the same router.  In the OSI model, this maps to
layers 1 & 2.

Currently a driver needs to be specifically added to `ratmand` and
included at compile time.  We are working on a dynamic loading
mechanism however (either via `.so` object loading or an IPC socket).

### Irdest router

Ratman is a decentralised packet router daemon `ratmand`.  It comes
with a small set of utilities such as `ratcat` (a `netcat` analogue),
`ratctl` (a `batctl` analogue), and a simple management web UI.

Clients communicate with Ratman via a local TCP socket and protobuf
envelope schema.  For most use-cases we recommend the
[`ratman-client`] library.  Alternative implementations don't
currently exist and this API is also extremely unstable (sorry in
advance...)

In the OSI model, this maps to layer 3 and 4.


### Integration shims

Currently only one (work in progress) shim exists: `irdest-proxy`.
This layer aims to create interoperability layers between existing IP
networks and an Irdest/ Ratman network.

In future many different shims could exist, tunneling Tor traffic
through Irdest, or providing apps on mobile devices to take advantage
of the "VPN" functionality of the OS.

In the OSI model this maps to layer 5.  This is because in Ratman a
connection is stateless, and thus no real session state exists.  This
shim introduces the concept of sessions for the benefit of existing
applications that rely on them.


### Clients

The Irdest project is mainly focused on developing Ratman and its
associated drivers and shims.  We do however hope to provide some good
examples of applications written _specifically_ for an Irdest network
to inspire other developers, and showcase to users how these
technologies can be used.  This also aims to make the on-boarding
process less daunting.


## What next?

If you are interested in writing an application for Irdest, or porting
your existing application's networking to use Irdest in the
background, these sections are for you.

- [Ratman overview](./ratman/index.html)
- [Ratman client lib](./ratman/client.html)

If you want to work on a specific issue in Ratman or the drivers, we
recommend you check out the [issue tracker], or come talk to us in our
[Matrix channel]!
