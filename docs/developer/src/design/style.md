# Style Guide

A unified styleguide is meant to make the Irdest codebase more
consistent and easier for external contributors to pick up.  There are
three separate style guides contained in this document.

* Source code
* Commit messages
* Commit history

**When sending contributions, please make sure to check that your
submission adheres to these styles!**

## Source code checklist

* CI enforces formatting via `rustfmt` so please run it before sending
  an MR!
* **Add the [SDX
  header](https://git.irde.st/we/irdest/-/blob/develop/ratman/Cargo.toml#L1-2)
  to any file that are changed significantly/ created!**
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


## Commit messages

Prefix your commit message with the component name that the commit
touches.  For example: `android: perform some minor housekeeping`.

If a commit touches multiple components then you may ommit the
component name, HOWEVER consider breaking the commit up into multiple
parts if this makes sense.

Commit messages SHOULD be written in present-tense, passive voice.
For example: `irdest-core: add authentication module` or `ratman:
improve frame collection algorithm complexity`.

### Valid components

We haven't been the most consistent about this in the past but
**please try** to format your commit messages along one of these
component identifiers:

- `client/android-vpn`
- `client/echo`
- `client/mblog`
- `docs/developer`
- `docs/user`
- `docs/website`
- `docs`
- `netmod/datalink`
- `netmod/inet`
- `netmod/lan`
- `netmod/lora`
- `ratcat`
- `ratctl`
- `ratman/netmod`
- `ratman/types`
- `ratman`
- `util/<crate name>`
- ...

## Commit history

You MUST NOT use merge commits, either while merging or in your branch
history.  Rebase your changes on top of the latest `develop` HEAD
regularly to avoid conflicts that can no longer be merged.

This results in the smallest possible commit-change size.  Avoid using
squash commits whenever possible!
