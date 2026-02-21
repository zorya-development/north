#!/usr/bin/env bash
set -euo pipefail

# Hybrid release note generator:
# 1. Try to extract user-facing notes from structured commits
# 2. If ANTHROPIC_API_KEY is set, enhance with AI
# 3. Always include technical details in collapsible section

VERSION="${1:-}"
if [[ -z "$VERSION" ]]; then
  echo "Usage: $0 <version>" >&2
  exit 1
fi

TEMP_DIR=$(mktemp -d)
trap 'rm -rf "$TEMP_DIR"' EXIT

TECHNICAL="$TEMP_DIR/technical.md"
USER_FACING="$TEMP_DIR/user-facing.md"
FINAL_NOTES="$TEMP_DIR/final.md"

# Get previous tag for range
PREV_TAG=$(git describe --tags --abbrev=0 2>/dev/null || echo "")
if [[ -n "$PREV_TAG" ]]; then
  COMMIT_RANGE="$PREV_TAG..HEAD"
else
  COMMIT_RANGE="HEAD"
fi

echo "📝 Generating technical changelog..."
git-cliff --config cliff.toml \
  --unreleased \
  --tag "v$VERSION" \
  --strip header \
  > "$TECHNICAL"

echo "📋 Extracting user-facing notes from commits..."
git log "$COMMIT_RANGE" --format=%B \
  | bash scripts/extract-user-notes.sh "$VERSION" \
  > "$USER_FACING"

# Check if we got any user-facing content
USER_CONTENT_LINES=$(grep -cv '^$' "$USER_FACING" || true)

# Start building final notes
{
  echo "# v$VERSION"
  echo

  if [[ "$USER_CONTENT_LINES" -gt 5 ]]; then
    echo "## Release Notes"
    echo
    cat "$USER_FACING"
    echo
  else
    echo "## 📝 Changes"
    echo
    echo "*Note: This release uses auto-generated notes. Future releases will include user-friendly summaries.*"
    echo

    # If API key is available, try AI enhancement
    if [[ -n "${ANTHROPIC_API_KEY:-}" ]]; then
      echo "🤖 Enhancing with AI..."

      read -r -d '' PROMPT << 'EOF' || true
Transform this technical changelog into 3-5 bullet points focused on user-facing changes.
For each change, write ONE concise sentence about what the user can now do or what improved.
Skip purely internal refactors. Format as markdown list with emoji prefixes (✨ for features, 🐛 for fixes, ⚡ for performance).

Technical changelog:
EOF

      AI_SUMMARY=$(curl -s https://api.anthropic.com/v1/messages \
        -H "Content-Type: application/json" \
        -H "x-api-key: $ANTHROPIC_API_KEY" \
        -H "anthropic-version: 2023-06-01" \
        -d "$(jq -n \
          --arg prompt "$PROMPT"$'\n\n'"$(cat "$TECHNICAL")" \
          '{
            model: "claude-sonnet-4-20250514",
            max_tokens: 1024,
            messages: [{role: "user", content: $prompt}]
          }')" \
        | jq -r '.content[0].text' || echo "")

      if [[ -n "$AI_SUMMARY" ]]; then
        echo "$AI_SUMMARY"
        echo
      else
        # Fallback to simple extraction
        grep -E '^\- ' "$TECHNICAL" | head -10
        echo
      fi
    else
      # No API key, just show highlights
      echo "### Highlights"
      echo
      grep -E '^\- ' "$TECHNICAL" | head -5
      echo
    fi
  fi

  # Always include technical details
  echo "---"
  echo
  echo "<details>"
  echo "<summary>🔧 Technical Details (for contributors)</summary>"
  echo
  cat "$TECHNICAL"
  echo
  echo "</details>"

} > "$FINAL_NOTES"

cat "$FINAL_NOTES"

# Save for GitHub Actions
if [[ -n "${GITHUB_OUTPUT:-}" ]]; then
  {
    echo "notes<<EOF"
    cat "$FINAL_NOTES"
    echo "EOF"
  } >> "$GITHUB_OUTPUT"
fi
