# Hacking on Irdest

Hey, it's cool that you want to hack on Irdest :) We recommend you
install [nix](https://nixos.org) to handle dependencies.  Depending on
the directory you are in you can fetch development dependencies:

```console
$ cd irdest/
$ nix-shell # install the base dependencies
...
$ cd docs/
$ nix-shell # install documentation dependencies
```

With [lorri] and [direnv] installed transitioning from one directory
to another will automatically load additional dependencies!

[lorri]: https://github.com/target/lorri
[direnv]: https://direnv.net/
