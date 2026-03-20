# Automated Release Notes System

## Quick Start

### Option 1: Structured Commits (Recommended)

Write commits with `User-facing:` sections:

```bash
git commit -m "feat: add keyboard navigation

User-facing: Navigate tasks with arrow keys - Up/Down for siblings,
Left/Right for parent/child. Press Enter to edit.

Technical details:
- Add TraversableTaskList component
- Implement cursor-based navigation
- Add keyboard event handlers"
```

**Result:** Clean user-facing notes automatically extracted. No API needed.

### Option 2: AI Enhancement (Fallback)

If you forget the `User-facing:` section, add an API key and the system will auto-generate user-friendly notes:

```bash
# In GitHub repo settings → Secrets → Actions
# Add: ANTHROPIC_API_KEY = sk-ant-...
```

**Result:** AI rewrites technical commits into user-focused release notes.

---

## How It Works

### Architecture

```
Commit Messages
    ↓
┌─────────────────────────────────────┐
│ git-cliff (cliff.toml)              │ ← Generates technical changelog
│            ↓                        │
│ generate-release-notes.sh           │ ← AI enhances if ANTHROPIC_API_KEY set
│            ↓                        │
│ Final release notes                 │ ← Markdown output
└─────────────────────────────────────┘
    ↓
GitHub Release
```

> **Note:** `scripts/generate-release-notes-v2.sh` implements the full hybrid approach (parsing `User-facing:` sections + AI fallback) described in this doc, but CI currently uses `scripts/generate-release-notes.sh` which takes pre-generated git-cliff output and optionally AI-enhances it.

### Files

| File | Purpose |
|------|---------|
| `scripts/generate-release-notes.sh` | CI orchestrator (AI-enhances git-cliff output) |
| `scripts/generate-release-notes-v2.sh` | Hybrid orchestrator (parses `User-facing:` + AI fallback) |
| `scripts/extract-user-notes.sh` | Parse structured commits (used by v2) |
| `cliff.toml` | Technical changelog config |
| `.github/workflows/release.yml` | CI integration |
| `.github/commit-template.md` | Commit message template |

---

## Usage

### Local Testing

```bash
# Test release notes for next version
./scripts/generate-release-notes-v2.sh 0.3.1

# With AI enhancement
export ANTHROPIC_API_KEY="sk-ant-..."
./scripts/generate-release-notes-v2.sh 0.3.1
```

### Automated (GitHub Actions)

When you run `just bump-version`, the release workflow automatically:

1. Generates release notes using `generate-release-notes.sh`
2. Uses API key if available (set `ANTHROPIC_API_KEY` in repo secrets)
3. Creates GitHub release with formatted notes

---

## Commit Message Format

### Structure

```
<type>: <short summary>

User-facing: <what users will notice>

[Optional: BREAKING CHANGE: <description>]

Technical details:
- <implementation detail 1>
- <implementation detail 2>
```

### Types

| Type | Category | Visible to Users |
|------|----------|------------------|
| `feat:` | ✨ New Features | Yes |
| `fix:` | 🐛 Bug Fixes | Yes |
| `perf:` | ⚡ Performance | Yes |
| `refactor:` | 🔧 Technical | Collapsed |
| `docs:` | 📚 Documentation | Collapsed |
| `test:` | 🔧 Technical | Collapsed |
| `chore:` | 🔧 Technical | Collapsed |

### Examples

**Good commit:**
```
feat: add inline task creation

User-facing: Create tasks directly in the list without opening a modal.
After adding the first task, the input stays open for rapid entry.

Technical details:
- Remove TaskCreateModal component
- Add CreateTop mode to TraversableTaskList
- Implement TtlHandle for imperative control
```

**Minimal commit (AI will enhance if key is available):**
```
fix: cursor jumping after task completion

Advance cursor to next sibling before completing task to prevent orphaned state.
```

---

## Output Format

### User-Facing Section (Main Release Notes)

