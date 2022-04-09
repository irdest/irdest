#!/bin/sh

# SPDX-FileCopyrightText: 2022 Katharina Fey <kookie@spacekookie.de>
#
# SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

# build ratmand first
cargo build --bin ratmand --all-features

# first whipe the previous state directory
rm -rf state/multi-node

# re-create the state directory
mkdir -p state/multi-node/{router1,router2}

# start ratmand with the state directory set
env XDG_DATA_HOME=state/multi-node/router1 ../target/debug/ratmand --inet '[::0]:9000' --accept-unknown-peers --no-discovery -v trace --no-webui
