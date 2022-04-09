#!/usr/bin/env bash

# SPDX-FileCopyrightText: 2022 Katharina Fey <kookie@spacekookie.de>
#
# SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

set -xeo pipefail

# build ratmand first
cargo build --bin ratmand --all-features

# first whipe the previous state directory
rm -rf state/single-node

# re-create the state directory
mkdir -p state/single-node/router

# start ratmand with the state directory set
env XDG_DATA_HOME=state/single-node/router ../target/debug/ratmand --accept-unknown-peers --no-discovery -v trace
