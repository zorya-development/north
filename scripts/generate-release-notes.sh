#!/usr/bin/env bash
set -euo pipefail

# Generate release notes:
# - If ANTHROPIC_API_KEY is set: AI-enhanced user-friendly notes
# - Otherwise: Use git-cliff technical changelog as-is

VERSION="${1:-}"
if [[ -z "$VERSION" ]]; then
  echo "Usage: $0 <version>" >&2
  exit 1
fi

TEMP_DIR=$(mktemp -d)
trap 'rm -rf "$TEMP_DIR"' EXIT

TECHNICAL="$TEMP_DIR/technical.md"

# Get previous tag for range
PREV_TAG=$(git describe --tags --abbrev=0 "v$VERSION^" 2>/dev/null || echo "")
if [[ -n "$PREV_TAG" ]]; then
  COMMIT_RANGE="$PREV_TAG..v$VERSION"
  echo "📝 Generating changelog for range: $COMMIT_RANGE" >&2
else
  COMMIT_RANGE="v$VERSION"
  echo "📝 Generating changelog for tag: $COMMIT_RANGE" >&2
fi

# Generate technical changelog with git-cliff
git-cliff --config cliff.toml \
  --strip header \
  "$COMMIT_RANGE" \
  > "$TECHNICAL"

# If no API key, just output technical notes
if [[ -z "${ANTHROPIC_API_KEY:-}" ]]; then
  echo "ℹ️  No ANTHROPIC_API_KEY set, using technical changelog" >&2
  cat "$TECHNICAL"

  if [[ -n "${GITHUB_OUTPUT:-}" ]]; then
    {
      echo "notes<<EOF"
      cat "$TECHNICAL"
      echo "EOF"
    } >> "$GITHUB_OUTPUT"
  fi
  exit 0
fi

# AI enhancement
echo "🤖 Enhancing with Claude API..." >&2

read -r -d '' PROMPT << 'EOF' || true
You are a technical writer creating user-friendly release notes for a GTD task management app called North.

Transform the technical changelog below into engaging, scannable release notes.

# Output Format

Start with a catchy title summarizing the main theme, then organize into sections:

### 🎉 Highlights
Pick the top 2-3 most exciting user-facing changes. Focus on benefits, not implementation.
Each highlight should be 1-2 sentences explaining what the user can now do.

### ✨ New Features
List user-facing features. Each should:
- Start with what the user can now do
- Be 1-2 sentences max
- Avoid technical jargon (no component names, no architecture terms)
- Focus on the benefit

### 🐛 Bug Fixes
List fixes users would notice. Skip internal refactors. Be specific about what was broken and is now fixed.

### ⚡ Performance
Only include if there are noticeable speed improvements. Quantify if possible.

### 📦 Upgrade Notes
- State if there are breaking changes (look for "breaking" or "remove" in changelog)
- If no breaking changes, say "No breaking changes. Pull the latest image and restart."
- If breaking changes exist, list migration steps

---

<details>
<summary>🔧 Technical Details (for contributors)</summary>

[Include the full technical changelog here, unchanged]

</details>

# Rules
- Write for end users, not developers
- Use present tense, active voice
- Be specific (not "improved navigation" but "navigate with arrow keys")
- Skip purely internal changes from main sections (put only in Technical Details)
- If a change is too technical to explain to users, skip it from main sections
- Keep highlights to 3 items maximum
- Use emojis only in section headers
- Be concise but clear

# Technical Changelog to Transform

EOF

# Combine prompt with technical changelog
FULL_PROMPT="$PROMPT

$(cat "$TECHNICAL")"

# Call Claude API
RESPONSE=$(curl -s https://api.anthropic.com/v1/messages \
  -H "Content-Type: application/json" \
  -H "x-api-key: $ANTHROPIC_API_KEY" \
  -H "anthropic-version: 2023-06-01" \
  -d "$(jq -n \
    --arg prompt "$FULL_PROMPT" \
    '{
      model: "claude-sonnet-4-20250514",
      max_tokens: 4096,
      messages: [
        {
          role: "user",
          content: $prompt
        }
      ]
    }')")

# Extract content
AI_NOTES=$(echo "$RESPONSE" | jq -r '.content[0].text' 2>/dev/null || echo "")

if [[ -z "$AI_NOTES" || "$AI_NOTES" == "null" ]]; then
  echo "❌ AI enhancement failed, falling back to technical notes" >&2
  echo "Response: $RESPONSE" >&2
  cat "$TECHNICAL"

  if [[ -n "${GITHUB_OUTPUT:-}" ]]; then
    {
      echo "notes<<EOF"
      cat "$TECHNICAL"
      echo "EOF"
    } >> "$GITHUB_OUTPUT"
  fi
  exit 0
fi

echo "✅ Release notes generated!" >&2
echo "$AI_NOTES"

# Save to GitHub Actions output
if [[ -n "${GITHUB_OUTPUT:-}" ]]; then
  {
    echo "notes<<EOF"
    echo "$AI_NOTES"
    echo "EOF"
  } >> "$GITHUB_OUTPUT"
fi
