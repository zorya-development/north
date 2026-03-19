# Task Detail Modal Fixes — Status Update

## Completed

### 1. Token parsing merged into `TaskService::create()` and `update()`
- Moved `#tag` and `@project` parsing from `create_with_tokens`/`update_with_tokens` into `create()`/`update()`
- Introduced private `update_raw()` to avoid infinite recursion from background URL resolution
- Deleted `_with_tokens` variants from core, server-fns, repositories, and stores
- REST API now gets token parsing for free — no code changes needed in route handlers
- **Compiles clean, no regressions**

### 2. "@Inbox" prefix on All Tasks page
- Tasks without a project now show `@Inbox` (tertiary color, links to `/inbox`) on the All Tasks page
- Updated `task_list_item/view.rs` with an else branch when `project_id` is `None`
- Updated `all-tasks.spec.ts` to expect `@Inbox` instead of no prefix

### 3. Popover opacity fix
- Removed `opacity: 0 → 1` pattern from `Popover` component (`crates/ui/src/popover.rs`)
- Panel now relies on `display: none/block` for visibility instead of opacity trick
- This was causing Playwright to see elements as "not visible" during the opacity-0 phase

## Blocked — 5 E2E tests still failing

All failures are in `task-detail-modal.spec.ts`:

| Test | Root cause |
|------|-----------|
| change project | Popover content not actionable after trigger click |
| unset project | Same popover issue (now navigates to `/all-tasks` correctly) |
| change start date | Same popover issue |
| add tag | `fill()` fails — input inside popover not interactable |
| remove tag | `tag-remove` button stays in DOM after click (store re-renders TagPicker with stale props before server round-trip completes) |

### Core issue: Popover positioning Effect timing
The `Popover` component uses a Leptos `Effect` to position the panel after `display: block`. Playwright's actionability checks (`click`, `fill`) require the element to be stable and fully rendered. The Effect runs asynchronously (next microtask), so there's a gap where the panel has `display: block` but no `top`/`left` set — the panel renders at default position (top-left corner), possibly off-screen or overlapping the backdrop.

**Likely fix:** Set initial `top`/`left` to match the trigger position via inline style, or use `requestAnimationFrame` callback to signal readiness, or add a small `waitForTimeout` in tests after popover opens.

### Secondary issue: TagPicker re-initialization
The task detail modal's reactive closure re-creates `TagPicker` on every store update, resetting `current_tags` from the (potentially stale) store data. The local optimistic removal gets overwritten by a re-render before the server round-trip completes.

**Likely fix:** Make `TagPicker` accept reactive `Signal<Vec<TagInfo>>` instead of `Vec<TagInfo>`, or debounce the re-render.

## Not in scope (pre-existing failures)
- `settings.spec.ts` timezone test
- `task-url-links.spec.ts` URL resolution test
