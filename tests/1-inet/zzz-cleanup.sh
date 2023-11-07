#!/bin/sh

# SPDX-FileCopyrightText: 2023 Katharina Fey <kookie@spacekookie.de>
#
# SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

## Make sure the directory exists before we try to run an rm -r ...
export IRDEST_TEST_DIR=$(realpath "$(dirname "$0")")
source $IRDEST_TEST_DIR/0-setup.sh

kill $(cat $IRDEST_TEST_DIR/.data/two.pid)
kill $(cat $IRDEST_TEST_DIR/.data/one.pid)

## Do NOT add -f for _some_ failsafe protection
rm -r $IRDEST_TEST_DIR/.data
unset $IRDEST_TEST_DIR
unset $IRDEST_BIN_DIR
