use crate::proxies::NetworkManager::Settings::SettingsProxy;

pub struct NMSettings<'a> {
    proxy: SettingsProxy<'a>,
}

impl<'a> NMSettings<'a> {}
