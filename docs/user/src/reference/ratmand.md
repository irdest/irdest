# Ratman Daemon

## State

Ratman stores some amount of state in: 
  - XDG system: `$XDG_DATA_HOME/share/ratmand`.
  - macOS:  `/Users/[USER_NAME]/Library/Application Support/org.irdest.ratmand`.

If you want to wipe all registered addresses simply delete the
directory and restart `ratmand`.


## Example usage

It's recommended not to use `inet` peering to connect to devices on
the same network as you, since the `lan` module takes care of this
already.

The most barebones way to launch Ratman is with the following command,
which doesn't by itself attempt to peer with anyone via the `inet`
module, but will accept incoming peer requests from the outside.  This
also runs the `lan` module in the background.

```
$ ratmand --accept-unknown-peers
```

We recommend you use the `-v debug` flag to increase the log volume
for Ratman in case you run into any issues (info logging is _very_
sparse at the moment!)

The Irdest project runs a set of peering servers.  To start your
daemon to connect to one of them you can use the next command.

```
$ ratmand -p 'inet#hyperion.kookie.space:9000' -v debug
```

This creates a connection to the peering server, meaning that the
server doesn't need to be able to reach you if you are behind a
firewall.  This is however enough to get you access to the rest of the
Irdest test network!

Beware trying to peer computers on the same local network this way, as
it may cause loops with the `lan` discovery module, which are not
currently handled very well.  You can also disable local discovery
with `--no-discovery`.

To demonstrate that you are indeed connected you can use `ratcat` in
combination with the `irdest-echo` application, as outlined
[here](../irdest-echo.html#public-instance)!

The following section outlines each commandline option and what it
does in more detail!


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


### `--no-dashboard`

By default Ratman runs a small web server on port 8090 to display a
usage and network state dashboard.  If this is unwanted you can turn
off this functionality.


### `-b`, `--bind`

This parameter flag allows you to override the default listening port
for Ratman IPC connections.  This is useful in case you want to run
multiple routers on the same machine, but will cause issue with
applications that don't allow you to specify the IPC connection socket
address (for example `irdest-echo`)!


### `--inet`

Specify the bind address and port for the netmod-inet overlay driver.
It supports both IPv6 and IPv4 address schemas.

**Note: when binding to either IPv4 or IPv6 specifically it makes it
impossible for a peer of the other version to properly peer with you.
This is an open issue tracked
[here](https://git.irde.st/we/irdest/-/issues/36)!


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
