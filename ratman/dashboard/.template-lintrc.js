// SPDX-FileCopyrightText: 2022 embr <git@liclac.eu>
//
// SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

'use strict';

module.exports = {
  extends: 'recommended',
  rules: {
    // Prevent unlocalised string literals in templates, use {{t '...'}}.
    // Unlocalisable strings, like arrows, emojis, etc. can be allowlistd here.
    'no-bare-strings': [
      // TODO: These are placeholder strings until we have the metrics API working.
      '↑ 1.0kb',
      '↓ 69.0kb',
    ],
  },
};
