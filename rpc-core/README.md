# `qrpc` core components

A generic, async message bus to connect various applications together.
Each 'service' on the bus has an ID, which can be addressed directly.
A central broker (qrpc-broker) facilitates messages propagation and
reply timeouts.

As a developer for the qaul application ecosystem you will very rarely
have to interact with these components yourself.  Instead, use the
[qaul component SDKs](../sdk)!
