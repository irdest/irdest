import Route from '@ember/routing/route';
import Component from '@glimmer/component';
import { setRouteComponent } from 'experimental-set-route-component';
import { inject as service } from '@ember/service';

class ApplicationRoute extends Route {}

class ApplicationComponent extends Component {
  @service router;

  get displayLongTitle() {
    return this.router.currentRouteName === 'index';
  }

  <template>
    <header class="block">
      <div class="wrap">

        {{!-- top logo & nav bar --}}
        <div class="navbar">
          <span class="logo"><a href="/" class="title-link"><img width="128px" src="/img/logo.svg" /></a></span>
          <div class="navbar-links">
            <a class="navbar-link" href="/download">Download</a>
            <a class="navbar-link" href="/community">Community</a>
            <a class="navbar-link" href="https://lists.irde.st/archives/list/announce@lists.irde.st/latest">Announcements</a>
            <a class="navbar-link" href="/learn">Learn</a>
          </div>
        </div>

        {{#if this.displayLongTitle}}
          <div class="title">
            <h1>Research project for decentralised communication</h1>
            <p><strong>Irdest</strong> is making the tools for the next internet more accessible</p>
          </div>
        {{/if}}
      </div>
    </header>

    {{outlet}}

    <footer class="block">
      <div class="wrap">
        <div class="navbar">
          <div class="navbar-links">
            <a class="navbar-link" href="/legal/impressum">Impressum</a>
          </div>
        </div>
      </div>
    </footer>
  </template>
}

export default setRouteComponent(ApplicationComponent, ApplicationRoute);
