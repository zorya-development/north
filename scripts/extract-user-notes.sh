#!/usr/bin/env bash
set -euo pipefail

# Extract user-facing notes from commit messages
# Commits should have format:
#   feat: short summary
#
#   User-facing: What users will notice
#
#   Technical details:
#   - implementation
#
# Usage: git log --format=%B | ./scripts/extract-user-notes.sh

VERSION="${1:-unreleased}"
OUTPUT_FORMAT="${2:-github}" # github or markdown

declare -A sections
sections=(
  ["feat"]="### ✨ New Features"
  ["fix"]="### 🐛 Bug Fixes"
  ["perf"]="### ⚡ Performance"
)

echo "## $VERSION"
echo

# Track which sections have content
declare -A has_content

current_commit=""
commit_type=""
commit_summary=""
user_facing=""
is_breaking=false

while IFS= read -r line; do
  # Detect commit type from first line
  if [[ "$line" =~ ^(feat|fix|perf|refactor|docs|test|chore):[[:space:]](.+)$ ]]; then
    # Save previous commit if it had user-facing content
    if [[ -n "$user_facing" && -n "$commit_type" ]]; then
      if [[ -z "${has_content[$commit_type]:-}" ]]; then
        echo "${sections[$commit_type]}"
        echo
        has_content[$commit_type]=1
      fi

      if [[ "$is_breaking" == true ]]; then
        echo "- ⚠️ **BREAKING:** $user_facing"
      else
        echo "- $user_facing"
      fi
      echo
    fi

    # Start new commit
    commit_type="${BASH_REMATCH[1]}"
    commit_summary="${BASH_REMATCH[2]}"
    user_facing=""
    is_breaking=false

  # Extract user-facing line
  elif [[ "$line" =~ ^User-facing:[[:space:]](.+)$ ]]; then
    user_facing="${BASH_REMATCH[1]}"

  # Detect breaking change
  elif [[ "$line" =~ ^BREAKING[[:space:]]CHANGE: ]]; then
    is_breaking=true

  # Skip other lines (technical details, empty lines, etc.)
  fi
done

# Handle last commit
if [[ -n "$user_facing" && -n "$commit_type" ]]; then
  if [[ -z "${has_content[$commit_type]:-}" ]]; then
    echo "${sections[$commit_type]}"
    echo
  fi

  if [[ "$is_breaking" == true ]]; then
    echo "- ⚠️ **BREAKING:** $user_facing"
  else
    echo "- $user_facing"
  fi
  echo
fi

# Add upgrade guide if there were breaking changes
if [[ -n "${has_content[breaking]:-}" ]]; then
  echo "### 📦 Upgrade Guide"
  echo
  echo "This release contains breaking changes. See details above."
  echo
else
  echo "### 📦 Upgrade"
  echo
  echo "No breaking changes. Pull the latest image and restart:"
  echo '```bash'
  echo "docker pull ghcr.io/zorya-development/north:v$VERSION"
  echo "docker compose up -d"
  echo '```'
  echo
fi
