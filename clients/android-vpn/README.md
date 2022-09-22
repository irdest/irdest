## Irdest android vpn
This android app let you connnect to irdest network from android device. When user click connect button, It will runs `ratmand` and `irdest-proxy` on background thread and starts android vpn service. Android VPN service will intercept the local network packets and tunnel to `Irdest-proxy`. `Irdest proxy` will change local network address(IPv4 or IPv6) to irdest network ID before send packets to other irdest nodes in irdest network.

### Current state
It runs ratmand on background thread. you can test of running ratmand with `ratcat`.
It runs VpnService and intercepts local packets.
It starts `forground service` when service is starting. (user can close the app while app is running on the backgorund and control connect/disconnect via notification manager.)
It shows rust log on logcat(android).

### For contributors
##### Need to do(Maybe I will open issues)
- Run `irdest-proxy` and connect with tun interface.
- Implements encryption for vpn tunnel(If we need).
- User version of UI.
- UI for configuration.
- Overall refactoring.
- Implements `WebView` for `dashboard`.
- Update docker image.

### Rust on android.
`IrdestVPN` uses Mozila's project [rust-android](https://github.com/mozilla/rust-android-gradle) to deploy rust library on android. Here is nice step by step guild about [Building and Deploying a Rust library on Android](https://mozilla.github.io/firefox-browser-architecture/experiments/2017-09-21-rust-on-android.html).

###### Build library for testing
For the testing after you made any changes, maybe manually build `libratamn_android.so` with `cargo` will be faster then with `gradle`.
```.sh
cargo build --target aarch64-linux-android --lib // for arm 64bit system
```

###### Personal opinion🧅
When I start to build IrdestVPN, Irdest already uses mozila's `rust-android-gradle`. It's painless and easy but maybe in the future you can deplying it another way.
[Android Doc](https://source.android.com/docs/setup/build/rust/building-rust-modules/overview)

### Gitlab CI job and Docker image for rust x android.
I wrote a CI job, and dockerfile and both needs update(not urgent)
###### .gitlab-ci.yml
[android-vpn-ci.yml](https://git.irde.st/we/irdest/-/blob/develop/ci/pipeline-scripts/android-vpn-ci.yml)
- This ci job will install `protoc` on running time. (this should be removed after update docker image).
###### Docker
[Dockerfile](https://gitlab.com/oooh_chew/ci-test/-/blob/master/Dockerfile)
- The probloem is that it will download gradle and sdk on job running time. One benefit is that we can test new version of gradle without make changes on docker image [gradle-wrapper.properties](https://git.irde.st/we/irdest/-/blob/develop/clients/android-vpn/gradle/wrapper/gradle-wrapper.properties)🫑.
