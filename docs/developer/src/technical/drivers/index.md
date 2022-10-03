# Network drivers

In Irdest a network driver is called a _netmod_ (short for _network
module_).  It is responsible for linking different instances of Ratman
together through some network channel.

The decoupling of router and network channel means that Ratman can run
on many more devices without explicit support in the Kernel for some
kind of networking.

Because interfacing with different networking channels comes with a
lot of logical overhead these network modules can become quite complex
and require their own framing, addressing, and discovery mechanisms.
This section in the manual aims to document the internal structure of
each network module to allow future contributors to more easily
understand and extend the code in question.


## Backplane types

There are three types of network backplanes that Irdest can interact
with:

- **Broadcast** :: only allowing messages to _all_ participants
- **Unicast** :: only alloing messages to a _single_ participant
- **Full range** :: allowing both broadcast and unicast message sending


## Netmod API

The netmod `Endpoint` API looks as follows:

```rust
#[async_trait]
trait Endpoint {
    fn msg_size_hint(&self) -> usize;

    async fn peer_mtu(&self, target: Option<Target>) -> Result<usize>;

    async fn send(&self, frame: Frame, target: Target, exclude: Option<u16>) -> Result<()>;

    async fn next(&self) -> Result<(Frame, Target)>;
}
```

<small>(This API is still in flux and needs to be extended in various
ways in the future.  Please note that this documentation _may_ be out
of date.  If you notice this being the case, please get in touch with
us so we can fix it!)</small>

* `msg_size_hint` is used to communicate a maximum size per message
  transfer and will be used to populate the
  `announcement.route.size_hint` parameter (on endpoints that support
  this!)
      
* `peer_mtu` is used to determine the immediate hop MTU to a target
  and will be used to populate the `announcement.route.mtu` parameter

* `send` is used to send messages.

  The `exclude` parameter is important on certain unicast & boardcast
  backplanes to prevent endless replication of flood messages.

* `next` is polled by the router in an asynchronous task to receive
  the next segment from the incoming frame queue.

This API is auto-implemented for all `Arc<T> where T: Endpoint`.
