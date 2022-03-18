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

* `single_node.sh` starts a ratmand instance with its state directory
  set to `state/single_node`
* `single_node_recv.sh` connects to ratmand via ratcat, registers an
  address and waits to receive a message
* `single_node_send.sh` connects to ratmand via ratcat, registers an
  address and sends a message to the previously registered receiver
  address

To run this example you will need _three terminal windows_.

**In terminal A**

```console
$ ./single_node.sh
... ratmand output
```

**In terminal B**

```console
$ ./single_node_recv.sh
... bla bla
```

**In terminal C**

```console
$ ./single_node_send.sh
... bla bla
```
