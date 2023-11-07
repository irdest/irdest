#!/bin/sh

# SPDX-FileCopyrightText: 2023 Katharina Fey <kookie@spacekookie.de>
#
# SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

export IRDEST_BIN_DIR=$(realpath $IRDEST_TEST_DIR/../../target/debug)

cargo build --bin ratmand
cargo build --bin ratcat

mkdir $IRDEST_TEST_DIR/.data
