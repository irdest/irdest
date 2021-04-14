# alexandria 📚

Strongly typed, embedded record-database with built-in encryption at
rest storage.  Supports key-value Diff transactions, as well as
externally loaded binary payloads.  Encrypted metadata without extra
configuration.

Alexandria has the following features:

- Store data on internal db path
- Query the database by path or dynamic search tags
- Subscribe to events based on query
- Iterate over query dynamically
- Store data in session or global namespaces

**Notice:** alexandria should be considered experimental and not used
in production systems where data loss is unacceptable.


## How to use

Alexandria requires `rustc` 1.42 to compile.

```rust
use alexandria::{Library, Builder};
use tempfile::tempdir();

let dir = tempdir().unwrap();
let lib = Builder::new()
              .offset(dir.path())
              .root_sec("car horse battery staple")
              .build()?


```


Alexandria is developed as part of [irdest].  We have a
[Matrix] channel! Please come by and ask us questions!  (the issue
tracker is a bad place to ask questions)

[Matrix]: https://matrix.to/#/#irdest:fairydust.space?via=ontheblueplanet.com&via=matrix.org&via=fairydust.space


## License

Alexandria is free software and part of [irdest]. You are free to use,
modify and redistribute the source code under the terms of the GNU
General Public License 3.0 or (at your choice) any later version. For
a full copy of the license, see `LICENSE` in the source directory
attached.

**Additional Permissions:** For Submission to the Apple App Store:
Provided that you are otherwise in compliance with the GPLv3 for each
covered work you convey (including without limitation making the
Corresponding Source available in compliance with Section 6 of the
GPLv3), the qaul developers also grant you the additional permission
to convey through the Apple App Store non-source executable versions
of the Program as incorporated into each applicable covered work as
Executable Versions only under the Mozilla Public License version 2.0.

A copy of both the GPL-3.0 and MPL-2.0 license texts are included in
this repository.

[irdest]: https://irde.st
