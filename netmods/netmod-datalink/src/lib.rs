// SPDX-FileCopyrightText: 2019-2021 Katharina Fey <kookie@spacekookie.de>
// SPDX-FileCopyrightText: 2019-2021 Yureka Lilian <yuka@yuka.dev>
// SPDX-FileCopyrightText: 2022 Christopher A. Grant <grantchristophera@gmail.com>
//
// SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

#[macro_use]
extern crate tracing;

mod socket;
use std::{collections::HashMap, convert::TryInto, time::Duration};

use libratman::NetmodError;
pub(crate) use socket::Socket;

use useful_netmod_bits::addrs::AddrTable;
use useful_netmod_bits::framing::Envelope;

use async_std::{future, sync::Arc, task};
use async_trait::async_trait;
use libratman::netmod::{Endpoint as EndpointExt, InMemoryEnvelope, Target};
use libratman::types::{RatmanError, Result};
use pnet::util::MacAddr;
use pnet_datalink::interfaces;

use zbus::{
    export::futures_util::{pin_mut, stream, StreamExt},
    zvariant::{OwnedObjectPath, Value},
    Connection,
};
use zbus_nm::{
    devices::device::FromDevice,
    devices::device::NMDevice,
    devices::wifi::NMDeviceWifi,
    settings::{NMActiveConnectionState, PartialConnection},
    NMClient, Options,
};

#[derive(Clone)]
pub struct Endpoint {
    socket: Arc<Socket>,
    addrs: Arc<AddrTable<MacAddr>>,
    //Our network connection will terminate when this closes.
    #[allow(dead_code)]
    nmconn: Arc<Connection>,
}

#[allow(dead_code)]
#[cfg(target_os = "linux")]
fn configure_vif() {
    todo!();
}

async fn scan_wireless_for_ssid<'a>(
    nm: &'a NMClient<'a>,
    iface: Option<&str>,
    ssid: &str,
) -> Option<(NMDevice<'a>, OwnedObjectPath)> {
    let devices = nm.get_all_devices().await.unwrap();

    for device in devices {
        let dev_iface = device.get_iface().await.unwrap();

        if iface.is_some() && iface.unwrap() != dev_iface {
            continue;
        }

        if let Ok(wireless) = NMDeviceWifi::from_device(&nm, &device).await {
            let timestamp = nix::time::clock_gettime(nix::time::ClockId::CLOCK_BOOTTIME).unwrap();
            let time = timestamp.tv_sec() * 1000 + timestamp.tv_nsec() / 1000000;
            let scan_options = HashMap::from([("ssids", Value::from(vec![ssid.as_bytes()]))]);

            //Look for an ssid across all wireless devices
            wireless.request_scan(scan_options).await.unwrap();

            info!("Scanning for SSID on {}...", dev_iface);

            task::block_on(async {
                future::timeout(Duration::from_secs(30), async {
                    while time > wireless.last_scan().await.unwrap() {
                        task::sleep(Duration::from_secs(1)).await;
                    }
                })
                .await
                .unwrap_or_else(|_| warn!("Scan timed out on {}.", dev_iface));
            });

            let aps = wireless.get_all_access_points().await.unwrap();

            for ap in aps {
                if let Ok(ap_ssid) = ap.get_ssid().await {
                    if ssid == String::from_utf8_lossy(&ap_ssid) {
                        //HACK: This function should be returning AP and there should be a trait
                        //to convert an NMAccessPoint object into OwnedObjectPath. However, I am
                        //not sufficiently skilled with Rust at the moment to understand how to
                        //re-write the initialization such that the compiler does not have to worry
                        //about the connection lifetime. For today, I am going to break the
                        //abstraction.
                        return Some((device, ap.get_path()));
                    }
                }
            }
        }
    }
    None
}

async fn create_new_network<'a, 'b>(
    nm: &'a NMClient<'a>,
    iface: Option<&str>,
    ssid: &'b str,
) -> (NMDevice<'a>, PartialConnection<'b>) {
    let config = PartialConnection::from([
        (
            "connection",
            HashMap::from([
                ("type", Value::from("802-11-wireless")),
                ("id", Value::from("Irdest automagic IBSS")),
            ]),
        ),
        //NOTE: Setting channel and band might be important in the future.
        (
            "802-11-wireless",
            HashMap::from([
                ("ssid", Value::from(ssid.as_bytes())),
                ("mode", Value::from("ap")),
                ("band", Value::from("bg")),
                ("channel", Value::from(1u32)),
            ]),
        ),
        //HACK: NetworkManager wants an IP address, so just put some garbage in so it can be
        //satisfied.
        //NOTE: The docs are wrong again. address-data does not work with manual.
        (
            "ipv4",
            HashMap::from([("method", Value::from("link-local"))]),
        ),
        (
            "ipv6",
            HashMap::from([("method", Value::from("link-local"))]),
        ),
    ]);

    let device = match iface {
        Some(i) => {
            let dev = nm.get_device_by_iface(i).await.unwrap();
            //HACK: This is an incredibly stupid hack for type checking :x
            NMDeviceWifi::from_device(nm, &dev).await.unwrap();
            dev
        }
        None => {
            let devices =
                stream::iter(nm.get_all_devices().await.unwrap()).filter_map(|item| async move {
                    //HACK: Same type checking hack again.
                    NMDeviceWifi::from_device(nm, &item)
                        .await
                        .ok()
                        .map(|_| item)
                });
            pin_mut!(devices);
            devices
                .next()
                .await
                .expect("Cannot find a wireless device!")
        }
    };
    (device, config)
}

