# qaul documentation

This section outlines various pieces of the qaul project that
aren't directly code related.


## Manuals

Because this manual is part of the main source repository, you can
build it from the same environment as the main code.  If you use [nix]
to scope dependencies, you can simply run `mdbook serve` to build and
serve the built html files.

```console
$ cd docs/contributors
$ mdbook serve
```

## Websites

qaul runs many different web services:

* [qaul.org](https://qaul.org) the qaul web site
  * [contributor manual](https://docs.qaul.org/contributors) (this document)
  * [user manual](https://docs.qaul.org/users)
  * [http-api](https://docs.qaul.org/http-api) qaul REST API guide
  * [Rust documentaiion](https://docs.qaul.org/api) the qaul rust software API documentation (automatically created from the qaul code sources)
* [get.qaul.org](https://get.qaul.org) the qaul download directory for the qaul binaries and big content files (e.g. videos, etc.).

This chapter explains how they are hosted, updated, where to look to
edit and change them and who to contact when the service is not
working or you would like to have access to it.


### qaul Web Site

There is an [own chapter] in this guide on the editing of the qaul
web site.  Please have a look there on how to edit and translate the
web site.

* Server: https://qaul.org
* Source repository: https://git.qaul.org/qaul/qaul/blob/develop/docs/website
* Updated: by deploy script
* Admin contact: contact@qaul.org

[own chapter]: /website


### docs.qaul.org Documentation

The software documentation & guides of qaul

* Server: https://docs.qaul.org
* Source repository: https://git.qaul.org/qaul/qaul/tree/develop/docs/
* Updated: by deploy script
* Admin contact: contact@qaul.org


### get.qaul.org Download Directory

The Download server for the binary builds and big content files such
as videos etc.

* Server: https://get.qaul.org
* Updated:
  * the builds are uploaded by CI
  * content is uploaded manually by the administrators
* Admin contact: contact@qaul.org
