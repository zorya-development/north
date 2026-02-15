default:
    @just --list

dev:
    cargo leptos watch

build:
    cargo leptos build --release

test *args:
    cargo test {{ args }}

fmt:
    cargo fmt

fmt-check:
    cargo fmt --check

lint:
    cargo clippy -- -D warnings

migrate:
    diesel migration run

migration name:
    diesel migration generate {{ name }}

migration-diff name:
    diesel migration generate --diff-schema {{ name }}

migrate-revert:
    diesel migration revert

migrate-redo:
    diesel migration redo

seed:
    cargo run --bin north-server --features ssr -- --seed

check: fmt-check lint test

# Bump base image version: just bump-base {major,minor,patch}
bump-base part:
    #!/usr/bin/env bash
    set -euo pipefail
    current=$(cat docker/base/VERSION | tr -d '[:space:]')
    IFS='.' read -r major minor patch <<< "$current"
    case "{{ part }}" in
        major) major=$((major + 1)); minor=0; patch=0 ;;
        minor) minor=$((minor + 1)); patch=0 ;;
        patch) patch=$((patch + 1)) ;;
        *) echo "Usage: just bump-base {major,minor,patch}"; exit 1 ;;
    esac
    version="$major.$minor.$patch"
    echo "$version" > docker/base/VERSION
    sed -i "s|^ARG BASE_VERSION=.*|ARG BASE_VERSION=$version|" docker/dev/Dockerfile
    sed -i "s|image: north-base:.*|image: north-base:$version|" docker-compose.yml
    sed -i "s|BASE_VERSION: \".*\"|BASE_VERSION: \"$version\"|" docker-compose.yml
    echo "Base image version bumped: $current → $version"
    echo "Files updated: docker/base/VERSION, docker/dev/Dockerfile, docker-compose.yml"
    echo "Run 'docker compose build' to rebuild locally"
