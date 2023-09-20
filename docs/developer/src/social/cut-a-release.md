# Irdest release checklist

This page is meant for anyone on the project with release access.
Potentially this page should be moved into a Wiki or similar.

1. Make sure that CI passes on `develop`
2. Update `CHANGELOG.md` and make sure that it contains all relevant
   changes for the release.
3. Select a set of packages/ targets to include in the `Releases` section
4. Bump any relevant version numbers
5. Create a `release/{version}` tag corresponding to the new
   `ratmand`/ `libratman` version
6. Check the issue tracker and mailing list for issues that are being
   closed by this release
7. Update any relevant crates to crates.io (via `cargo release`)
8. Update the website to point to the new bundle download (as soon as
   release CI passes)
9. Re-deploy the website
10. Write a release description on the release tag
11. Optionally: write an announcement on the mailing list
