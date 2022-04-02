import Route from '@ember/routing/route';
import { service } from '@ember/service';
import RSVP from 'rsvp';

export default class IndexRoute extends Route {
  @service store;

  model() {
    return RSVP.hash({
      addrs: this.store.findAll('addr'),
    });
  }
}
