# Irdest developer APIs

This section of the developer manual outlines the basic capabilities
of the irdest service API.  Most likely you will also want to read the
next section [RPC], which explains, how to connect to an
existing irdest daemon via the RPC interface.

Each of the sections below corresponds to an API scope in the irdest
service API!

[RPC]: ./rpc

## Users & UserAuth

The core authentication scope in irdest is a user.  A user is
represented and advertised on the network by its ID, a 128 bit
cryptographic public key.  This key is used to encrypt messages _to_
the user, and to verify the integrity of messages _from_ the user.

In order to issue a command to the API as a particular user, a session
first needs to be started.  This yields an authentication type called
`UserAuth`.  It contains both the user ID, as well as a random token
chosen by irdest-core, which is checked for validity on every future
API call.  Multiple sessions for the same user can exist at the same
time!

**Sessions do not currently time out, however this behaviour is
subject to change!**


## Messages

Sending messages between users is the primary way that data is
exchanged over an irdest network.  More complicated and stateful
algorithms can be implemented on top of this basic message sending
capability.  For that reason, messages sent via the irdest service API
are non-typed, arbitrarily sized byte arrays.  It is up to an
application/ service to decide what the payload of a message means.

When sending messages your service name is also encoded in them, to
allow remote peers to decide whether to keep a message, or discard it.
For example, a device that does not run the `foo` service may choose
to not keep incoming messages associated with it.

There are two modes when sending messages.  DIRECT, and FLOOD.  Direct
messages are addressed to a single user, and are routed by the
underlying network to the device and user in question.  Flood messages
are very different.  They are spread to every device reachable via the
network.  Devices that do not run your service will not _keep_ the
message, but they will still propagate it through the network to a
device that potentially does.

Also, keep in mind that neighbouring devices may choose to discard
your FLOOD messages if their message payload is too large.


## Contacts

Contact updates are a way for users to reveal more personal
information about themselves to other people on their network.  The
irdest API has a set of endpoints to search and update the local
contact book.  It is also a way for users to track "trust" and
friendships between users on the network.

The local contact book is filled with contact updates from peers on
the network.  Higher priority is given to updates from trusted and
friendly users in case of space limitations.

It is not currently possible to limit contact updates to trusted
users, but this will change in the future.


## Storage

The irdest core uses an encrypted database called [alexandria].  To
protect users against potential metadata side-channel attacks, your
service can store data in a per-user, per-service store inside the
same database.

This is of course optional, but encouraged, unless you have very a
very specific security-at-rest scheme in mind.
