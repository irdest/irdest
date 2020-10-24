import JSONSerializer from '@ember-data/serializer/json';
import { pluralize } from 'ember-inflector';
import { underscore } from '@ember/string';

export default class ApplicationSerializer extends JSONSerializer {
  keyForAttribute(attr) {
    return underscore(attr);
  }
  normalizeSingleResponse (store, primaryModelClass, payload, id, requestType) {
    // we convert from { user: [ { ...data... } ] } to { ...data... }
    return super.normalizeSingleResponse(store, primaryModelClass, payload[underscore(primaryModelClass.modelName)], id, requestType);
  }
  normalizeArrayResponse(store, primaryModelClass, payload, id, requestType) {
    const primary = payload[underscore(pluralize(primaryModelClass.modelName))];

    return super.normalizeArrayResponse(store, primaryModelClass, primary, id, requestType);
  }

  serialize(snapshot, options) {
    let json = {};

    if (options && options.includeId) {
      const id = snapshot.id;
      if (id) {
        json[this.primaryKey] = id;
      }
    }

    const changedAttributes = Object.keys(snapshot.changedAttributes());
    snapshot.eachAttribute((key, attribute) => {
      if(changedAttributes.includes(key)) {
        this.serializeAttribute(snapshot, json, key, attribute, options);
      }
    });

    snapshot.eachRelationship((key, relationship) => {
      if (relationship.kind === 'belongsTo') {
        this.serializeBelongsTo(snapshot, json, relationship);
      } else if (relationship.kind === 'hasMany') {
        this.serializeHasMany(snapshot, json, relationship);
      }
    });

    return json;
  }

  serializeAttribute(snapshot, json, key, attribute, options) {
    super.serializeAttribute(...arguments);

    // we need to wrap out data in a { set: }
    if(options.isUpdate) {
      const serializedKey = this.keyForAttribute(key);
      json[serializedKey] = { set: json[serializedKey] };
    }
  }
}
