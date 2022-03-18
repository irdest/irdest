# Integration tests

This directory contains integration tests (meaning tests that cover
user-facing scenarios) and example setups featuring `ratmand` and
`ratcat` in particular.

The idea is to provide a simple set of scripts to verify functionality
in a real-world setting, while also giving developers looking to work
with Irdest


## Available tests

### Single-node

This test contains three test scripts:

* `single_node.sh` starts a ratmand instance with its state directory set to `state/single_node`

Start a single ratmand instance, 
