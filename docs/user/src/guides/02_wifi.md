# Wi-Fi setup

[If you have not already done so, please read the Basic setup section of the manual. It will be referenced frequently.](./01_basic.md)

**Please note that the interface described in this file will be subject to 
change as the project matures. It is not recommended to use this module without
a dedicated wireless interface yet as it will disrupt other network stacks. 
Additionally, only Linux is currently supported.**

If you wish to use the network autoconfiguration features, NetworkManager must
be installed and running on your machine. The interface requires privileges as
it accesses Linux's ethernet network stack directly.

At the moment, there are 3 ways of using the raw networking module that powers 
the wifi connectivity depending on how you supply the configuration to the 
module. The configuration fields of interest are called `ssid` and 
`datalink-iface`. The first expects the **utf-8 ONLY** ssid name and the 
second expects the OS's interface name, such as wlan0, wlp1s0, wlo0, etc. They 
can be configured as follows:

1. Remove the SSID field from config.json and only supply the interface field. 
The endpoint will attempt to connect directly to the given device. This works 
well for accessing local peers over wifi or ethernet and in cases where manual
configuration of the wireless device is desired.

2. Provide both the SSID and interface fields. The endpoint will then attempt 
to use NetworkManager to scan for the given SSID and connect to it. If it is 
not found, a new ad-hoc network will be created that can then be joined from
other device.

3. Provide only the SSID and remove interface field. The endpoint will attempt
to scan over all available wireless interfaces to find the given SSID. If it is
not found, a device will be picked and a new ad-hoc network will be created
that can then be joined from other devices.

## Advanced configuration (manual)

It would be nice to support Wi-Fi Direct and VIF auto-configuration on Linux.
These can currently be manually configured with your tool of choice (iw, 
wpa_supplicant, etc.) and used in ratman via the first configuration method.

## Troubleshooting

### Ratman panics on startup

Ensure that you have the correct permissions. This means you can either run the 
process as root (not recommended) or with CAP_NET_RAW and CAP_NET_ADMIN. 

See `man capabilities(7)` for more details.

### Ratman does not automatically connect to the network

It is currently unclear why this happens. At the moment, the easiest way to 
resolve this problem is to use `nmtui` and connect to the ad-hoc network
manually. It will likely fail to connect on nmtui, however it usually works if 
you edit the connection, manually set an ipv4 address and enable automatic 
connection. Feel free to set the ip address to anything above `0.0.0.0`. 
There are no subnetworks, so the netmask should be set to `/0`.

### Why can't I connect to the internet?

This endpoint captures all packets sent to the given device. If you had any 
higher level network protocols operating on it, you're out of luck.
