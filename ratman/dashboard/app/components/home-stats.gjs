import Component from '@glimmer/component';
import { t } from 'ember-intl'


class HomeStatsRows extends Component {
  get addrs_count() {
    return this.args.addrs.length || 0;
  }

  get peers_count() {
    return this.args.peers.length || 0;
  }

  get spaces_count() {
    return this.args.spaces.length || 0;
  }

  get neighbours_count() {
    return this.args.neighbours.length || 0;
  }

  get journal_quota() {
    return this.args.quota.journal_quota || "no limit";
  }

  get journal_usage() {
    return this.args.quota.journal_usage;
  }

  get meta_quota() {
    return this.args.quota.meta_quota || "no limit";
  }

  get meta_usage() {
    debugger;
    return this.args.quota.meta_usage;
  }

  <template>
    <tr>
      <td>{{ t 'home.addrs' }}</td>
      <td>{{ this.addrs_count }}</td>
    </tr>
    <tr>
      <td>{{ t 'home.peers' }}</td>
      <td>{{ this.peers_count }}</td>
    </tr>
    <tr>
      <td>{{ t 'home.spaces' }}</td>
      <td>{{ this.spaces_count }}</td>
    </tr>
    <tr>
      <td>{{ t 'home.neighbours' }}</td>
      <td>{{ this.neighbours_count }}</td>
    </tr>
    <tr>
      <td>{{ t 'home.quota_metadb' }}</td>
      <td>{{ this.meta_usage }} MB ({{ this.meta_quota }})</td>
    </tr>
    <tr>
      <td>{{ t 'home.quota_journal' }}</td>
      <td>{{ this.journal_usage }} MB ({{ this.journal_quota }})</td>
    </tr>
  </template>
}

<template>
  <table>
    <HomeStatsRows
      @addrs={{ @model.addrs }}
      @peers={{ @model.peers }}
      @spaces={{ @model.spaces }}
      @neighbours={{ @model.neighbours }}
      @quota={{ @model.quota }}
    />
  </table>
</template>
