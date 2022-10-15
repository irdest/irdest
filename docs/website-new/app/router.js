import EmberRouter from '@ember/routing/router';
import config from 'irdest-website/config/environment';

export default class Router extends EmberRouter {
  location = config.locationType;
  rootURL = config.rootURL;
}

Router.map(function () {
  this.route('download');
  this.route('community');
  this.route('learn');
  this.route('impressum', { path: '/legal/impressum' });
});
