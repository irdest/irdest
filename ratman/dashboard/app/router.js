// SPDX-FileCopyrightText: 2022 embr <git@liclac.eu>
//
// SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

import EmberRouter from '@ember/routing/router';
import config from 'ratman-dashboard/config/environment';

export default class Router extends EmberRouter {
  location = config.locationType;
  rootURL = config.rootURL;
}

Router.map(function () {});
