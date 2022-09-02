import Controller from '@ember/controller';

export default class IndexController extends Controller {
  get addrs() {
    return this.model.addrs;
  }
  get sortedAddrs() {
    return this.addrs.sortBy('bytesTotal');
  }
  get localAddrs() {
    return this.sortedAddrs.filterBy('isLocal', true);
  }
  get remoteAddrs() {
    return this.sortedAddrs.filterBy('isLocal', false);
  }
}
