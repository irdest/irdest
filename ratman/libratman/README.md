# libratman

**libratman** is a  core part of the Irdest project, providing shared utilities for `ratmand` (a routing daemon), `ratcat` (a messaging tool), and `ratctl` (a control utility). It includes type definitions, encoding logic, and functions used by these tools and `netmod` libraries—userspace drivers for network hardware and protocols.

The crate offers two optional features: `client` and `netmod`. The `client` feature enables applications to connect to `ratmand` via a TCP socket for tasks like address management or message streaming, used by `ratctl` and `ratcat`.  The `netmod` feature provides tools for building `netmod` drivers, handling frame processing and link configuration independently of the router.

If you're looking at these crate docs you're most likely interested in writing an application (or integration) for Ratman, which means you should check out the [api] module!
