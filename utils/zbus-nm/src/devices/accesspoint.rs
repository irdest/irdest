use zbus::{Connection, Result};
use zvariant::OwnedObjectPath;

use crate::proxies::NetworkManager::AccessPoint::AccessPointProxy;

pub enum NM80211Mode {
    Unknown,
    Adhoc,
    Infra,
    AP,
    Mesh,
}

impl From<u32> for NM80211Mode {
    fn from(u: u32) -> Self {
        match u {
            0 => Self::Unknown,
            1 => Self::Adhoc,
            2 => Self::Infra,
            3 => Self::AP,
            4 => Self::Mesh,
            _ => Self::Unknown,
        }
    }
}

#[derive(Clone)]
pub struct NMAccessPoint<'a> {
    proxy: AccessPointProxy<'a>,
}

impl<'a> NMAccessPoint<'a> {
    pub(crate) async fn new(conn: &Connection, path: OwnedObjectPath) -> Result<NMAccessPoint<'a>> {
        Ok(NMAccessPoint {
            proxy: AccessPointProxy::builder(conn)
                .destination(crate::DESTINATION)?
                .path(path)?
                .build()
                .await?,
        })
    }

    pub async fn get_ssid(&self) -> Result<Vec<u8>> {
        self.proxy.ssid().await
    }

    pub async fn get_mode(&self) -> Result<NM80211Mode> {
        Ok(self.proxy.mode().await?.into())
    }

    ///Deadline is approaching so I need to break the abstraction :(
    pub fn get_path(&self) -> OwnedObjectPath {
       self.proxy.path().clone().into()
    }
}


