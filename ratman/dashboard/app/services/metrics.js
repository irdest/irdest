import Service from '@ember/service';
import { later } from '@ember/runloop';
import { tracked } from '@glimmer/tracking';
import fetch from 'fetch';
import parsePrometheusTextFormat from 'parse-prometheus-text-format';

export default class MetricsService extends Service {
  intervalSecs = 5;

  @tracked oldMetrics = null;
  @tracked metrics = null;

  start() {
    fetch('/_/metrics').then((response) => {
      if (response.ok) {
        response.text().then((text) => {
          this.parse(text);
        });
      } else {
        console.error("Couldn't fetch metrics", response);
      }
      later(this, this.start, this.intervalSecs * 1000);
    });
  }

  parse(text) {
    this.oldMetrics = this.metrics;
    this.metrics = parsePrometheusTextFormat(text).reduce((metrics, metric) => {
      metric.old = metrics[metric.name];
      metrics[metric.name] = metric;
      return metrics;
    }, {});
  }

  find(name, labels, _metrics) {
    let metric = (_metrics || this.metrics)[name];
    if (!metric) {
      return [];
    }
    return metric.metrics.filter((sample) => {
      for (let key in labels) {
        if (sample.labels[key] !== labels[key]) {
          return false;
        }
      }
      return true;
    });
  }

  // Return the sum of all metrics with the given name and labels.
  sum(name, labels, _metrics) {
    return this.find(name, labels, _metrics).reduce((sum, sample) => {
      return sum + parseInt(sample.value);
    }, 0);
  }

  // Return the rate at which a sum() call increased between the last two ticks.
  sumRate(name, labels) {
    if (this.oldMetrics === null) {
      return 0;
    }
    return (
      (this.sum(name, labels, this.metrics) -
        this.sum(name, labels, this.oldMetrics)) /
      this.intervalSecs
    );
  }
}
