# Irdest clients

This directory contains different irdest user applications.  For a more
detailed write-up on supported platforms, install instructions, and
client capabilities, check out the [user
manual](https://docs.irde.st/user/)!

- [irdest-hubd](./hubd) -- a daemon that accepts connections via the
  QRPC interface.  Comes with TCP, UDP, and UPnP support.
- [irdest-gtk](./irdest-gtk) -- a native Gtk+ irdest client, supporting
  text messaging and file sharing (prototype state)
- [irpc-client](./irpc-client) -- a simple RPC client that takes json
  inputs and connects to a qaul backend (such as `qaul-hubd`) to
  execute user-commands.
- [irdest-droid](./android) -- an integrated Android app with wireless
  capabilities, and a native user interface (prototype state)
