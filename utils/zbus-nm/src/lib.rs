use std::collections::HashMap;

use zbus::{Connection, Result};
use zvariant::{OwnedObjectPath, OwnedValue, Value};

mod proxies;

use crate::proxies::NetworkManager::NetworkManagerProxy;
use crate::proxies::NetworkManager::Settings::SettingsProxy;

pub mod devices;

use crate::devices::device::NMDevice;

pub mod settings;

use settings::{NMActiveConnection, NMConnection, NMSettings, PartialConnection};

const DESTINATION: &str = "org.freedesktop.NetworkManager";
const SETTINGS_PATH: &str = "/org/freedesktop/NetworkManager/Settings";

pub struct NMClient<'a> {
    pub(crate) proxy: NetworkManagerProxy<'a>,
    pub settings: NMSettings<'a>,
}

pub type Options<'a> = HashMap<&'a str, Value<'a>>;

impl<'a> NMClient<'a> {
    pub async fn new(system_connection: &Connection) -> Result<NMClient<'a>> {
        let nm_proxy = NetworkManagerProxy::new(system_connection).await?;

        Ok(NMClient {
            proxy: nm_proxy,
            settings: NMSettings {
                proxy: SettingsProxy::builder(system_connection)
                    .destination(DESTINATION)?
                    .path(SETTINGS_PATH)?
                    .build()
                    .await?,
            },
        })
    }

    //PERF: Iterator is still foreign to me and Vec is easy. Lazy evaluation is probably sensible here.
    pub async fn get_all_devices(&self) -> Result<Vec<NMDevice>> {
        let reply = self.proxy.get_all_devices().await?;

        let mut devices = Vec::new();

        for path in reply {
            if let Ok(device) =
                NMDevice::from_owned_object_path(&self.proxy.connection(), path).await
            {
                devices.push(device);
            }
        }

        Ok(devices)
    }

    pub async fn get_device_by_iface(&self, iface: &str) -> Result<NMDevice<'_>> {
        Ok(NMDevice::from_owned_object_path(
            &self.proxy.connection(),
            self.proxy.get_device_by_ip_iface(iface).await?,
        )
        .await?)
    }

    pub async fn add_and_activate_connection(
        &self,
        connection: PartialConnection<'_>,
        device: NMDevice<'_>,
        specific_object: OwnedObjectPath,
    ) -> Result<(NMConnection, NMActiveConnection)> {
        let reply = self
            .proxy
            .add_and_activate_connection(connection, &device.path, &specific_object)
            .await?;
        Ok((
            NMConnection::new(&self.proxy.connection(), reply.0).await?,
            NMActiveConnection::new(&self.proxy.connection(), reply.1).await?,
        ))
    }

    pub async fn add_and_activate_connection2(
        &self,
        connection: PartialConnection<'_>,
        device: &NMDevice<'_>,
        specific_object: OwnedObjectPath,
        options: Options<'_>,
    ) -> Result<(
        NMConnection,
        NMActiveConnection,
        HashMap<String, OwnedValue>,
    )> {
        let reply = self
            .proxy
            .add_and_activate_connection2(connection, &device.path, &specific_object, options)
            .await?;
        Ok((
            NMConnection::new(&self.proxy.connection(), reply.0).await?,
            NMActiveConnection::new(&self.proxy.connection(), reply.1).await?,
            reply.2,
        ))
    }
}
