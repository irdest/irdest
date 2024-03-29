// SPDX-FileCopyrightText: 2023 Katharina Fey <kookie@spacekookie.de>
//
// SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

//! Utilities for starting up the Ratman router daemon
//!
//!

use crate::{config::ConfigTree, util::DriverMap};
use libratman::RatmanError;
use netmod_datalink::Endpoint as DatalinkEndpoint;
use netmod_inet::InetEndpoint;
use netmod_lan::Endpoint as LanEndpoint;
use netmod_lora::LoraEndpoint;

pub(crate) async fn initialise_netmods(cfg: &ConfigTree) -> Result<DriverMap, RatmanError> {
    let mut map = DriverMap::new();

    //// If the 'inet' config tree exists...
    if let Some(tree) = cfg.get_subtree("inet") {
        let enable = tree.get_bool_value("enable");
        let bind = tree.get_string_value("bind");

        // Print a helpful warning about a missing feature
        if let Some(true) = tree.get_bool_value("use_upnp") {
            warn!("UPNP setup is currently broken; the configuration option 'use_upnp' will be ignored");
        }

        match (enable, bind) {
            // If enable is true and a bind address was provided
            (Some(true), Some(bind)) => match InetEndpoint::start(bind.as_str()).await {
                Ok(inet) => {
                    map.insert("inet".into(), inet);
                }
                Err(e) => {
                    error!("Netmod 'inet' failed to initialise: {}. skipping...", e);
                }
            },
            // If enable is true, but no (valid utf-8) bind address was provided
            (Some(true), None) => {
                error!("Netmod 'inet' requires configuration field 'bind' to start!");
                // TODO: should initialisation just fail here then ??
            }
            // If enable is false, we do nothing
            _ => {}
        }
    }

    //// If the 'lan' subtree exists...
    if let Some(tree) = cfg.get_subtree("lan") {
        let iface = tree.get_string_value("interface");
        let enable = tree.get_bool_value("enable");
        let port = tree.get_number_value("port");

        match (enable, port) {
            // If enable is true and a port was provided (in principle)
            (Some(true), Some(port)) => match LanEndpoint::spawn(iface, port as u16) {
                Ok(lan) => {
                    map.insert("lan".into(), lan);
                }
                Err(e) => {
                    error!("Netmod 'lan' failed to initialise: {}. skipping...", e);
                }
            },
            // If enable is true, but no port was provided
            (Some(true), None) => {
                warn!("Netmod 'lan' requires configuration field 'port' to start!");
            }
            // If enable is false we do nothing
            _ => {}
        }
    }

    //// If the 'lora' subtree exists
    if let Some(tree) = cfg.get_subtree("lora") {
        let enable = tree.get_bool_value("enable");
        let port = tree.get_string_value("port");
        let baud = tree.get_number_value("baud");

        match (enable, port, baud) {
            (Some(true), Some(port), Some(baud)) => {
                let lora = LoraEndpoint::spawn(&port, baud as u32);
                map.insert("lora".into(), lora);
            }
            (Some(true), None, _) => {
                warn!("Netmod 'lora' requires configuration field 'port' to start!");
            }
            (Some(true), _, None) => {
                warn!("Netmod 'lora' requires configuration field 'baud' to start!");
            }
            _ => {}
        }
    }

    if let Some(tree) = cfg.get_subtree("datalink") {
        let enable = tree.get_bool_value("enable");
        let interface = tree.get_string_value("interface");
        let ssid = tree.get_string_value("ssid");

        match (enable, interface, ssid) {
            // If enable is true, we don't care about whether interface or ssid are missing
            (Some(true), iface, ssid) => {
                map.insert(
                    "datalink".into(),
                    DatalinkEndpoint::spawn(iface.as_deref(), ssid.as_deref()),
                );
            }
            // If enable is false, we do nothing
            _ => {}
        }
    }

    Ok(map)
}
