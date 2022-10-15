'use strict';

const EmberApp = require('ember-cli/lib/broccoli/ember-app');
const MergeTrees = require('broccoli-merge-trees');
const Funnel = require('broccoli-funnel');

module.exports = function (defaults) {
  let app = new EmberApp(defaults, {
    fingerprint: {
      extensions: ['js', 'css', 'map'],
    },
    prember: {
      urls: ['/', '/download', 'community', 'learn', '/legal/impressum'],
    },
  });

  app.import('node_modules/normalize.css/normalize.css');
  app.import(
    'node_modules/@typopro/web-source-sans-pro/TypoPRO-SourceSansPro.css'
  );

  const sourceSansPro = new Funnel(
    'node_modules/@typopro/web-source-sans-pro',
    {
      destDir: 'assets',
      include: ['*.eot', '*.ttf', '*.woff'],
    }
  );

  return new MergeTrees([sourceSansPro, app.toTree()]);
};
