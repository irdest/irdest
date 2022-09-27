use std::collections::HashMap;

use zbus::{export::async_trait::async_trait, Result};
use zvariant::Value;

use crate::{proxies::NetworkManager::DeviceWireless::WirelessProxy, NMClient};

use super::{
    accesspoint::NMAccessPoint,
    device::{FromDevice, NMDevice, NMDeviceType},
};

pub struct NMDeviceWifi<'a> {
    proxy: WirelessProxy<'a>,
}

type ScanOptions<'a> = HashMap<&'a str, Value<'a>>;

#[async_trait]
impl<'a> FromDevice for NMDeviceWifi<'a> {
    async fn from_device(nm: &NMClient<'_>, device: &NMDevice) -> Result<NMDeviceWifi<'a>> {
        match device.kind {
            NMDeviceType::Wifi => Ok(NMDeviceWifi {
                proxy: WirelessProxy::builder(nm.proxy.connection())
                    .destination(crate::DESTINATION)?
                    .path(device.path.clone())?
                    .build()
                    .await?,
            }),
            _ => Err(zbus::Error::Unsupported),
        }
    }
}

impl<'a> NMDeviceWifi<'a> {
    ///This function requests an SSID scan from the wireless device. The callback is an
    ///asynchronous closure. The SSIDs can then be found by calling get_all_access_points.
    ///
    ///
    pub async fn request_scan(&self, options: ScanOptions<'a>) -> Result<()> {
        self.proxy.request_scan(options).await
    }

    pub async fn last_scan(&self) -> Result<i64> {
        self.proxy.last_scan().await
    }

    //PERF: Iterator is still foreign to me and Vec is easy. Lazy evaluation is probably sensible here.
    pub async fn get_all_access_points(&self) -> Result<Vec<NMAccessPoint>> {
        let paths = self.proxy.get_access_points().await?;
        let mut aps = Vec::new();
        for path in paths {
            aps.push(NMAccessPoint::new(self.proxy.connection(), path).await?);
        }
        Ok(aps)
    }
}
