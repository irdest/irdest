# Changelog

All notable changes to Irdest are documented in this file. The
sections should follow the order `Packaging`, `Added`, `Changed`,
`Fixed`, `Removed`, and `Security`.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).


## Unreleased

### Added

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


## Fixed

- Peering discovery via `netmod-lan` no longer results in broadcast
  loops
- Peering via `netmod-inet` now works more consistently
