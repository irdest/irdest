use zbus::{export::async_trait::async_trait, Result};

use crate::{proxies::NetworkManager::AccessPoint::AccessPointProxy, NMClient};

use super::device::{FromDevice, NMDevice, NMDeviceType};

pub struct AccessPoint<'a> {
    proxy: AccessPointProxy<'a>,
}

impl AccessPoint<'_> {}

#[async_trait]
impl<'a> FromDevice for AccessPoint<'a> {
    async fn from_device(nm: NMClient<'_>, device: NMDevice) -> Result<AccessPoint<'a>> {
        match device.kind {
            NMDeviceType::AccessPoint => Ok(AccessPoint {
                proxy: AccessPointProxy::builder(nm.proxy.connection())
                    .destination(crate::DESTINATION)?
                    .path(device.path)?
                    .build()
                    .await?,
            }),
            _ => Err(zbus::Error::Unsupported),
        }
    }
}
