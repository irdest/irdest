# irdest documentation

This directory tree contains the irdest documentation, and website
source.  You can find a hosted version at https://irde.st and
https://irde.st/learn.  If you find any mistakes in any of these
documents, please feel free to get in touch with us, or open a merge
request to fix things.


## Manuals

* `developer`: a manual for potential irdest developers and hackers
* `user`: a manual forend-users of irdest applications.  Contains
  setup guides and available configuration options


## How to build

The manuals are built with `mdbook`.  You can use the corresponding
`build.sh` scripts to build the books.  You can also use the
[`nix`](../nix) build files to build the `irdest-manual-developer` and
`irdest-manual-user` targets.
