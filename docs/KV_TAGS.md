# Plan: Key:Value Tags

## Context

The app currently has flat tags (`urgent`, `shopping`). The user wants structured k:v tags like `priority:high`, `type:bug`, `complexity:easy`. When filtering, values within the same key should work as OR (selecting `priority:high` and `priority:medium` shows tasks matching either), while different keys use AND logic. This is the standard **faceted filtering** pattern (Kubernetes labels, GitHub labels, e-commerce filters).

**Key insight**: No database or schema changes needed. The existing tag system already stores tags by `name` text, and colons are already valid characters in tag names (confirmed by `tag_tokens.rs` test on line 119). The work is purely in parsing utilities, display formatting, and filter logic.

## Changes

### 1. Utility functions in `crates/dto/src/tag.rs`

Add:
- `parse_kv(name: &str) -> Option<(&str, &str)>` — splits `"priority:high"` into `("priority", "high")`, returns `None` for simple tags
- `group_tag_names(names: &[String]) -> (Vec<String>, BTreeMap<String, Vec<String>>)` — groups selected tag names into simple tags and k:v groups (for filter logic)

These are pure functions, no IO, work on both server and WASM.

### 2. TagFilterRow: Faceted display (`crates/app/src/containers/traversable_task_list/components/tag_filter_row.rs`)

**Current**: Flat list of `#tagname` buttons, each toggling individually.

**New**: k:v tags display as **grouped chips** in the toolbar:
- k:v tags show as `key:all` by default (one chip per key)
- Clicking opens a **Popover** with checkboxes for each value under that key
- When values are selected, chip shows `key:value1,value2`
- Simple tags (no colon) remain as flat toggle buttons, same as today

The `available_tags` prop already provides `Vec<(String, String)>` (name, color). Group these using `parse_kv()` before rendering.

### 3. Filter logic in TTL controller (`crates/app/src/containers/traversable_task_list/controller.rs`, ~line 179)

**Current** (line 182-185): All active tags must be present (AND):
```rust
if !active_tags.iter().all(|req| tag_names.contains(&req.as_str())) { continue; }
```

**New**: Faceted logic:
- Group `active_tags` by key using `group_tag_names()`
- For each k:v group: task must have **at least one** of the selected values (OR)
- For simple tags: task must have the tag (AND between different simple tags)
- Between all groups: AND

```rust
fn matches_faceted(task_tags: &[&str], active: &[String]) -> bool {
    let (simple, kv_groups) = group_tag_names(active);
    // Simple tags: AND
    for name in &simple {
        if !task_tags.contains(&name.as_str()) { return false; }
    }
    // k:v groups: OR within, AND between
    for (_key, values) in &kv_groups {
        if !values.iter().any(|v| task_tags.contains(&v.as_str())) { return false; }
    }
    true
}
```

This same logic applies in both the TaskTreeView path (~line 179) and the flat-mode path (~line 265).

### 4. Tag display formatting

**`task_list_item/view.rs`** (line 242-267): Currently shows `#tagname` as link. For k:v tags, show `#key:value` with key dimmed:
- Use `parse_kv()` on the tag name
- If k:v: render key in `text-text-tertiary` and value portion normally
- If simple: render as today

**`task_meta/view.rs`** (line 97-110): Currently shows `tag.name` with `tag.color`. Same treatment — format k:v tags with dimmed key prefix.

### 5. Tag Picker (`crates/app/src/containers/tag_picker/view.rs`)

Keep flat list (user's choice). No grouping. k:v tags show as `priority:high` in the list — they already do since the name includes the colon. The "New tag..." input already accepts colons.

One small enhancement: sort k:v tags by key prefix so they appear grouped naturally (alphabetical sort already does this via BTreeMap in the controller).

## Files to modify

| File | Change |
|------|--------|
| `crates/dto/src/tag.rs` | Add `parse_kv()`, `group_tag_names()` |
| `crates/app/src/containers/traversable_task_list/components/tag_filter_row.rs` | Grouped k:v chips with popover multiselect |
| `crates/app/src/containers/traversable_task_list/controller.rs` | Faceted filter logic (~line 179 and ~line 265) |
| `crates/app/src/containers/task_list_item/view.rs` | Format k:v tag display (~line 242) |
| `crates/app/src/containers/task_meta/view.rs` | Format k:v tag display (~line 97) |

## What does NOT change

- Database schema (no migration)
- DB models (`TagRow`, `NewTag`)
- Core services (`TagService`)
- Token parsing (colons already supported)
- Filter DSL (already works: `tags = "priority:high"`, `tags =~ "priority:*"`)
- Server functions, repositories
- Tag Picker popover (stays flat per user preference)

## Implementation order (TDD: test first, code after)

### Step 1: `parse_kv()` — RED then GREEN
1. Write tests in `crates/dto/src/tag.rs` `#[cfg(test)]` module:
   - `"priority:high"` → `Some(("priority", "high"))`
   - `"urgent"` → `None` (no colon)
   - `":value"` → `None` (empty key)
   - `"key:"` → `None` (empty value)
   - `"a:b:c"` → `Some(("a", "b:c"))` (only first colon splits)
2. Run `just test dto` — tests fail (RED)
3. Implement `parse_kv()` — tests pass (GREEN)

### Step 2: `group_tag_names()` — RED then GREEN
1. Write tests:
   - Mixed input `["priority:high", "priority:low", "urgent", "type:bug"]` → simple: `["urgent"]`, kv: `{"priority": ["priority:high", "priority:low"], "type": ["type:bug"]}`
   - Empty input → both empty
   - Only simple tags → kv empty
   - Only k:v tags → simple empty
2. Run `just test dto` — tests fail (RED)
3. Implement `group_tag_names()` — tests pass (GREEN)

### Step 3: `matches_faceted_filter()` — RED then GREEN
1. Write tests (also in dto or as a standalone fn):
   - Task with `["priority:high", "type:bug"]`, filter `["priority:high", "priority:low"]` → `true` (OR within key)
   - Task with `["priority:high"]`, filter `["priority:high", "type:bug"]` → `false` (AND between keys)
   - Task with `["priority:high", "type:bug"]`, filter `["priority:high", "type:bug"]` → `true`
   - Task with `["urgent"]`, filter `["urgent"]` → `true` (simple tag)
   - Empty filter → `true` (matches all)
   - Mixed: task `["priority:high", "urgent"]`, filter `["priority:high", "urgent"]` → `true`
2. Run tests — fail (RED)
3. Implement — pass (GREEN)

### Step 4: Wire faceted filter into TTL controller
- Replace AND-all logic at `controller.rs:~179` and `~265` with `matches_faceted_filter()`
- Run `just test` to verify no regressions

### Step 5: TagFilterRow — k:v grouped chips with popover
- Modify `tag_filter_row.rs` to group k:v tags, show `key:all` chips with popover multiselect
- Simple tags stay as flat toggle buttons

### Step 6: Tag display formatting
- `task_list_item/view.rs`: k:v tags show with dimmed key prefix
- `task_meta/view.rs`: same treatment

### Step 7: E2E test
- Write Playwright test covering:
  - Create tasks with k:v tags via `#priority:high` etc.
  - Verify faceted filter chips appear in toolbar
  - Click chip, select values, verify OR filtering
  - Combine multiple keys, verify AND between keys

## Verification

1. `just test dto` — after each RED/GREEN step
2. `just test` — after step 4 (no regressions)
3. `just dev` — manual check after steps 5-6
4. `just playwright-exec --grep "kv tag"` — after step 7
