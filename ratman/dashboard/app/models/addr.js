// SPDX-FileCopyrightText: 2022 embr <git@liclac.eu>
//
// SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

import Model, { attr } from '@ember-data/model';

export default class AddrModel extends Model {
  @attr isLocal;
}
