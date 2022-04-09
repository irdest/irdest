// SPDX-FileCopyrightText: 2022 embr <git@liclac.eu>
//
// SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

import { module, test } from 'qunit';
import { setupTest } from 'ember-qunit';

module('Unit | Model | addr', function (hooks) {
  setupTest(hooks);

  // Replace this with your real tests.
  test('it exists', function (assert) {
    let store = this.owner.lookup('service:store');
    let model = store.createRecord('addr', {});
    assert.ok(model);
  });
});
