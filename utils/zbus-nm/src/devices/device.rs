use std::str::FromStr;
use zbus::fdo::IntrospectableProxy;
use zbus::xml::Node;
use zbus::{export::async_trait::async_trait, Result};
use zvariant::OwnedObjectPath;

use crate::{proxies::NetworkManager::Device::DeviceProxy, NMClient};

const NM_OBJ_STR_ACCESSPOINT: &str = "org.freedesktop.NetworkManager.Device.AccessPoint";
const NM_OBJ_STR_DEVICE: &str = "org.freedesktop.NetworkManager.Device";
const NM_OBJ_STR_WIRELESS: &str = "org.freedesktop.NetworkManager.Device.Wireless";
const NM_OBJ_STR_WIFIP2P: &str = "org.freedesktop.NetworkManager.Device.WifiP2P";

#[async_trait]
pub trait FromDevice {
    async fn from_device(nm: NMClient<'_>, device: NMDevice) -> Result<Self>
    where
        Self: Sized;
}

/// TODO: Complete device type enum.
#[derive(Clone, Copy, Debug)]
pub(crate) enum NMDeviceType {
    AccessPoint,
    Wifi,
    WifiP2P,
}

impl NMDeviceType {
    pub(crate) fn connection_str(&self) -> &str {
        match self {
            NMDeviceType::AccessPoint => "802-11-wireless",
            NMDeviceType::Wifi => "802-11-wireless",
            NMDeviceType::WifiP2P => "wifi-p2p",
        }
    }
}

impl FromStr for NMDeviceType {
    type Err = ();

    /// TODO: Complete device type list.
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            NM_OBJ_STR_ACCESSPOINT => Ok(NMDeviceType::AccessPoint),
            NM_OBJ_STR_WIRELESS => Ok(NMDeviceType::Wifi),
            NM_OBJ_STR_WIFIP2P => Ok(NMDeviceType::WifiP2P),
            _ => Err(()),
        }
    }
}

#[derive(Clone, Debug)]
pub struct NMDevice {
    pub(crate) name: String,
    pub(crate) kind: NMDeviceType,
    pub(crate) path: OwnedObjectPath,
}

impl NMDevice {
    pub(crate) async fn from_owned_object_path(
        nm: &NMClient<'_>,
        path: OwnedObjectPath,
    ) -> Result<Self> {
        let name = DeviceProxy::builder(&nm.proxy.connection())
            .destination(crate::DESTINATION)?
            .path(&path)?
            .build()
            .await?
            .interface()
            .await?;

        let mut iface_type = None;

        let intro_proxy = IntrospectableProxy::builder(&nm.proxy.connection())
            .destination(crate::DESTINATION)?
            .path(&path)?
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

        let kind = iface_type.expect("");

        Ok(NMDevice { name, kind, path })
    }

    pub async fn into<T: FromDevice>(&self) -> Result<T> {
        todo!();
        Err(zbus::Error::Unsupported)
    }
}
