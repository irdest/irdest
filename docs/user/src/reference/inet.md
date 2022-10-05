# Peering via `inet`

For a broad overview of Irdest see [overview](../overview.html)!

`inet` is the "internet overlay" driver, which can establish
connections to other Ratman instances over the internet.  As part of
this peer-to-peer network there are different ways of configuring
`inet`, which will be outlined in this chapter of the manual.


## Connection modes

The `inet` driver has two modes of connecting to a remote: `standard`
and `cross`.

- A `standard` connection works similar to how your computer connects
  to any server.  It is especially useful if your computer is behind a
  firewall that does not support UPnP!  You create an outgoing
  connection, and in case of a connection loss (for example your
  internet goes down) your computer is responsible for re-establishing
  the connection.  This is the simplest connection mode, and also the
  default.
  
- The alternative connection mode is `cross`, which is recommended for
  peering setups between servers, Ratman hubs, or any computer behind
  a firewall that supports UPnP.  In this case both instances create
  connections to each other, meaning that neither of the two is the
  "server" (or both are, depending on how you look at it).
  
  The advantage to this mode is that after a connection loss, either
  side can re-establish the connection, making this peering mode more
  resilient.  A cross connection is established by appending an `X` to
  the address of a peer.


### Examples

Following is an example `peerfile.txt` which instructs the `inet`
driver to connect to a set of peers.  In this case the first two
connection is a `standard` connection, creating a client-server
relationship, whereas the third creates a true peer-to-peer
connection.

```
clouds.irde.st:9000
hyperion.kookie.space:9000
mynas42.dyndns.com:9000X
```


## Additional settings

`inet` can be configured via the main `ratmand` configuration file (in
the `inet` section).  While settings are listed in the [configuration]
page, this section contains more in-depth explanations behind the
settings.


