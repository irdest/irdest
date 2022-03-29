import RESTSerializer from '@ember-data/serializer/rest';
import { underscore } from '@ember/string';

export default class ApplicationSerializer extends RESTSerializer {
  keyForAttribute(attr) {
    return underscore(attr);
  }
}
