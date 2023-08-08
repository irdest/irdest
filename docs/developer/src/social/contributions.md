# How to contribute?

First of all: thank you for wanting to help out :)

The irdest source can be found in our [mono repo].  We accept
submissions via our [mailing list], and (in a more limited capacity)
via GitLab merge requests.  See sections below for details.

[mono repo]: https://git.irde.st/we/irdest
[mailing list]: https://lists.irde.st/archives/list/community@lists.irde.st/


## Reporting an issue

If you've encountered a problem using Irdest software, we would highly
appreciate it if you could tell us about it.

Since we use our own GitLab instance (and don't want to open
registrations without verification) it's hard to submit issues via
GitLab.

To submit an issue, just write an e-mail to the [community
mailinglist](mailto:community@lists.irde.st), in a format like: `[BUG]
ratman: sometimes crashes when ...` or `[QUESTION] irdest-proxy: how
to set ...`, etc.  Do please try to first search for an existing
e-mail thread in the [mail
archive](https://lists.irde.st/archives/list/community@lists.irde.st/)
though.


## Contributions via e-mail

The easiest way to contribute code is via e-mail.  This can be done in
two ways:

1. Send a patch via `git send-email`
2. Upload your contributions to a different forge/ repository, and
   send an e-mail [pull
   request](https://www.git-scm.com/docs/git-request-pull)

### Contribution via `send-email`

You can follow the guide at https://git-send-email.io/ to get yourself
set up for sending e-mail patches.

For any patch set that touches more than one component, please include
a cover-letter to explain the rationale of the changes.

### Sending an e-mail pull request

To send a pull-request via e-mail you must first upload your changes
to your own copy of the irdest repository.  You can host this anywhere
that is convenient to you (for example [GitLab](https://gitlab.com) or
[Codeberg](https://codeberg.org)).


## Contributing via GitLab merge requests

If you want an account for development, please say hi in the Matrix
channel so we know who you are.

- If a relevant issue exists, please tag in your description
- Include a short description of the accumulative changes
- If you want your history to be rebased/ merged, please clean it up
  to be useful.  Otherwise we will probably squash it.
- Feel free to open a work-in-progress MR as a place to have a
  discussion about changes or to get feedback.


## Submitting an e-mail patch

If you can't contribute via GitLab , you're very welcome to submit
your patch via our community mailing list.

The easiest way of doing this is to configure `git send-email`.

**Without git send-email**

- Send an e-mail with the title `[PATCH]: <your title here>`.
- Format your patch with `git diff -p`
- Don't send HTML e-mail!
- Make sure your line-wrapping is wide enough to allow the patch to
  stay un-wrapped!


## Lorri & direnv

You can enable automatic environment loading when you enter the
irdest repository, by configuring [lorri] and [direnv] on your system.

[lorri]: https://github.com/nix-community/lorri
[direnv]: https://direnv.net/

```console
 ❤ (uwu) ~/p/code> cd irdest
direnv: loading ~/projects/code/irdest/.envrc
direnv: export +AR +AR_FOR_TARGET +AS +AS_FOR_TARGET +CC
        // ... snip ...
 ❤ (uwu) ~/p/c/irdest> cargo build                           lorri-keep-env-hack-irdest
 ...
```
