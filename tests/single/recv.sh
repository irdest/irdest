#!/usr/bin/env bash

set -xeo pipefail

# build ratmand first
cargo build --bin ratcat --all-features

# re-create the state directory
mkdir -p state/single-node/recv

# register ratcat address and then set to receive a message
env XDG_CONFIG_HOME=state/single-node/recv ../target/debug/ratcat --register
cat state/single-node/recv/config | jq .addr | tr -d '"' | tee > state/single-node/recv-addr

env XDG_CONFIG_HOME=state/single-node/recv ../target/debug/ratcat --recv --count 1

# After the message has been received we shut down ratmand
# todo hide this behind a flag
pkill ratmand
