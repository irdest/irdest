#!/bin/sh

# SPDX-FileCopyrightText: 2023 Katharina Fey <kookie@spacekookie.de>
#
# SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

## Load the setup hook
export IRDEST_TEST_DIR=$(realpath "$(dirname "$0")")
source $IRDEST_TEST_DIR/0-setup.sh

set -e

$IRDEST_BIN_DIR/ratcat -b "127.0.0.1:9992" \
                       --register \
                       --config $IRDEST_TEST_DIR/.data/two-ratcat.cfg

sleep 3

$IRDEST_BIN_DIR/ratcat -b "127.0.0.1:9992" \
                       --config $IRDEST_TEST_DIR/.data/two-ratcat.cfg \
                       "$(cat $IRDEST_TEST_DIR/.data/recv-addr)" \
                       "A message from me (two-send.sh) to you (one-recv.sh).  Good day!"
