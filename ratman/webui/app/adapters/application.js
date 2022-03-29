import RESTAdapter from '@ember-data/adapter/rest';
import ENV from 'webui/config/environment';

export default class ApplicationAdapter extends RESTAdapter {
  namespace = 'v1';
  host = ENV.apiHost;
}
