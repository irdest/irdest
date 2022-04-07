import Route from '@ember/routing/route';
import { service } from '@ember/service';

export default class ApplicationRoute extends Route {
  @service intl;

  beforeModel() {
    this._super(...arguments);

    this.intl.setLocale(['en']); // TODO: Look at `navigator.languages`.
  }
}
