// SPDX-FileCopyrightText: 2022 embr <git@liclac.eu>
//
// SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

'use strict';

module.exports = {
  extends: 'recommended',
  rules: {
    // Prevent string literals in templates; use localisation placeholders!
    'no-bare-strings': true,
  },
};
