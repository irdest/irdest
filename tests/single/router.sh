#!/usr/bin/env bash

set -xeo pipefail

# build ratmand first
cargo build --bin ratmand --all-features

# first whipe the previous state directory
rm -rf state/single-node

# re-create the state directory
mkdir -p state/single-node/router

# start ratmand with the state directory set
env XDG_DATA_HOME=state/single-node/router ../target/debug/ratmand --accept-unknown-peers --no-discovery -v trace
