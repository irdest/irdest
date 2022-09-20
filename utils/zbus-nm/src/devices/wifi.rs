use std::collections::HashMap;
use std::future::Future;

use zbus::{
    export::{async_trait::async_trait, futures_util::StreamExt},
    Result,
};
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
    pub async fn request_scan<F, C>(&self, options: ScanOptions<'a>, callback: C) -> Result<()>
    where
        F: Future<Output = ()>,
        C: FnOnce() -> F,
    {
        self.proxy.request_scan(options).await.unwrap();

        let current_stamp = self.proxy.receive_last_scan_changed().await.next().await;

        match current_stamp {
            Some(_) => {
                callback().await;
                Ok(())
            }
            None => Err(zbus::Error::InvalidReply), // This really should not ever occur.
        }
    }

    //NOTE: Rust's Iterator is still foreign to me, but lazy evaluation is probably sensible here.
    pub async fn get_all_access_points(&self) -> Result<Vec<NMAccessPoint>> {
        let paths = self.proxy.get_access_points().await?;
        let mut aps = Vec::new();
        for path in paths {
            aps.push(NMAccessPoint::new(self.proxy.connection(), path).await?);
        }
        Ok(aps)
    }
}
