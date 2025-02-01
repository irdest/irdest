import t from 'ember-intl'

<template>
  <table>
    <tr>
      <td>{{ t 'home.addrs' }}</td>
      <td>1</td>
    </tr>
    <tr>
      <td>{{ t 'home.peers' }}</td>
      <td>?</td>
    </tr>
    <tr>
      <td>{{ t 'home.spaces' }}</td>
      <td>1</td>
    </tr>
    <tr>
      <td>{{ t 'home.quota_metadb' }}</td>
      <td>16MB (no limit)</td>
    </tr>
    <tr>
      <td>{{ t 'home.quota_journal' }}</td>
      <td>712MB / 2GB</td>
    </tr>
  </table>
</template>
