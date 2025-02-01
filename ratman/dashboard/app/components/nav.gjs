import { LinkTo } from '@ember/routing';
import { t } from 'ember-intl';

<template>
  <div class="navbar">
    <LinkTo @route="index">{{ t 'navigation.home' }}</LinkTo>
    <LinkTo @route="peers">{{ t 'navigation.peers' }}</LinkTo>
    <LinkTo @route="neighbours">{{ t 'navigation.neighbours' }}</LinkTo>
    <LinkTo @route="spaces">{{ t 'navigation.spaces' }}</LinkTo>
    <LinkTo @route="api">{{ t 'navigation.api' }}</LinkTo>
  </div>
</template>
