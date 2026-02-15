---
name: north-bump-base-docker
description: Bump the base Docker image version
user_invocable: true
---

# Bump Base Docker Image

Bump the base Docker image version after modifying `docker/base/Dockerfile`.

## Steps

1. Ask the user which semver part to bump: `major`, `minor`, or `patch` (default: `patch`)
2. Run `just bump-base {part}` to update the version across all files:
   - `docker/base/VERSION`
   - `docker/dev/Dockerfile` (ARG BASE_VERSION)
   - `docker-compose.yml` (image tag + build arg)
3. Show the user the version change and updated files
4. Rebuild locally: `docker compose build`
5. Commit with message: `chore(docker): bump base image to {new_version}`
6. Ask the user to confirm before pushing

## Notes

- The test workflow (`.github/workflows/test.yml`) detects changes in `docker/base/**` and automatically builds + pushes the new base image to ghcr.io
- After bumping, run `docker compose build` locally to rebuild your dev environment
- The prod Dockerfile (`docker/prod/Dockerfile`) picks up the base version from the CI `resolve` job, so it stays in sync automatically
