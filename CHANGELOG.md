# Changelog

All notable changes to Irdest are documented in this file. The
sections should follow the order `Releases`, `Added`, `Changed`,
`Fixed`, `Removed`, and `Security`.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).


## 0.5.0

This release brings many small changes together that have accumulated
over the last year or so.  Importantly, Irdest is once again grant
funded (via NLnet ♥), so development speed should pick up from this
point onwards.

The changelog has also been changed in structure, since this monorepo
consists of lots of components, each with their own release versions.

### Releases

- Ratman daemon (ratmand) `0.5.0`
- libratman `0.5.0`
- Irdest mblog `0.1.0`

### Added

- LoRa connection support via `netmod-lora`.  See the user manual on
  how to set up a LoRa modem.
- Managed WiFi connection support via `netmod-datalink` (supports Linux/
  NetworkManager)
- `ratmand generate` command to pre-generate a ratmand configuration.
  Overrides can be applied with the `--patch` (for simple key=value
  configuration settings) and `--add-peer` (for peers) arguments.

### Changed

- Router dashboard now updates automatically and shows connection
  statistics for peers
- User and developer manuals were completely overhauled
- The Ratman configuration now uses the KDL language, which supports
  comments and nested blocks, without getting unwielding to edit.
- Many internal and external structural changes that aren't directly
  reflected in the user experience, but will make development and
  maintenance easier and faster in the future
  - The developer facing crates `ratman-client`, `ratman-netmod`,
    `ratman-types`, etc have been consolidated into `libratman` --
    `ratman` on crates.io, which was previously the daemon.
  - The daemon crate is now called `ratmand` and no longer available
    on crates.io.

### Fixed

- Announcement forwarding would sometimes be interrupted
- Messages are once again encrypted by default
- Ratman SDK would sometimes send invalid API messages

### Security

- Cryptography was changed from libsodium to native Rust
  (dalek-cryptography)

### Known bugs

- Due to [kdl issue #65](https://github.com/kdl-org/kdl-rs/issues/65)
  insertions made to the `peers` block of the ratmand configuration
  produce wrong formatting for the first entry.

## 0.4.0 (2022-04-16)

### Added

- `ratmand` supports `--daemonize` and `--pidfile` CLI arguments
- Support socket activating `ratmand` on the API socket
- `ratmand` now stores configuration values in
  `$XDG_CONFIG_HOME/ratmand/config.json` and can be launched without
  providing any command line arguments
- `ratmand` now serves a simple dashboard on `localhost:8090` to list
  known network addresses


### Changed

- Network flood messages are now namespaced via an address, instead of
  going to every participant
- `netmod-inet` has been simplified


### Fixed

- Peering discovery via `netmod-lan` no longer results in broadcast
  loops
- Peering via `netmod-inet` now works more consistently
