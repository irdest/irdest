##IrdestVpn

### Build and run a debug version
1. Clone Irdest repo 
```shell
git clone https://git.irde.st/we/irdest.git
cd clients/android-vpn
```
2. Build the app
```shell
./gradlew assembleDebug
```

3. Install app on your phone with ADB
```shell
adb -d install -t app/build/outputs/apk/debug/app-debug.apk
```