// SPDX-FileCopyrightText: 2022 embr <git@liclac.eu>
//
// SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

import Model, { attr } from '@ember-data/model';
import { service } from '@ember/service';

export default class AddrModel extends Model {
  @service metrics;

  @attr isLocal;

  get metricBytesSent() {
    return this.metrics.sumRate('ratman_dispatch_bytes_total', {
      recp_id: this.id,
    });
  }

  get metricBytesRecv() {
    return this.metrics.sumRate('ratman_switch_received_bytes_total', {
      recp_id: this.id,
    });
  }
}
