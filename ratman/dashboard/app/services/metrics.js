import Service from '@ember/service';
import { later } from '@ember/runloop';
import fetch from 'fetch';

export default class MetricsService extends Service {
  start() {
    fetch('/_/metrics').then((response) => {
      if (response.ok) {
        response.text().then((text) => {
          console.log(text);
        });
      } else {
        console.error("Couldn't fetch metrics", response);
      }
    });
    later(this, this.start, 1000);
  }
}
