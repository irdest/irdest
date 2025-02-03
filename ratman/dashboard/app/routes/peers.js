// SPDX-FileCopyrightText: 2025 Katharina Fey <kookie@spacekookie.de>
//
// SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

import Route from '@ember/routing/route';
import { service } from '@ember/service';
import RSVP from 'rsvp';

export default class PeersRoute extends Route {
    @service store;

  async model() {
	  const res = await this.store.requestManager.request({
      url: "/api/v1/peers"
	  });

	  return res.content;
  }
}
