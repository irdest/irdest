# Changelog

All notable changes to Irdest are documented in this file. The sections should
follow the order `Releases`, `Added`, `Changed`, `Fixed`, `Removed`, and
`Security`.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).


## 0.5.0

The probably biggest Irdest release so far.  It brings many minor and some major
changes to the codebase and usability of the project.  Importantly, Irdest is
once again grant funded (via NLnet ♥).

The changelog has also been changed in structure, since this monorepo consists
of lots of components, each with their own release versions.  We'll experiment
with different changelog formats for future releases to make it more clear what
changes were made to which components.

Hope you enjoy!

### Releases

- Ratman daemon (ratmand) `0.5.0`
- ratcat `0.5.0`
- ratctl `0.5.0`
- libratman `0.5.0`

### Added

- LoRa connection support via `netmod-lora`.  See the user manual on how to set
  up a LoRa modem.
- Managed WiFi connection support via `netmod-datalink` (supports Linux/
  NetworkManager)
- `ratmand generate` command to pre-generate a ratmand configuration.  Overrides
  can be applied with the `--patch` (for simple key=value configuration
  settings) and `--add-peer` (for peers) arguments.
- Locally registered addresses and routing tables are now persisted on disk
  between restarts.
- Frames that fail to be delivered are now cached in an on-disk journal instead
  of being dropped
- Blocks are cached on disk while a large message stream is being encoded.  For
  large message streams this means that the router will no longer run out of
  memory and crash.  Currently we hold back the whole message stream until all
  blocks are encoded to allow the manifest to be sent first.
- Clients can now subscribe to an address (if they posses the auth token) or
  namespace to be notified of future messages.  Missed messages are replayed
  when restoring a subscription from a restarting client.
- Domain name resolution in `netmod-inet`: you can now list your peers as any
  DNS accessible domain entry and ratmand will resolve the IP address itself.
  This may fail if no resolver is available on your system.

### Changed

- Router dashboard now updates automatically and shows connection statistics for
  peers
- User and developer manuals were completely overhauled
- The Ratman configuration now uses the KDL language, which supports comments
  and nested blocks, without getting unwielding to edit.
- The `ratcat` CLI has changed significantly
  - Managament functions (handle addresses, subscriptions, and in future
  contacts, peers, links, routes, ...)  has been moved to `ratctl`
  - Stream handling (sending and receiving data) remains in `ratcat`, with
    easier commandline interfaces.
- On-disk state has been changed from a random assortment of json files to the
  [fjall](https://github.com/fjall-rs/fjall) embedded database, enabling better
  persistence and upgradability.
- Entries in the routing table support more than a single link, and will update
  based on the most recently received announcement.  This behaviour is temporary
  and will be replaced with real route scoring in the next release.
- Many internal and external structural changes that aren't directly reflected
  in the user experience, but will make development and maintenance easier and
  faster in the future
  - The developer facing crates `ratman-client`, `ratman-netmod`,
    `ratman-types`, etc have been consolidated into `libratman` -- `ratman` on
    crates.io, which was previously the daemon.
  - The daemon crate is now called `ratmand` and no longer available on
    crates.io.

### Fixed

- Announcement forwarding would sometimes be interrupted
- Messages are once again encrypted by default
- Ratman SDK would sometimes send invalid API messages

### Security

- Replaced libsodium signatures and encryption with native Rust alternatives
  - ed25519-dalek is used for signatures, key management and secret negotiation
  - chacha20 is used for block encryption as per the [ERIS] specification

### Removed

- `armv7l` CI pipelines were soft-removed since the new database backend doesn't
  currently support 32-bit systems.  This will change with the next release of
  fjall, but for the time being we're disabling those targets.
- The web dashboard is currently forcably disabled.  Some diagnostics can be
  retrieved via `ratctl peers` or `ratctl status`.

### Known bugs

- Due to [kdl issue #65](https://github.com/kdl-org/kdl-rs/issues/65) insertions
  made to the `peers` block of the ratmand configuration produce wrong
  formatting for the first entry.
- `ratcat` stream chunks are experimental and will start encoding invalid data
  after 150-200MB of input data.


## 0.4.0 (2022-04-16)

### Added

- `ratmand` supports `--daemonize` and `--pidfile` CLI arguments
- Support socket activating `ratmand` on the API socket
- `ratmand` now stores configuration values in
  `$XDG_CONFIG_HOME/ratmand/config.json` and can be launched without providing
  any command line arguments
- `ratmand` now serves a simple dashboard on `localhost:8090` to list known
  network addresses


### Changed

- Network flood messages are now namespaced via an address, instead of going to
  every participant
- `netmod-inet` has been simplified


### Fixed

- Peering discovery via `netmod-lan` no longer results in broadcast loops
- Peering via `netmod-inet` now works more consistently
