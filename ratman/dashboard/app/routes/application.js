import Route from '@ember/routing/route';
import { service } from '@ember/service';

export default class ApplicationRoute extends Route {
  @service intl;
  @service bestLanguage;

  beforeModel() {
    super.beforeModel(...arguments);
    this.setupIntl();
  }

  // Try to set a locale based on browser settings.
  setupIntl() {
    let locales = this.intl.get('locales');
    let best = this.bestLanguage.bestLanguage(locales);
    if (best) {
      console.log('🏴️ Using browser language:', best);
      this.intl.setLocale(best.language);
    } else {
      console.warn(
        '🏳 No localisation for',
        navigator.languages,
        'only',
        locales
      );
      this.intl.setLocale('en');
    }
  }
}
