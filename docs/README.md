# irdest documentation

This folder contains the irdest documentation. You can find them online
under [https://docs.irde.st](https://docs.irde.st)

You're welcome to contribute to them!


## Guides

* `developer`: a manual aimed at potential irdest developers, and
  hackers who want to understand the internals of various components.
* `user`: a manual aimed at end-users of irdest applications.  Contains
  setup guides and available configuration options


## How to build

The manuals are built with `mdbook`.  You can use the corresponding
`build.sh` scripts to build the books.  You can also use the
[`nix`](../nix) build files to build the `irdest-manual-developer` and
`irdest-manual-user` targets.
