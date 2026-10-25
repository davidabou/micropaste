# Code signing policy

Free code signing provided by [SignPath.io](https://about.signpath.io), certificate by [SignPath Foundation](https://signpath.org).

## What is signed

The Windows installer (`Micropaste_x.y.z_x64-setup.exe`) published in the [GitHub releases](https://github.com/davidabou/micropaste/releases) and through the Microsoft Store.

## How it is built

Every signed installer is built by [GitHub Actions](.github/workflows/release.yml) from the source code of this repository, from a version tag. Nothing built on a personal computer is signed. Each signing request is approved manually before the release is published.

## Team roles

| Role | Members |
| --- | --- |
| Committers and reviewers | [David Abou](https://github.com/davidabou) |
| Approvers | [David Abou](https://github.com/davidabou) |

All members use multi-factor authentication on GitHub and SignPath.

## Privacy

See the [privacy policy](PRIVACY.md): Micropaste keeps your clips on your computer and sends nothing to its author.

## Reporting a problem

If you think a file signed for Micropaste is malicious or wasn't built from this repository, don't run it and [open an issue](https://github.com/davidabou/micropaste/issues) with where you downloaded it and its SHA-256 hash.
