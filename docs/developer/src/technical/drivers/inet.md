# Internet overlay netmod (inet)

The main way to use Ratman with other people at the moment is via the
_internet overlay network module_ inet.  It creates peering sessions
over the internet and TCP.  With that comes a significant amount of
connection state logic and routing outside of Ratman, because each
instance of inet can be connected with many other instances of inet.


## Structure diagram

Following is a class structure diagram for the three main components
of the `inet` driver.  Note that `Server` is a dispatch-type, meaning
that after allocation it copies itself to a private task-stack and
remains running until the containing application is shut down.

`TODO: figure out why the mermaid graph is borked`

```mermaid
classDiagram-v2

class InetEndpoint {
    +Arc[Routes] routes
    +ChannelPair channel
    +start( bind )
    +port()
    +add_peers( peers )
    +send( target, frame )
    +send_all( frame )
    +next()
}

class Routes {
    +AtomicU16 latest
    +BTreeMap[Target, Peer] inner
    +next_target()
    +add_peer( target, peer )
    +remove_peer( target )
    +exists( target )
    +get_peer_by_id( target )
    +get_all_valid()
}

class Server {
    +Option[TcpListener] ipv4_listen
    +TcpListener ipv6_listen
    +port()
    +run()
}
```

## Flowchart

While the `inet` driver doesn't have a lot of type components, their
interactions can get quite complex.  Furthermore there are some
stateless function components that can't be expressed in a traditional
class diagram.

```dot process
digraph G {
  size="8,30!"
  graph [fontname = "Handlee"]
  node [fontname = "Handlee"]
  edge [fontname = "Handlee"]
  splines="polyline"
  bgcolor=transparent
  nodesep=0.5
  

  subgraph cluster_0 {
    color=orange
    node [color=orange]
    
    A [label="InetEndpoint::new"]
    B [label="add_peers()"]
    S [label="Listen for\nincoming connections"]
    C [label="for each peer", shape=Mdiamond]
    D [label="Resolve address"]
    E [label="start_connection()"]
    E1 [label="connect()"]
    E2 [label="handshake()"]
    E3 [label="Add peer\nto Arc<Routes>"]
    E4 [label ="SPAWN peer.run()"]
    E5 [label="setup_cleanuptask()"]
    Z [label="SPAWN restart.recv()"]

    A -> B
    A -> S
    B -> C
    C -> D
    D -> E
    E -> E1
    E1 -> E1 [label="retry"]
    E1 -> E2
    E2 -> E3
    E2 -> E4
    E3 -> E5
    E5 -> Z
    
    label = "Initialisation"
    fontsize = 20
  }
  
  subgraph cluster_1 {
    color=cyan
    label = "Peer Frame Receiver"
    node [color=cyan]
    
    P1 [label="run() loop", shape=Mdiamond]
    P2 [label="Read Frame"]
    P22 [label="Release Mutex"]
    P222 [label="break", style=filled]
    P3 [label="receiver.send()"]
    
    P1 -> P2 [label="Lock Mutex"]
    P2 -> P22 [label="No Data"]
    P22 -> P1 [label="yield_now()"]
    P2 -> P222 [label="Read failed"]
    P2 -> P3 [label="Valid Frame"]
  }
  
  subgraph cluster_2 {
    color=green
    label="Server loop"
    node[color=green]
    
    S1 [label="for each\nincoming", shape=Mdiamond]
    S2 [label="SPAWN\nhandle_stream()"]
    S22 [label="Drop stream"]
    S3 [label="accept_connection()"]
    
    S -> S1
    S1 -> S2 [label="valid"]
    S1 -> S22 [label="invalid"]
    S2 -> S3
    S3 -> E4 [label="valid"]
    S3 -> S22 [label="invalid"]
  }
}
```
