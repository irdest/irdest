# About this manual

This manual is split into four sections:

- **Explanation** (mainly [Concepts & Ideas](./concepts.md))
- **Tutorials**, step-by-step instructions for beginners
- **"How to"**, outline on how to do specific in-depth tasks for
  people with some existing Irdest experience
- **Reference**, lookup for different configuration values or
  behaviours


## Building this manual

This manual is part of the main source repository and you can build it
from the same [nix](./nix.md) environment as the rest of the project.


```console
$ cd docs/users
$ mdbook build
```


If you want to enable live-reloading to work on this manual, you can
also use the dev server.

```console
$ mdbook serve
```
