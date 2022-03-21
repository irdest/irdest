#!/usr/bin/env bash

set -xeo pipefail

# build ratmand first
cargo build --bin ratcat --all-features

# re-create the state directory
mkdir -p state/single-node/send

# register ratcat address and then set to receive a message
env XDG_CONFIG_HOME=state/single-node/send ../target/debug/ratcat --register

RECV_ADDR=$(cat state/single-node/recv-addr)

# send a message to the previously (hopefully lol) registered address
env XDG_CONFIG_HOME=state/single-node/send ../target/debug/ratcat $RECV_ADDR "HELLO WORLD!"
