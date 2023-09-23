# Basic setup

We trust that you have successfully installed Ratman (`ratmand`,
`ratcat`, `ratctl`) on your system.  Please refer to the
[Installation](../install/index.md) section for details.

The Ratman daemon is primarily set-up from a configuration file.  You
can find it at the following path, depending on your operating system:

- XDG system: `$XDG_CONFIG_HOME/ratmand/ratmand.kdl`.
- macOS: `Users/[USER_NAME]/Library/Application Support/org.irdest.ratmand`

Ratman also stores some runtime state.  This includes registered local
addresses, known network peer addresses, connection statistics, and
in-transit messages.  Delete these directories to wipe all data and
re-start Ratman from a blank slate.

- XDG system: `$XDG_DATA_HOME/share/ratmand`.
- macOS:  `/Users/[USER_NAME]/Library/Application Support/org.irdest.ratmand`.

**Since Ratman is still in alpha it may be neccessary to wipe
application state in-between updates!  Please do not rely on this
software yet!**

## Public internet test network - coming soon

**This is not available yet. We are working hard to bring this.**

As part of the Irdest project we put up a small test network between
our servers.  You can join it via the internet!  This network is meant
for testing the performance and stability of the Irdest tools.  If you
are a developer you are welcome to interact with this network with
your own applications.

First, make sure that Ratman is configured to support `inet` peering!

```json
{
  ...

  "inet_bind": "[::]:9000",
  "use_inet": true,
  "peer_file": "public-test-network.pm",

  ...
}
```

Once available, you will be able to download `public-test-network.pm`
and place it next to the Ratman configuration.

```console
systemctl --user restart ratmand
```

Next up you can check the router dashboard for incoming address
announcements by navigating to
[localhost:8080](http://localhost:8080) in your browser!

### Public services

If you are running a public test service, feel free to submit it for
inclusion in this manual.  The service must be open source!

| Service name | Description                                        | Address(es) |
| ------------ | -------------------------------------------------- | ----------- |
| irdest-echo  | Accepts messages and echos them back to the sender |             |

