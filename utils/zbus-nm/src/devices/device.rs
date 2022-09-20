use std::str::FromStr;
use zbus::fdo::IntrospectableProxy;
use zbus::xml::Node;
use zbus::Connection;
use zbus::{export::async_trait::async_trait, Result};
use zvariant::OwnedObjectPath;

use crate::{proxies::NetworkManager::Device::DeviceProxy, NMClient};

const NM_OBJ_STR_WIRELESS: &str = "org.freedesktop.NetworkManager.Device.Wireless";
const NM_OBJ_STR_WIFIP2P: &str = "org.freedesktop.NetworkManager.Device.WifiP2P";

#[async_trait]
pub trait FromDevice {
    async fn from_device(nm: &NMClient<'_>, device: &NMDevice) -> Result<Self>
    where
        Self: Sized;
}

/// TODO: Complete device type enum.
#[derive(Clone, Copy, Debug)]
pub(crate) enum NMDeviceType {
    Wifi,
    WifiP2P,
}

impl FromStr for NMDeviceType {
    type Err = ();

    /// TODO: Complete device type list.
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            NM_OBJ_STR_WIRELESS => Ok(NMDeviceType::Wifi),
            NM_OBJ_STR_WIFIP2P => Ok(NMDeviceType::WifiP2P),
            _ => Err(()),
        }
    }
}

#[derive(Clone, Debug)]
pub struct NMDevice<'a> {
    pub(crate) proxy: DeviceProxy<'a>,
    pub(crate) kind: NMDeviceType,
    pub(crate) path: OwnedObjectPath, //path property hides path method usually available on
                                      //proxies.
}

impl<'a> NMDevice<'a> {
    pub(crate) async fn from_owned_object_path(
        conn: &Connection,
        path: OwnedObjectPath,
    ) -> Result<NMDevice<'a>> {
        let proxy = DeviceProxy::builder(conn)
            .destination(crate::DESTINATION)?
            .path(path.clone())?
            .build()
            .await?;

        let mut iface_type = None;

        let intro_proxy = IntrospectableProxy::builder(conn)
            .destination(crate::DESTINATION)?
            .path(path.clone())?
            .build()
            .await?;
        let xmlinfo = intro_proxy.introspect().await?;
        let node = Node::from_reader(xmlinfo.as_bytes())?;

        for interface in node.interfaces() {
            // Introspection allows us to determine the available interface on an object
            if let Ok(dt) = NMDeviceType::from_str(interface.name()) {
                iface_type = Some(dt);
            }
        }

        let kind = iface_type.ok_or(zbus::Error::Unsupported)?;

        Ok(NMDevice { proxy, kind, path })
    }

    pub async fn into_device<T: FromDevice>(&self, nm: &NMClient<'_>) -> Result<T> {
        T::from_device(nm, self).await
    }

    pub async fn get_iface(&self) -> Result<String> {
        self.proxy.interface().await
    }
}
