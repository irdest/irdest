#!/bin/sh

# SPDX-FileCopyrightText: 2022 Katharina Fey <kookie@spacekookie.de>
#
# SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

## Load the setup hook
export IRDEST_TEST_DIR=$(realpath "$(dirname "$0")")
source $IRDEST_TEST_DIR/0-setup.sh

mkdir "$IRDEST_TEST_DIR/.data/two"
XDG_DATA_HOME=$IRDEST_TEST_DIR/.data/two \
    $IRDEST_BIN_DIR/ratmand --config "$IRDEST_TEST_DIR/.data/cfg-two.kdl" \
    generate \
    -p "ratmand/verbosity=debug" \
    -p "ratmand/api_bind=127.0.0.1:9992" \
    -p "ratmand/enable_dashboard=false" \
    -p "ratmand/ephemeral=true" \
    -p "inet/bind=[::]:5552" \
    -p "lan/enable=false" \
    --add-peer "inet:[::]:5860"

XDG_DATA_HOME=$IRDEST_TEST_DIR/.data/two \
    $IRDEST_BIN_DIR/ratmand -v debug --config "$IRDEST_TEST_DIR/.data/cfg-two.kdl" &
echo "$!" > $IRDEST_TEST_DIR/.data/two.pid
