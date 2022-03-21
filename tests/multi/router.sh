#!/usr/bin/env bash

set -xeo pipefail

# build ratmand first
cargo build --bin ratmand --all-features

# first whipe the previous state directory
rm -rf state/multi-node

# re-create the state directory
mkdir -p state/multi-node/{router1,router2}

# start ratmand with the state directory set
env XDG_DATA_HOME=state/multi-node/router1 ../target/debug/ratmand --inet '127.0.0.1:9000' -p 'inet#127.0.0.1:7000' --no-discovery -v trace &
env XDG_DATA_HOME=state/multi-node/router2 ../target/debug/ratmand --inet '127.0.0.1:7000' -p 'inet#127.0.0.1:9000' --no-discovery -v trace -b '127.0.0.1:7020'
