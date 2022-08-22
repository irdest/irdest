import Service from '@ember/service';
import { later } from '@ember/runloop';
import fetch from 'fetch';
import parsePrometheusTextFormat from 'parse-prometheus-text-format';

export default class MetricsService extends Service {
  metrics = {};

  start() {
    fetch('/_/metrics').then((response) => {
      if (response.ok) {
        response.text().then((text) => {
          this.parse(text);
        });
      } else {
        console.error("Couldn't fetch metrics", response);
      }
      later(this, this.start, 10 * 1000); // Every 10s for now.
    });
  }

  parse(text) {
    this.metrics = parsePrometheusTextFormat(text).reduce((metrics, metric) => {
      metrics[metric.name] = metric;
      return metrics;
    }, {});
  }

  sum(metricName, labels) {
    let metric = this.metrics[metricName];
    if (!metric) {
      return 0;
    }
    return metric.metrics.reduce((sum, sample) => {
      for (let key in labels) {
        if (sample.labels[key] !== labels[key]) {
          return sum;
        }
      }
      return sum + parseInt(sample.value);
    }, 0);
  }
}
