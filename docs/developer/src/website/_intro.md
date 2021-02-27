# qaul.org website

The qaul website is built via the static site generator [hugo].  Its
contents and sources are part of the [qaul mono repo].


## Building the website

You need to have [hugo] installed on your system to build the website.

```console
$ hugo build # build the website for deployment
$ hugo serve # serve the website for development
```

## Build with Nix

Alternatively you can use the Nix package manager to build the
website.  The build process will create a `result` symlink to the
generated site data.

```console
$ nix build -f nix/ qaul-website
...
```

## Website structure

The website structure is somewhat non-linear and uses a lot of hugo
template features to support easy text translations.  Following is a
breakdown of the structure.

* Template
  * `qaul-theme` folder contains base HTML and CSS templates.  The
    only page not generated via these files is the root page.
  * The root page template can be found in `layouts/index.html`
* Content
  * Markdown section content can be found in the `content` directory
  * The root page content is `content/indemd` and the `content/home`
    directory (to allow multi-language versions).


[hugo]: https://gohugo.io/
[qaul mono repo]: https://git.qaul.org/qaul/qaul/
