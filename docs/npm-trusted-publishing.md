# npm Trusted Publishing

This document describes the planned automated npm registry publish flow for
`@zzozorang/sec-issue-finder` and its platform packages.

It does not replace the manual release checklist. Do not run this workflow for a
new version until the release owner has approved publication.

## Goal

Use npm Trusted Publishing with GitHub Actions OIDC so releases can be published
without storing a long-lived npm token and without passing a one-time password to
CI.

The first `0.1.0` preview can be published manually. Future releases should use
the `npm Publish` workflow after Trusted Publisher settings are configured on
npmjs.com.

## Packages

Configure Trusted Publishing for each package:

```text
@zzozorang/sec-issue-finder-darwin-arm64
@zzozorang/sec-issue-finder-linux-x64
@zzozorang/sec-issue-finder-win32-x64
@zzozorang/sec-issue-finder
```

Future packages such as `@zzozorang/sec-issue-finder-darwin-x64` and
`@zzozorang/sec-issue-finder-linux-arm64` need their own Trusted Publisher
configuration before they can be published by CI.

## npm Website Setup

For each package on npmjs.com:

1. Open the package settings.
2. Find the Trusted Publisher or Publishing Access settings.
3. Add a GitHub Actions trusted publisher.
4. Use these values:

```text
GitHub owner/user: ZzozoranG
Repository: sec-issue-finder
Workflow filename: npm-publish.yml
Environment: leave empty unless a protected GitHub Environment is added later
```

The workflow filename must match exactly. If the repository owner changes, update
both the npm Trusted Publisher settings and `package.json` repository metadata.

## GitHub Workflow

The workflow file is:

```text
.github/workflows/npm-publish.yml
```

It uses:

```yaml
permissions:
  contents: read
  id-token: write
```

`id-token: write` is required so GitHub Actions can request an OIDC token that
npm accepts for Trusted Publishing.

The workflow uses Node.js 24 because npm Trusted Publishing requires a recent
Node/npm combination. It does not use `NPM_TOKEN`.

## Publish Flow

The workflow:

1. Runs Rust, npm, and package content verification.
2. Builds platform packages on GitHub-hosted runners.
3. Uploads platform package tarballs as artifacts.
4. Downloads the platform tarballs in the publish job.
5. Packs the main package.
6. Publishes platform packages first.
7. Publishes the main package last.

Publish order:

```text
1. @zzozorang/sec-issue-finder-darwin-arm64
2. @zzozorang/sec-issue-finder-linux-x64
3. @zzozorang/sec-issue-finder-win32-x64
4. @zzozorang/sec-issue-finder
```

The main package must remain last because it references the platform packages via
`optionalDependencies`.

## Manual Trigger

The workflow supports manual execution with `workflow_dispatch`.

To reduce accidental publishes, the input must be:

```text
confirm_publish=publish
```

The workflow also runs for `v*` tags. Do not create a release tag until the
version, changelog, and package metadata are finalized.

## Provenance

npm Trusted Publishing from GitHub Actions automatically generates provenance for
public packages when the repository is public and the npm Trusted Publisher
configuration is valid. The workflow intentionally does not use a long-lived npm
token.

## Failure Handling

If a workflow fails after publishing some packages:

- Do not rerun blindly with the same version.
- Check which packages were published with `npm view`.
- If only platform packages were published, either publish the remaining packages
  manually or prepare a patch version.
- If the main package was published with incorrect `optionalDependencies`,
  publish a fixed patch version.
- Do not unpublish as a normal rollback strategy.

## Required Human Checks

Before enabling CI publishing for a new version:

- [ ] The version has not already been published for any package.
- [ ] npm Trusted Publisher settings exist for all four packages.
- [ ] The GitHub owner, repository, and workflow filename match npm settings.
- [ ] Release notes are updated.
- [ ] Platform artifact workflow or this publish workflow has passed.
- [ ] macOS arm64 smoke test has passed.
- [ ] Linux and Windows runtime smoke tests are either passed or explicitly
      accepted as deferred preview risk.
