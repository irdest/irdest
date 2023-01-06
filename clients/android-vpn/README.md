# irdestVPN 
## Build

1. Get the repository
```shell
git clone https://git.irde.st/we/irdest.git
```

2. Build the app
with gradlew
```shell
cd irdest/clients/android-vpn 
./gradlew assembleDebug
```
or open and build in AndroidStudio.

3. Install it on your phone:
```shell
adb -d install -t app/build/outputs/apk/debug/app-debug.apk
```
or run in AndroidStudio.

## Run test server 
* Original [ToyVpnServer](https://android.googlesource.com/platform/development/+/master/samples/ToyVpn/server/linux/ToyVpnServer.cpp)
* I copied the server code into this repo because I needed to make small change.
* This server only runs on Linux.
* This is a simple guide I wrote for you.
  You can read the official description in the [file](https://android.googlesource.com/platform/development/+/master/samples/ToyVpn/server/linux/ToyVpnServer.cpp).

1. Compile the server
```shell
cd ToyVpnServer
g++ -o ToyVpnServer ToyVpnServer.cpp
chmod +x ToyVpnServer 
```

2. Create and configure the TUN interface   
with automation script:
```shell
sudo ./open_tun.sh
```
Also you can change the default settings:
```shell
vim open_tun.sh
...
TUN_NAME="tun0"
SRC_ADDR="10.0.0.1/32"
DEST_ADDR="10.0.0.2/32"
...
```
or create a tun interface and set the NAT rules manually.

3. Run server
```shell
// # Create a server on port 8000 with shared secret "test".
./ToyVpnServer tun0 8000 test -m 1400 -a 10.0.0.2 32 -d 8.8.8.8 -r 0.0.0.0 0
```
