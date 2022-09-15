use zbus::{Connection, Result};

mod proxies;

use crate::proxies::NetworkManager::NetworkManagerProxy;

mod devices;

use crate::devices::device::NMDevice;

mod settings;

use settings::NMSettings;

const DESTINATION: &str = "org.freedesktop.NetworkManager";
const SETTINGS_PATH: &str = "/org/freedesktop/NetworkManager/Settings";

pub struct NMClient<'a> {
    pub(crate) proxy: NetworkManagerProxy<'a>,
    pub settings: NMSettings<'a>,
}

impl<'a> NMClient<'a> {
    ///Allow the client lib to create the connection.
    pub async fn new(conn: Option<Connection>) -> Result<NMClient<'a>> {
        let system_connection = match conn {
            Some(val) => val,
            None => Connection::system().await?,
        };

        let nm_proxy = NetworkManagerProxy::new(&system_connection).await?;

        Ok(NMClient {
            proxy: nm_proxy,
            settings: NMSettings {
                proxy: SettingsProxy::builder(&system_connection)
                    .destination(DESTINATION)?
                    .path(SETTINGS_PATH)?
                    .build()
                    .await?,
            },
        })
    }

    pub async fn get_all_devices(&self) -> Result<Vec<NMDevice>> {
        let reply = self.proxy.get_all_devices().await?;

        let mut devices = Vec::new();

        for path in reply {
            if let Ok(device) = NMDevice::from_owned_object_path(self, path).await {
                devices.push(device);
            }
        }

        Ok(devices)
    }
}

//#[async_std::main]
//async fn main() -> Result<(), Box<dyn Error>> {
//
//
//    // The SettingsProxy provides access to NetworkManager's saved connections and provides better
//    // control when adding a new connection than the base object.
//    let settings_proxy = SettingsProxy::builder(&system_connection)
//        .destination(DESTINATION)?
//        .path("/org/freedesktop/NetworkManager/Settings")?
//        .build()
//        .await?;
//
//    let reply = settings_proxy.list_connections().await?;
//
//    // Item should always be a path to a org.freedesktop.NetworkManager.Settings.Connection object
//    for item in reply {
//        let settings_connection_proxy = ConnectionProxy::builder()
//            .destination(DESTINATION)?
//            .path(&item)?
//            .build()
//            .await?;
//
//        let value = settings_connection_proxy.get_settings().await?;
//
//        dbg!(value);
//
//        //TODO: Finish automatically connecting to IBSS or AP
//        //if value.connection.ssid == "ratman" {
//        //    nm_proxy.activate_connection(item, devices.first(), ).await?;
//        //}
//    }
//
//
//
//    //TODO: Wireless
//    let wireless_device = devices
//        .iter()
//        .take_while(|test| test.name == "org.freedesktop.NetworkManager.Device.Wireless")
//        .next()
//        .unwrap();
//
//    let mut test_conn = HashMap::new();
//    //TODO: consider concatenating random base32 string to end of ssid name?
//    //TODO: NetworkManager connection types and device types as enums?
//    test_conn.insert("ssid", "ratman");
//    test_conn.insert("type", "802-11-wireless");
//    test_conn.insert("interface-name", &wireless_device.name);
//    test_conn.insert("", "");
//
//    //let reply = settings_proxy.add_connection_unsaved(test_conn);
//
//    Ok(())
//}
