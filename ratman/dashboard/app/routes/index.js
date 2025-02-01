// SPDX-FileCopyrightText: 2022 embr <git@liclac.eu>
// SPDX-FileCopyrightText: 2025 Katharina Fey <kookie@spacekookie.de>
//
// SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

import Route from '@ember/routing/route';
import { service } from '@ember/service';
import RSVP from 'rsvp';

export default class IndexRoute extends Route {
  @service store;
  // @service metrics;

  async model() {
	  const addrs = await this.store.requestManager.request({
      url: "/api/v1/addrs"
	  });

    const peers = await this.store.requestManager.request({
      url: "/api/v1/peers"
    });

    const neighbours = await this.store.requestManager.request({
      url: "/api/v1/neighbours"
    });

	  //return { addrs: addrs.content, peers: peers.content, neighbours: neighbours.content };
    return addrs.content;
  }
}
