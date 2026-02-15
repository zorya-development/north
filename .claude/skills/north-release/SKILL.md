---
name: north-release
description: Bump app version and prepare a release
user_invocable: true
---

# North Release

Bump the app version and push to trigger the release workflow.

## Steps

1. Ask the user which semver part to bump: `major`, `minor`, or `patch` (default: `patch`)
2. Run `just bump-version {part}` to update the version in `Cargo.toml`
3. Show the user the version change (old → new)
4. Commit with message: `chore(release): bump version to {new_version}`
5. Ask the user to confirm before pushing to master
6. If confirmed, push to master to trigger the release workflow

## Notes

- The release workflow (`.github/workflows/release.yml`) runs automatically on master push
- It checks if the git tag `v{version}` already exists — if so, it skips the release
- The workflow builds a prod Docker image, pushes to ghcr.io, generates a changelog via git-cliff, and creates a GitHub release
- Do NOT create the git tag manually — the release workflow handles that
