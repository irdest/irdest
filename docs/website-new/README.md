# irdest-website

## Prerequisites

You will need the following things properly installed on your computer.

* [Git](https://git-scm.com/)
* [Node.js](https://nodejs.org/) (with npm)
* [Ember CLI](https://cli.emberjs.com/release/)
* [Google Chrome](https://google.com/chrome/)
* [NixOS](https://nixos.org/)

## Installation

* `nix-shell`
* `npm install`

## Running / Development

* `npx ember serve`
* Visit your app at [http://localhost:4200](http://localhost:4200).
* Visit your tests at [http://localhost:4200/tests](http://localhost:4200/tests).

### Code Generators

Make use of the many generators for code, try `npx ember help generate` for more details

### Running Tests

* `npx ember test`
* `npx ember test --server`

### Linting

* `npm run lint`
* `npm run lint:fix`

### Building

* `npx ember build` (development)
* `npx ember build --environment production` (production)

### Deploying

We have a nix derivation. So just do `nix-build nix -A irdest-website` in the repo root.

## Further Reading / Useful Links

* [ember.js](https://emberjs.com/)
* [ember-cli](https://cli.emberjs.com/release/)
* Development Browser Extensions
  * [ember inspector for chrome](https://chrome.google.com/webstore/detail/ember-inspector/bmdblncegkenkacieihfhpjfppoconhi)
  * [ember inspector for firefox](https://addons.mozilla.org/en-US/firefox/addon/ember-inspector/)
