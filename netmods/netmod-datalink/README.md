<!--
SPDX-FileCopyrightText: 2019-2021 Katharina Fey <kookie@spacekookie.de>
SPDX-FileCopyrightText: 2022 Christopher Grant <grantchristophera@gmail.com>

SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore
-->

# netmod-raw

The netmod-raw endpoint can be used for local communication over ethernet, but it includes
components meant for configuring over 802.11 (which uses ethernet packets by default).

Several features are not supported, including jumbo frames which are usually supported in wireless
data packets.

Please note that permissions are required for accessing ethernet packets directly. This endpoint
also currently requires a dedicated physical endpoint.
