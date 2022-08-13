# Irdest Proxy

This program implements an IP traffic proxy through a Ratman network.
Currently it only supports TCP connections for static routes.  The
easiest way to install `irdest-proxy` is via our static binary
bundles!

Alternatively you can compile it from source:

```console
$ cargo build --release --bin irdest-proxy --all-features
...
$ target/release/irdest-proxy
```

## Configuration

irdest-proxy uses static routes between an **inlet** and an **outlet**
of the network.  Traffic will flow into one side, be transported
through a Ratman network, and emerge from the other side.  To
configure these routes, both endpoints must agree on their
configuration.

This is done via the `routes.pm` configuration:

```
## Configuration for the Inlet
0.0.0.0:8000 -> CC92-A682-2FEF-C84F-974C-3423-5359-5BFE-3042-1C89-47CC-7064-3E2F-ECF7-A84D-7841

## Configuration for the Outlet
duckduckgo.com:443 <- CC92-A682-2FEF-C84F-974C-3423-5359-5BFE-3042-1C89-47CC-7064-3E2F-ECF7-A84D-7841
```

The syntax follow this schema: `<IP address | hostname> [<-|->]
<Ratman address>`.  An Inlet uses the `->` syntax to map a **bind
address** to a Ratman address.  The Outlet uses the `<-` syntax to map
a Ratman address to a **destination address**.  In the above example
traffic sent to `localhost:8000` on the Inlet machine will be sent to
`duckduckgo.com:443` from the Outlet machine.  This is a
bi-directional mapping, meaning that response packets from the
destination will make it back to the originator.

## Setup

Currently the setup process is somewhat manual.  You should create
`~/.config/irdest-proxy` (or wherever your `XDG_CONFIG_HOME` is
located).  Inside that directory, create a `routes.pm` file with the
above syntax.  For every mapping you will have to create a dedicated
address on the **Outlet machine**.  This is best done via `ratcat
--register --no-default`, which will register a new address without
making it the default address `ratcat` uses in the future.

