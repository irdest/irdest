// SPDX-FileCopyrightText: 2022 embr <git@liclac.eu>
//
// SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

import { module, test } from 'qunit';
import { setupTest } from 'ember-qunit';

module('Unit | Route | application', function (hooks) {
  setupTest(hooks);

  test('it exists', function (assert) {
    let route = this.owner.lookup('route:application');
    assert.ok(route);
  });

  module('detects browser language', function () {
    for (let { lang, langs, loc, name } of [
      { lang: 'en', langs: undefined, loc: 'en' },
      { lang: 'en-GB', langs: undefined, loc: 'en' },
      { lang: 'en', langs: [], loc: 'en' },
      { lang: 'en-GB', langs: [], loc: 'en' },
      { lang: 'en', langs: ['en'], loc: 'en' },
      { lang: 'en-GB', langs: ['en'], loc: 'en' },
      { lang: 'en-GB', langs: ['en-GB', 'en'], loc: 'en' },
      { lang: 'en-GB', langs: ['en-GB', 'en', 'de-AT', 'de'], loc: 'en' },
      {
        lang: 'en-GB',
        langs: ['it-CH', 'it', 'en-GB', 'en', 'de-AT', 'de'],
        loc: 'en',
      },
      {
        lang: 'en-GB',
        langs: ['it-CH', 'it', 'en-GB', 'en', 'de-AT', 'de'],
        loc: 'en',
      },

      { lang: 'de', langs: undefined, loc: 'de' },
      { lang: 'de-AT', langs: undefined, loc: 'de' },
      { lang: 'de', langs: [], loc: 'de' },
      { lang: 'de-AT', langs: [], loc: 'de' },
      { lang: 'de', langs: ['de'], loc: 'de' },
      { lang: 'de-AT', langs: ['de-AT'], loc: 'de' },
      { lang: 'de-AT', langs: ['de-AT', 'de'], loc: 'de' },
      { lang: 'de-AT', langs: ['de-AT', 'de', 'en-GB', 'en'], loc: 'de' },
      { lang: 'it-CH', langs: ['it-CH', 'it', 'de-AT', 'de'], loc: 'de' },
      {
        lang: 'it-CH',
        langs: ['it-CH', 'it', 'de-AT', 'de', 'en-GB', 'en'],
        loc: 'de',
      },

      // Monolingual klingon.
      {
        lang: 'tlh',
        langs: ['tlh'],
        loc: 'en',
        name: 'no supported language',
      },

      // Old or misbehaving browser, no language information available at all.
      {
        lang: undefined,
        langs: undefined,
        loc: 'en',
        name: 'no language indicated',
      },
    ]) {
      module(name || `lang=${lang}, langs=${langs}`, function (hooks) {
        hooks.before(function () {
          Object.defineProperty(window.navigator, 'language', {
            configurable: true,
            value: lang,
          });
          Object.defineProperty(window.navigator, 'languages', {
            configurable: true,
            value: langs,
          });
        });

        test(`it uses ${loc}`, function (assert) {
          this.owner.lookup('route:application').setupIntl();
          let intl = this.owner.lookup('service:intl');
          assert.deepEqual(intl.get('locale'), [loc]);
          assert.deepEqual(intl.get('primaryLocale'), loc);
        });
      });
    }
  });
});
