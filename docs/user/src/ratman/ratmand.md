# Ratman Daemon

The core component behind the Irdest project is `ratmand`, a
stand-alone, decentralised routing daemon.  Ratman provides an
alternative address space via 256bit cryptographic keys, while peering
with different Ratman instances over a variety of transport
mechanisms.

Currently Ratman can only peer over an existing internet connection
but this will not always be the case.


## Installation

Please refer to [the website](https://irde.st/downloads) on how to install Ratman.

## State

Ratman stores some amount of state in `$XDG_DATA_HOME/share/ratmand`.
If you want to wipe all registered addresses simply delete the
directory and restart `ratmand`.


## Ratman daemon Usage.

Ratman comes with various commandline arguments to configure its
behaviour.  The main usage of the program can be queried via the
`--help` flag.  This section will describe flags in further detail.

### `--accept-unknown-peers`

By default Ratman rejects unknown incoming peer requests.  To disable
this functionality you need to pass this flag.

Note: if you want to run Ratman **without** having to provide a set of
peers to begin with (for example to purely act as a peering server)
then you **must also** provide this flag.

### `--no-inet`

By default Ratman tries to bind the inet overlay driver to connect to
peers across the internet.  To disable this functionality you can pass
this flag.

### `--no-discovery`

By default Ratman runs a local IPv6 multicast discovery driver which
will find other Ratman instances on your local network to peer with.
This flag disables that functionality.

### `-b`, `--bind`

This parameter flag allows you to override the default listening port
for Ratman IPC connections.  This is useful in case you want to run
multiple routers on the same machine, but will cause issue with
applications that don't allow you to specify the IPC connection socket
address (for example `irdest-echo`)!

### `--inet`

Specify the bind address and port for the netmod-inet overlay driver.
It supports both IPv6 and IPv4 address schemas.

### `-p`, `--peers`

This multi-parameter flag allows you to specify an initial set of
peers to connect to.  The syntax used for this is called `PEER_SYNTAX`
and used across various companents in Irdest.

A peer is expressed as follows: 

```
<netmod>#<address>:<port>
```

 - `<netmod>` refers to name of the netmod that the peer should be
   introduced to.  Currently the only valid netmod-identifier is
   `inet`.
 - `<address>` contains the main address part.  Domain names (provided
   you have a working DNS setup) are also accepted.
 - `<port>` finally the port to connect to
 
### `-f`, `--peer-file`

Similar parameter to `-p`, `-peers`, but pointing to a file instead.
This file must then contain a peer formatted in PEER_SYNTAX on each line.


### `-v`, `--verbosity`

Ratman can be configured to log more or less, depending on your needs.
Following is a breakdown of available log levels and how they are
generally used.


 - `error` -- Only print enexpected behaviour that has caused some
   operation to fail.  This does not mean that the whole router is
   crashing, but may be an early indicator of a fault
 - `warn` -- Only print for unexpected behaviour that doesn't
   otherwise impact the operation of the router.
 - `info` -- print at most one statement for each high-level component
   action.  It is possible to gauge the general operation of the
   router from just these messages
 - `debug` -- print at most two statements for each operation (for
   example "start" and "finish").  Debug messages are pleanty because
   all components emit them, but limit their re-occurence in each
   component.
 - `trace` -- includes individual step to each operation with no real
   limit on recurrence


## Running Ratman with user startup

Because `ratmand` is an alternative network stack, it needs to be
running for your computer to send and receive messages from the
network.

Ideally you should install Ratman via your distribution package
manager which _should_ include a service file.  But in case you are
using the static binaries from the website, you can use this service
file instead.


```
TODO
```
