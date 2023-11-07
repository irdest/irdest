#!/bin/sh

# SPDX-FileCopyrightText: 2022 Katharina Fey <kookie@spacekookie.de>
#
# SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

## Load the setup hook
export IRDEST_TEST_DIR=$(realpath "$(dirname "$0")")
source $IRDEST_TEST_DIR/0-setup.sh

mkdir "$IRDEST_TEST_DIR/.data/one"
XDG_DATA_HOME=$IRDEST_TEST_DIR/.data/one \
    $IRDEST_BIN_DIR/ratmand --config "$IRDEST_TEST_DIR/.data/cfg-one.kdl" \
    generate \
    -p "ratmand/verbosity=trace" \
    -p "ratmand/api_bind=127.0.0.1:9991" \
    -p "ratmand/enable_dashboard=false" \
    -p "ratmand/accept_unknown_peers=true" \
    -p "ratmand/ephemeral=true" \
    -p "lan/enable=false"

XDG_DATA_HOME=$IRDEST_TEST_DIR/.data/one \
    $IRDEST_BIN_DIR/ratmand -v debug --config "$IRDEST_TEST_DIR/.data/cfg-one.kdl" &
echo "$!" > $IRDEST_TEST_DIR/.data/one.pid
