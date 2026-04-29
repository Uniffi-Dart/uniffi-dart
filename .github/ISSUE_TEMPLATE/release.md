---
name: Release
about: Checklist for releasing a new version of uniffi-dart
title: "Release vX.Y.Z+vA.B.C"
labels: release
assignees: ''
---

## Release checklist

Replace `X.Y.Z` with the uniffi-dart version and `A.B.C` with the
targeted uniffi-rs version throughout. Each supported uniffi-rs
version has its own `release/uniffi-vA.B.x` branch.

### Starting a new uniffi-rs line (skip if `release/uniffi-vA.B.x` exists)

If you are cutting the first release on a new uniffi-rs line, first
preserve the current line and bump `main` to the new target:

- [ ] Create a release branch from `main` for the current line (this
      preserves it before `main` is bumped):
      ```
      git switch main && git pull
      git switch -c release/uniffi-v<current>.x
      git push -u origin release/uniffi-v<current>.x
      ```
- [ ] Open a PR against `main` that bumps the workspace uniffi
      dependencies in the root `Cargo.toml`, the uniffi dependency in
      each fixture `Cargo.toml`, the `README.md` install snippet to
      `vX.Y.Z+vA.B.C`, and the `README.md` versioning table (add a
      new row pointing at `vX.Y.Z+vA.B.C`). The README values point
      at the upcoming release; the actual tag lands later in this
      checklist. Merge when green.
- [ ] Create the new line's release branch from the bumped `main`:
      ```
      git switch main && git pull
      git switch -c release/uniffi-v<new>.x
      git push -u origin release/uniffi-v<new>.x
      ```

### Pick the release line

- [ ] uniffi-rs line being released: `vA.B.x`
- [ ] Target branch: `release/uniffi-vA.B.x`. If `main` is still on
      `vA.B.x`, target `main` instead and fast-forward
      `release/uniffi-vA.B.x` to it after merge (see Tag & release).
- [ ] `git switch <target-branch> && git pull`
- [ ] Latest tag on this line:
      `git describe --tags --match 'v*+vA.B.*' --abbrev=0`
- [ ] If this release incorporates a uniffi-rs patch bump (e.g.
      `0.30.0` to `0.30.1`), land that as a regular PR against this
      branch first, updating the workspace uniffi deps in the root
      `Cargo.toml` and the `uniffi = "A.B"` lines in fixture
      `Cargo.toml` files.

### Version bump

- [ ] Determine version: bump **minor** for breaking changes, **patch**
      otherwise. Version lines are independent (e.g. 0.1.x on the 0.30
      line and 0.2.x on the 0.31 line is a valid state).
- [ ] Update `version` in `Cargo.toml` to `X.Y.Z+vA.B.C`. Keep the
      exact uniffi-rs patch from workspace deps; do not invent a bump.
- [ ] Update `version` in `uniffi_dart_macro/Cargo.toml` to match

### Documentation

- [ ] Update the installation snippet in `README.md` to reflect the latest
      line.
- [ ] Update the row for this line in the `README.md` versioning table
- [ ] Add a `## vX.Y.Z+vA.B.C` section to `CHANGELOG.md`. Prefix
      breaking changes with `**BREAKING**:` and critical fixes with
      `**IMPORTANT**:`.

### Review & merge

- [ ] Open a PR with the above changes against the target branch
- [ ] CI green
- [ ] Merge

### Tag & release

- [ ] If you targeted `main`, fast-forward `release/uniffi-vA.B.x`
      to it and push:
      ```
      git switch release/uniffi-vA.B.x && git pull
      git merge --ff-only main
      git push origin release/uniffi-vA.B.x
      ```
- [ ] Create a git tag on the merge commit: `vX.Y.Z+vA.B.C`
- [ ] Push the tag
- [ ] Create a GitHub Release from the tag with the changelog entry

### Backport to other lines

Review commits new to this release and decide which other
`release/uniffi-vX.Y.x` branches should receive them. Version bumps,
CHANGELOG entries, and README install-snippet updates are
line-specific and should not be backported; only the underlying fixes
or features.

For each other release branch that should receive a commit:

- [ ] `git switch release/uniffi-vX.Y.x && git pull`
- [ ] `git cherry-pick -x <sha>` (the `-x` annotates the backport
      commit with the original SHA so history is traceable)
- [ ] Resolve any conflicts (typically uniffi version pins in fixture
      `Cargo.toml` files)
- [ ] Open a backport PR titled
      `[backport uniffi-vX.Y.x] <original title>`
- [ ] CI green, merge

Do not merge one `release/uniffi-vX.Y.x` branch into another, and do
not merge `main` into a release branch. Both drag in changes to
workspace uniffi pins, which is the whole thing the branches exist to
keep separate. Cherry-pick only.
