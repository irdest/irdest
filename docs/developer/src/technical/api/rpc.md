# Irdest RPC interface

The [previous section][prev] outlined the basics of the irdest core API.
This section details how you can write a service to use this interface
via an RPC connection.

Note: all interactions with the irdest core require a "service", even
if your application doesn't expose a service API.  Such services are
called "client services".

[prev]: index.html


## Register with the broker

The core of the irdest RPC (irpc) system is the RPC broker.  Your
application connects to the broker and then needs to register itself
with its address (name), version, and a human friendly description.

- **name** (example: `org.irdest.ping`) -- used to send messages and
  replies to your service/ application
- **version** (example: `1`) -- used for future compatibility checks
- **description** (example: `Send regular pings`) -- describe your
  service to users of the system
  
After registration with the broker your service is given a service ID.
Currenly this ID is unused, but will allow you to resume RPC sessions
in the future!

![](./rpc1.svg)

Every service is connected to the RPC broker, which forwards messages
between them.


## Connect to irdest-core

Connecting to the irdest-core service via the `irdest-sdk` crate is
documented in the [API docs][sdk-reg] -- but generally this should be
straight forward.  You gain access to the same API scopes as the
`irdest-core` crate exposes, except that calls are tunneled through
the RPC connection.

Both the [`irdest-sdk`] and [`irdest-core`] APIs document the same
function endpoints.

[`irdest-sdk`]: ...
[`irdest-core`]: ...


## Register with irdest-core (for services)

You may have to register your service with irdest core as well!  This
is to ensure that incoming messages with your service name as the
"associator" will be kept by the system instead of discarded.  This
also gives you access to the encrypted storage module of the irdest
core API.

You can do this _after_ connecting to the `irdest-core` service, by
calling `sdk.connect(...)`.  Check the [API docs][reg] on how this
works in detail!

The following flowchart shows a full registration between a service
and the broker, and then between the service and irdest-core.

![](./rpc2.svg)

At the end of this handshake your application/ service is now able to
use the irdest-core API to send messages and interact with users on an
irdest network.

[sdk-reg]: https://docs.irde.st/api/irdest_sdk/struct.IrdestSdk.html#method.connect
[core-api]: https://docs.irde.st/api/irdest_core/
[sdk-api]: https://docs.irde.st/api/irdest_sdk/
[reg]: ...
