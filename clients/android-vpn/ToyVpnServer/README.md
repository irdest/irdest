## <center>[ToyVpnServer](https://android.googlesource.com/platform/development/+/master/samples/ToyVpn/server/linux/ToyVpnServer.cpp)

#### Changes
* line no.118 _function `get_tunnel(port, secret)`_
```c++
// Receive packets till the secret matches.
} while (packet[0] != 0 || strcmp(secret, &packet[1]));
```
change to
```c++
} while (packet[0] != 0 && strcmp(secret, &packet[1]));
```