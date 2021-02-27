# qaul documentation

This folder contains the qaul documentation. You can find them online
under [https://docs.qaul.org](https://docs.qaul.org)

You're welcome to contribute to them!


## Guides

* `developer`: a manual aimed at potential qaul developers, and
  hackers who want to understand the internals of various components.
* `user`: a manual aimed at end-users of qaul applications.  Contains
  setup guides and available configuration options


## How to build

The manuals are built with `mdbook`.  You can use the corresponding
`build.sh` scripts to build the books.  You can also use the
[`nix`](../nix) build files to build the `qaul-manual-developer` and
`qaul-manual-user` targets.