```markdown
# v0.3.0

## Release Notes

### ✨ New Features

- Create tasks directly in the list without opening a modal.
  After adding the first task, the input stays open for rapid entry.
- Navigate tasks with arrow keys - Up/Down for siblings, Left/Right
  for parent/child.

### 🐛 Bug Fixes

- Fixed cursor jumping after task completion.
- Fixed drag indicator flickering between tasks.

### 📦 Upgrade

No breaking changes. Pull latest image:
\`\`\`bash
docker pull ghcr.io/zorya-development/north:v0.3.0
\`\`\`
```

### Technical Section (Collapsed)

```markdown
---

<details>
<summary>🔧 Technical Details (for contributors)</summary>

## [0.3.0] - 2026-02-20

### Features

- Add TraversableTaskList with keyboard navigation
  - Cursor-based tree model with Up/Down/Left/Right
  - Enter to edit, Ctrl+Enter to create
  - Tab/Shift+Tab for indent/outdent
...

</details>
```

---

## Configuration

### Enable AI Enhancement

1. Get API key from https://console.anthropic.com
2. Add to GitHub repo:
   - Settings → Secrets and variables → Actions
   - New repository secret: `ANTHROPIC_API_KEY`
   - Value: `sk-ant-...`

Cost: ~$0.01 per release (Claude Sonnet processes ~2KB changelog)

### Customize Output

Edit `scripts/generate-release-notes.sh` (or `generate-release-notes-v2.sh` for the hybrid version):

```bash
# Change AI model
model: "claude-sonnet-4-20250514"  # Fast, cheap
# or
model: "claude-opus-4-20250514"    # Higher quality

# Adjust max tokens for longer changelogs
max_tokens: 4096  # Increase if needed
```

Edit `cliff.toml` for technical changelog format:

```toml
# Add custom commit parsers
{ message = "^wip", skip = true },
{ message = "^hotfix", group = "<!-- 1 -->Bug Fixes" },
```

---

## Migration Guide

### From Current System

You have two options:

**A) Start using structured commits (recommended)**

1. Use commit template: `git config commit.template .github/commit-template.md`
2. Add `User-facing:` to your commits
3. No API key needed

**B) Add API key for automatic enhancement**

1. Add `ANTHROPIC_API_KEY` to repo secrets
2. Continue committing as usual
3. AI will generate user-friendly notes

**C) Mix both (best of both worlds)**

1. Add API key as backup
2. Write `User-facing:` sections when you remember
3. AI fills in gaps for commits without structured sections

### Next Release

Your next release will automatically use the new system. No changes to `just bump-version` workflow needed.

---

## Troubleshooting

### "No release notes generated"

- Check commit range: `git log v0.3.0..HEAD --oneline`
- Verify cliff.toml syntax: `git-cliff --config cliff.toml --unreleased`
- Test locally: `./scripts/generate-release-notes.sh test`

### "AI enhancement failed"

- Verify API key is set: `echo $ANTHROPIC_API_KEY`
- Check API quota: https://console.anthropic.com
- System falls back to git-cliff output automatically

### "Release notes are too technical"

- Add `User-facing:` sections to commits
- Or set `ANTHROPIC_API_KEY` for AI rewriting
- Edit release notes manually on GitHub after creation

---

## Best Practices

### DO

✅ Write `User-facing:` sections as you commit
✅ Focus on user benefits, not implementation
✅ Use simple language (avoid jargon)
✅ One clear idea per feature/fix
✅ Include breaking changes with `BREAKING CHANGE:`

### DON'T

❌ Wait until release to write notes
❌ Use technical terms in user-facing sections
❌ Be vague ("fixed bug", "updated component")
❌ Mix user and technical content
❌ Skip upgrade instructions for breaking changes

---

## Examples

See `docs/examples/release-notes-examples.md` for:
- Before/after commit transformations
- Good vs bad user-facing notes
- Breaking change handling
- AI enhancement examples

---

## Further Reading

- **Commit Format:** `.github/commit-template.md`
- **Examples:** `docs/examples/release-notes-examples.md`
- **Writing Guide:** `docs/RELEASE_NOTES.md`
- **git-cliff Docs:** https://git-cliff.org
- **Claude API:** https://docs.anthropic.com
