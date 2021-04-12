# How to contribute?

First of all: thank you for wanting to help out :)

The irdest source can be found in our [mono repo].  We accept
submissions via GitLab merge requests, ~~or via patches sent to our
mailing list~~ (we currently have no mailing list).

[mono repo]: https://git.irde.st/we/irdest


## Submitting an MR

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
