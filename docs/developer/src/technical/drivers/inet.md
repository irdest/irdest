# Internet overlay netmod (inet)

The main way to use Ratman with other people at the moment is via the
_internet overlay network module_ inet.  It creates peering sessions
over the internet and TCP.  With that comes a significant amount of
connection state logic and routing outside of Ratman, because each
instance of inet can be connected with many other instances of inet.