///This function is currently only designed to handle wifi networks!
///NOTE: This should probably separate from the endpoint someday.
///It's easy to hook it in here and does not tie up the daemon with OS-specific code.
///
///Not specifying an interface and having the endpoint choose one is a bad idea, but neat for a
///proof-of-concept. Prompting the user to make a choice is almost certainly a better idea.
async fn configure_network_manager<'a>(
    conn: &Connection,
    iface: Option<&str>,
    ssid: Option<&str>,
) -> std::result::Result<String, String> {
    info!("Configuring NetworkManager");

    let nm = NMClient::new(conn).await.unwrap();

    //This looks absolutely atrocious, but there are effectively 3 possibilities:
    //Case 1: The router provides an SSID, so the endpoint should attempt to set up the wireless
    //network with the given SSID, with or without a default interface.
    //
    //Case 2: Just write to whatever interface was given. No setup.
    //
    //Case 3: The router gave no information. Do not continue at the
    //moment. This can be resolved in a few ways, including generating an SSID.
    let res = match (iface, ssid) {
        (_, Some(s)) => {
            let (conn, device, obj) = match scan_wireless_for_ssid(&nm, iface, s).await {
                Some((device, ap)) => (
                    HashMap::from([
                        (
                            "ipv4",
                            HashMap::from([("method", Value::from("link-local"))]),
                        ),
                        (
                            "ipv6",
                            HashMap::from([("method", Value::from("link-local"))]),
                        ),
                    ]),
                    device,
                    ap,
                ),
                None => {
                    let (device, config) = create_new_network(&nm, iface, s).await;
                    (config, device, "/".try_into().unwrap())
                }
            };
            let (_connection, active) = match nm
                .add_and_activate_connection2(
                    conn,
                    &device,
                    obj,
                    Options::from([
                        ("persist", Value::from("volatile")),
                        //NOTE: NM DBus docs are wrong.
                        ("bind-activation", Value::from("dbus-client")),
                    ]),
                )
                .await
            {
                Ok((c, a, _)) => (c, a),
                Err(e) => return Err(format!("Failed to configure NetworkManager: {}", e)),
            };

            info!("Waiting for NetworkManager to configure the device...");

            //Network sometimes takes a bit to get going.
            future::timeout(Duration::from_secs(30), async {
                while active.state().await.unwrap() != NMActiveConnectionState::Activated {
                    task::sleep(Duration::from_secs(1)).await;
                }
            })
            .await
            .unwrap_or_else(|_| warn!("Timed out trying to configure NetworkManager!"));

            device.get_iface().await.unwrap()
        }
        (Some(i), None) => i.to_owned(),
        (None, None) => panic!("Neither the interface nor the ssid were supplied."),
    };
    info!("Successfully configured NetworkManager");
    Ok(res)
}

impl Endpoint {
    /// Create a new endpoint and spawn a dispatch task
    pub fn spawn(iface: Option<&str>, ssid: Option<&str>) -> Arc<Self> {
        task::block_on(async move {
            let nmconn = Arc::new(Connection::system().await.unwrap());
            let iface_str = configure_network_manager(&nmconn, iface, ssid)
                .await
                .unwrap();

            let niface = interfaces()
                .into_iter()
                .rfind(|i| i.name == iface_str)
                .expect(&format!("Interface name {} not found.", iface_str));

            let addrs = Arc::new(AddrTable::new());
            Arc::new(Self {
                socket: Socket::new(niface, Arc::clone(&addrs)).await.unwrap(),
                addrs,
                nmconn,
            })
        })
    }

    #[cfg(test)]
    pub async fn peers(&self) -> usize {
        self.addrs.all().await.len()
    }
}

#[async_trait]
impl EndpointExt for Endpoint {
    fn size_hint(&self) -> usize {
        1500
    }

    async fn send(
        &self,
        InMemoryEnvelope { buffer, .. }: InMemoryEnvelope,
        target: Target,
        exclude: Option<u16>,
    ) -> Result<()> {
        if buffer.len() > 1500 {
            return Err(RatmanError::Netmod(NetmodError::FrameTooLarge));
        }

        let env = Envelope::Data(buffer);

        match target {
            Target::Single(ref id) => {
                self.socket
                    .send(&env, self.addrs.addr(*id).await.unwrap())
                    .await;
            }
            Target::Flood => match exclude {
                Some(u) => {
                    let exc = self
                        .addrs
                        .addr(u)
                        .await
                        .expect("Router sent invalid exclude id.");
                    let peers = self
                        .addrs
                        .all()
                        .await
                        .into_iter()
                        .filter(|&addr| addr != exc)
                        .collect();

                    self.socket.send_multiple(&env, &peers).await;
                }
                None => self.socket.multicast(&env).await,
            },
        }

        Ok(())
    }

    async fn next(&self) -> Result<(InMemoryEnvelope, Target)> {
        let fe = self.socket.next().await;
        Ok((fe.0, fe.1))
    }
}
