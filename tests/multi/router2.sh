#!/bin/sh

# SPDX-FileCopyrightText: 2022 Katharina Fey <kookie@spacekookie.de>
#
# SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

# start ratmand with the state directory set
env XDG_DATA_HOME=state/multi-node/router2 ../target/debug/ratmand --inet '[::0]:7000' -p 'inet#[::1]:9000' --no-discovery -v trace -b '127.0.0.1:7020' --no-webui
