#!/bin/sh

# SPDX-FileCopyrightText: 2022 Katharina Fey <kookie@spacekookie.de>
#
# SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

export IRDEST_TEST_DIR=$(realpath "$(dirname "$0")")
source $IRDEST_TEST_DIR/0-setup.sh

echo "Starting Ratmand instance one (the 'server')"
$IRDEST_TEST_DIR/one.sh &
sleep 3
echo "========== SLEEP 3 SECONDS TO LET THE ROUTER START"

echo "Starting Ratmand instance two (the 'connector')"
$IRDEST_TEST_DIR/two.sh &
sleep 3
echo "========== SLEEP 3 SECONDS TO LET THE ROUTER START"

pgrep ratmand -l -a

$IRDEST_TEST_DIR/one-recv.sh &

echo "========== SLEEP 5 SECONDS TO LET THE RECEIVER START"
sleep 5

echo "========== SEND TEST PAYLOAD ... "
$IRDEST_TEST_DIR/two-send.sh

