# Cap'n proto sockets

**This page is outdated!**

One way to inteact with irdest-core is via the `irdest-core-ipc` crate which
implements the same API as irdest-core, while tunneling all calls through
a previously negotiated unix socket, using the cap'n proto IPC
protocol.

The same ffi C interface as for irdest-core can be used, meaning that it's
possible to write a service that uses this high-performance IPC
channel from nearly any language.

More docs to follow, as this is still WIP!
