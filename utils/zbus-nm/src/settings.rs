// SPDX-FileCopyrightText: 2022 Christopher A. Grant <grantchristophera@gmail.com>
//
// SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

use std::collections::HashMap;

use zbus::{Connection, Result};
use zvariant::{OwnedObjectPath, OwnedValue, Value};

use crate::proxies::NetworkManager::{
    ConnectionActive::ActiveProxy, Settings::SettingsProxy, SettingsConnection::ConnectionProxy,
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
    //PERF: Iterator is still foreign to me and Vec is easy. Lazy evaluation is probably sensible here.
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
        Ok(NMConnection {
            proxy: ConnectionProxy::builder(conn)
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

#[derive(Debug, Eq, PartialEq)]
pub enum NMActiveConnectionState {
    Unknown,
    Activating,
    Activated,
    Deactivating,
    Deactivated,
}

impl<'a> NMActiveConnection<'a> {
    pub async fn new(conn: &Connection, path: OwnedObjectPath) -> Result<NMActiveConnection<'_>> {
        Ok(NMActiveConnection {
            proxy: ActiveProxy::builder(conn)
                .destination(crate::DESTINATION)?
                .path(path)?
                .build()
                .await?,
        })
    }

    pub async fn state(&self) -> Result<NMActiveConnectionState> {
        match self.proxy.state().await {
            Ok(0) => Ok(NMActiveConnectionState::Unknown),
            Ok(1) => Ok(NMActiveConnectionState::Activating),
            Ok(2) => Ok(NMActiveConnectionState::Activated),
            Ok(3) => Ok(NMActiveConnectionState::Deactivating),
            Ok(4) => Ok(NMActiveConnectionState::Deactivated),
            Ok(_) => Ok(NMActiveConnectionState::Unknown),
            Err(e) => Err(e),
        }
    }
}
