# Integration tests

This directory contains integration tests (meaning tests that cover
user-facing scenarios) and example setups featuring `ratmand` and
`ratcat` in particular.

The idea is to provide a simple set of scripts to verify functionality
in a real-world setting, while also giving developers looking to work
with Irdest


## Available tests

You need to have the `jq` command installed to run these tests.

### Single-node

This test contains three test scripts:

* `single/router.sh` starts a ratmand instance with its state
  directory set to `state/single_node`
* `single/recv.sh` connects to ratmand via ratcat, registers an
  address and waits to receive a message
* `single/send.sh` connects to ratmand via ratcat, registers an
  address and sends a message to the previously registered receiver
  address

To run this example you will need _three terminal windows_.

**In terminal A**

```console
$ ./single/router.sh
... ratmand output
```

**In terminal B**

```console
$ ./single/recv.sh
... bla bla
```

**In terminal C**

```console
$ ./single/send.sh
... bla bla
```

### Multi-node


This test contains three test scripts:

* `multi/router.sh` starts two ratmand instances with their state
  directory set to `state/multi_node`.  Both routers are peering with
  each other over TCP only.  Both routers are shut down when the
  script ends.
* `multi/recv.sh` connects to the first ratmand via ratcat,
  registers an address and waits to receive a message
* `multi/send.sh` connects to the second ratmand via ratcat,
  registers an address and sends a message to the previously
  registered receiver address

To run this example you will need _three terminal windows_.

**In terminal A**

```console
$ ./multi/multi_node.sh
... ratmand output
```

**In terminal B**

```console
$ ./multi/multi_node_recv.sh
... bla bla
```

**In terminal C**

```console
$ ./multi/multi_node_send.sh
... bla bla
```
