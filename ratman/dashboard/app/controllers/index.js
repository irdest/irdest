import Controller from '@ember/controller';

export default class IndexController extends Controller {
  get addrs() {
    return this.model.addrs;
  }
  get sortedAddrs() {
    return this.addrs.sortBy('isLocal', 'bytesTotal').reverse();
  }
}
