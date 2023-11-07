#!/bin/sh

# SPDX-FileCopyrightText: 2023 Katharina Fey <kookie@spacekookie.de>
#
# SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

## Load the setup hook
export IRDEST_TEST_DIR=$(realpath "$(dirname "$0")")
source $IRDEST_TEST_DIR/0-setup.sh

set -e

$IRDEST_BIN_DIR/ratcat -b "127.0.0.1:9991" \
                       --register \
                       --config $IRDEST_TEST_DIR/.data/one-ratcat.cfg \
                       2>/dev/null > $IRDEST_TEST_DIR/.data/recv-addr

sleep 3

$IRDEST_BIN_DIR/ratcat -b "127.0.0.1:9991" \
                       --config $IRDEST_TEST_DIR/.data/one-ratcat.cfg \
                       --count 1 --recv > $IRDEST_TEST_DIR/.data/recv-message

echo "HEY WE GOT A MESSAGE!! UwU"
echo "IT'S $(cat $IRDEST_TEST_DIR/.data/recv-message)"
echo "TEARING IT ALL DOWN AGAIN..."
echo "=================================================="
sleep 2

$IRDEST_TEST_DIR/zzz-cleanup.sh
echo "Test complete!"
echo "(press ENTER to reset your terminal)"
