# Changelog

All notable changes to Irdest are documented in this file. The
sections should follow the order `Releases`, `Added`, `Changed`,
`Fixed`, `Removed`, and `Security`.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).


## Unreleased

### Releases

- Ratman `0.5.0`
- Irdest mblog `0.1.0`

### Added

- LoRa connection support via `netmod-lora` (not compiled by default,
  use `lora` feature)
- Managed WiFi connection support via `netmod-raw` (supports Linux/
  NetworkManager)

### Changed

- Router dashboard now updates automatically and shows connection
  statistics for peers
- Messages are segmented into encrypted blocks before sending.  This
  will allow store & forwarding in the future
- User and developer manuals were completely overhauled

### Fixed

- Announcement forwarding would sometimes be interrupted
- Messages are once again encrypted by default
- Ratman SDK would sometimes send invalid API messages

### Security

- Cryptography was changed from libsodium to native Rust
  (dalek-cryptography)


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
