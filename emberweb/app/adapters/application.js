import RESTAdapter from '@ember-data/adapter/rest';
import { inject as service } from '@ember/service';
import { underscore } from '@ember/string';
import { singularize } from 'ember-inflector';

function serializeIntoHash(store, modelClass, snapshot, options = { includeId: true }) {
  const serializer = store.serializerFor(modelClass.modelName);

  if (typeof serializer.serializeIntoHash === 'function') {
    const data = {};
    serializer.serializeIntoHash(data, modelClass, snapshot, options);
    return data;
  }

  return serializer.serialize(snapshot, options);
}

export default class ApplicationAdapter extends RESTAdapter {
  namespace = 'http';

  @service() session;

  get headers() {
    if(this.session.isAuthenticated) {
      return {
        Authorization: JSON.stringify({
          id: this.session.data.authenticated.id,
          token: this.session.data.authenticated.token,
        }),
      }
    }

    return {};
  }

  pathForType(modelName) {
    return singularize(underscore(modelName));
  }

  async updateRecord(store, type, snapshot) {
    const data = serializeIntoHash(store, type, snapshot, {
      isUpdate: true,
    });

    let id = snapshot.id;
    let url = this.buildURL(type.modelName, id, snapshot, 'updateRecord');

    const response = await this.ajax(url, 'PATCH', { data });
    if(response === "success") {
      return undefined;
    }
    return response;
  }
}
