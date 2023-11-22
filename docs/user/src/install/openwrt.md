# [OpenWrt](https://openwrt.org/) installation

The OpenWrt Project is a Linux operating system targeting embedded devices.
Irdest Ratman is available as a package for OpenWrt via our CI for some architectures.

## Prerequisites

This is only tested for

- `armv7l` 32-bit architecture. e.g. this is tested
  with `Linux OpenWrt 5.10.146 #0 SMP Fri Oct 14 22:44:41 2022 armv7l GNU/Linu`.
- `musl libc (armhf)` stdlib.

## Download

You can do these steps on the router itself or on your machine. If you did the
above on your machine, then you need to copy the `.ipk` file to your router.
The complete set of instructions may look like this.

1. Download the package file using `wget` or `curl` etc.
    ```shell
    wget -O artifacts.zip "https://git.irde.st/we/irdest/-/jobs/53163/artifacts/download?file_type=archive"
    ```
2. You will have artifacts.zip here; unzip it
      ```shell
      unzip artifacts.zip
      ```
3. You will have .ipk file here like "ratman-openwrt-0.5.0-1_arm_cortex-a9_vfpv3.ipk". Copy that to the router running
   OpenWrt.
      ```shell
      scp ratman-openwrt-0.5.0-1_arm_cortex-a9_vfpv3.ipk remote-server:/root/
      ```

## Installation and check

1. Run the package installer
    ```shell
    root@OpenWrt:~# opkg install ratman-openwrt-0.5.0-1_arm_cortex-a9_vfpv3.ipk
    ```
   Possible output 👍
    ```shell
    Installing ratmand (0.5.0-1) to root...
    Configuring ratmand.
    ```
2. Check
    ```shell
    which ratmand
    ```
   Possible output 👍
    ```shell
    /usr/bin/ratmand
    ```
3. Run
    ```shell
    root@OpenWrt:~# ratmand
    ```
   Possible output 👍
    ```shell
    Loading config from "/root/.config/ratmand/ratmand.kdl"
    Nov 22 17:58:24.571  INFO ratmand::util: Initialised logger: welcome to ratmand
    ...
    ```

## Future plans

- Create a package for OpenWrt and have it available via their feeds. Some links to help you get started. You can ask
  Aman (aj@amanjeev.com) about where he is right now in this process.
    - [Get started with their packages and feeds](https://openwrt.org/packages/start).
    - [Packages repository on GitHub](https://github.com/openwrt/packages).
    - [Routing packages repository on GitHub](https://github.com/openwrt/routing) which is where Ratman is likely to be.
- Test this and create this package for other more common architectures for routers.