import Model, { attr } from '@ember-data/model';

export default class AddrModel extends Model {
  @attr('boolean') isLocal;
}
