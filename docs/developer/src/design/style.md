# Style Guide

A unified styleguide is meant to make the codebase and irdest
repository more consistent, and thus hopefully easier for external
contributors to pick up.  There are three separate style guides
contained in this document.

* Source code
* Commit messages
* Commit history


## Source Code

In the Rust source code we mostly follow the convenions set out by the
[rustfmt] project.  Code lines MUST be shorter than 80 characters.
Comments MUST be shorter than 100 characters.  If a function signature
becomes longer than 80 characters it is broken up into multiple lines.
Again: **use rustfmt**!

**Some notes**

* If a code block (for example a complex `match`) becomes less
  readable by letting `rustfmt` format it you MAY annotate it with
  `#[rustfmt(ignore)]`!
* Use named generic parameters in public facing APIs (So `A: Into<A>`
  instead of `impl Into<A>`) as this improves inter-operability of
  function API types.
* Structuring imports via nested `{ }` blocks is encouraged.  You
  however MUST NOT condense all imports into a single `use` statement!
  * Generally: try to make the imports look "pretty" and easy to parse
    for another contributor (this is vague I know - sorry)!
  * TODO: some examples


## Commit messages

Prefix your commit message with the component name that the commit
touches.  For example: `android: perform some minor housekeeping`.  If
a commit touches multiple components then you may ommit the component
name, HOWEVER consider breaking the commit up into multiple parts if
this makes sense.

Commit messages SHOULD be written in present-tense, passive voice.
For example: `irdest-core: add authentication module` or `ratman:
improve frame collection algorithm complexity`.


## Commit history

You MUST NOT use merge commits, either while merging or in your branch
history.  Rebase your changes on top of the latest `develop` HEAD
regularly to avoid conflicts that can no longer be merged.

This should result in the smallest possibly commit-change size.  Avoid
using squash commits whenever possible!
