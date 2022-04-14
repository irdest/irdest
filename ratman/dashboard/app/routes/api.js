import Route from '@ember/routing/route';

export default class ApiRoute extends Route {
  model() {
    return {
      url: '/api/v1/openapi.json',
    };
  }
}
