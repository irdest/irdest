import Route from '@ember/routing/route';

export default class ApiRoute extends Route {
  model() {
    return {
      url: '/api/v1/openapi.json',

      // Set URL hash when expanding items, allows direct links: /api#/addr/getAddrs
      deepLinking: true,
    };
  }
}
