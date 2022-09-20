use std::collections::HashMap;

use zbus::{Result, Connection};
use zvariant::{OwnedValue, OwnedObjectPath, Value};

use crate::proxies::NetworkManager::{
    Settings::SettingsProxy, SettingsConnection::ConnectionProxy, ConnectionActive::ActiveProxy,
};

pub struct NMSettings<'a> {
    pub(crate) proxy: SettingsProxy<'a>,
}

pub type PartialConnection<'a> = HashMap<&'a str, HashMap<&'a str, Value<'a>>>;

pub struct NMConnection<'a> {
    pub(crate) proxy: ConnectionProxy<'a>,
}

//TODO: Implement
#[allow(dead_code)]
pub struct NMActiveConnection<'a> {
    pub(crate) proxy: ActiveProxy<'a>,
}

impl<'a> NMSettings<'a> {
    pub async fn list_connections(&self) -> Result<Vec<NMConnection>> {
        let paths = self.proxy.list_connections().await?;
        let mut connections = Vec::new();

        for path in paths {
            connections.push(NMConnection::new(&self.proxy.connection(), path).await?);
        }

        Ok(connections)
    }
}

pub type NMSetting = HashMap<String, HashMap<String, OwnedValue>>;

impl<'a> NMConnection<'a> {
    pub async fn new(conn: &Connection, path: OwnedObjectPath) -> Result<NMConnection<'_>> {
        Ok(NMConnection { proxy: ConnectionProxy::builder(conn)
            .destination(crate::DESTINATION)?
                .path(path)?
                .build()
                .await?,
        })
    }

    pub async fn get_settings(&self) -> Result<NMSetting> {
        self.proxy.get_settings().await
    }
}

impl<'a> NMActiveConnection<'a> {
    pub async fn new(conn: &Connection, path: OwnedObjectPath) -> Result<NMActiveConnection<'_>> {
        Ok(NMActiveConnection { proxy: ActiveProxy::builder(conn)
            .destination(crate::DESTINATION)?
                .path(path)?
                .build()
                .await?, })
    }
}
