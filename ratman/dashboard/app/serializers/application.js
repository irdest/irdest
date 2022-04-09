// SPDX-FileCopyrightText: 2022 embr <git@liclac.eu>
//
// SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

import RESTSerializer from '@ember-data/serializer/rest';
import { underscore } from '@ember/string';

export default class ApplicationSerializer extends RESTSerializer {
  keyForAttribute(attr) {
    return underscore(attr);
  }
}
