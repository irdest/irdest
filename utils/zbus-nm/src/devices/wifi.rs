use std::collections::HashMap;
use std::future::Future;

use zbus::{
    export::{async_trait::async_trait, futures_util::StreamExt},
    Result,
};
use zvariant::Value;

use crate::{proxies::NetworkManager::DeviceWireless::WirelessProxy, NMClient};

use super::{
    accesspoint::AccessPoint,
    device::{FromDevice, NMDevice, NMDeviceType},
};

struct Wifi<'a> {
    proxy: WirelessProxy<'a>,
}

type ScanOptions<'a> = HashMap<&'a str, Value<'a>>;

#[async_trait]
impl<'a> FromDevice for Wifi<'a> {
    async fn from_device(nm: NMClient<'_>, device: NMDevice) -> Result<Wifi<'a>> {
        match device.kind {
            NMDeviceType::Wifi => Ok(Wifi {
                proxy: WirelessProxy::builder(nm.proxy.connection())
                    .destination(crate::DESTINATION)?
                    .path(device.path)?
                    .build()
                    .await?,
            }),
            _ => Err(zbus::Error::Unsupported),
        }
    }
}

impl<'a> Wifi<'a> {
    ///This function requests an SSID scan from the wireless device. The callback is an
    ///asynchronous closure. The SSIDs can then be found by calling get_all_access_points.
    ///
    ///
    pub async fn request_scan<F, C>(&self, options: ScanOptions<'a>, callback: C) -> Result<()>
    where
        F: Future<Output = ()>,
        C: FnOnce() -> F,
    {
        self.proxy.request_scan(options).await;

        let current_stamp = self.proxy.receive_last_scan_changed().await.next().await;

        match current_stamp {
            Some(_) => {
                callback().await;
                Ok(())
            }
            None => Err(zbus::Error::InvalidReply), // This really should not ever occur.
        }
    }

    pub async fn get_all_access_points(&self) -> Result<Vec<AccessPoint>> {
        let aps = self.proxy.get_access_points().await?;
        Err(zbus::Error::Unsupported)
    }
}
