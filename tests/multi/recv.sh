#!/bin/sh

# build ratmand first
cargo build --bin ratcat --all-features

# re-create the state directory
mkdir -p state/single-node/recv

# register ratcat address and then set to receive a message
env XDG_CONFIG_HOME=state/multi-node/recv ../target/debug/ratcat --register -b '127.0.0.1:7020'
cat state/multi-node/recv/config | jq .addr | tr -d '"' | tee > state/multi-node/recv-addr

env XDG_CONFIG_HOME=state/multi-node/recv ../target/debug/ratcat --recv --count 1 -b '127.0.0.1:7020'
