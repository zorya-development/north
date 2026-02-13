# PRD: Drag-and-Drop Task Ordering & Nesting

## Problem Statement

North currently supports task ordering via a `position: i32` field and nesting via `parent_id`, but the UI provides no way for users to manually reorder tasks or reorganize hierarchy through direct manipulation. Users must rely on creation order or the task detail modal to change a task's parent. This makes organizing tasks in Inbox and Project views cumbersome — especially for GTD workflows where reprioritizing and grouping tasks is a core activity.

## Goals

1. Let users reorder tasks by dragging within Inbox and Project views
2. Let users nest/unnest tasks by dragging onto other tasks
3. Keep Today, All Tasks, Filter, and Review views read-only (ordering is derived from dates/filters, not manual position)
4. Maintain optimistic, responsive UI — no waiting for server round-trips during drag

## Non-Goals

- Kanban column drag-and-drop (already has column_id support, separate feature)
- Touch/mobile drag support (HTML5 DnD doesn't support touch; defer to future phase)
- Cross-page dragging (e.g., dragging a task from Inbox to a Project sidebar link)
- Drag-and-drop in the SubtaskList inside the task detail modal (too cramped; keep existing up/down or manual reorder)

---

## Current Implementation Review & Issues

### What exists

| Aspect | Current State |
|--------|---------------|
| Position field | `position: i32`, sequential integers (0, 1, 2, ...) |
| Position on create | `max(position) + 1` among siblings |
| MoveTask DTO | `{ column_id, position, parent_id }` |
| `move_task()` service | Sets new values directly — **does not shift siblings** |
| Task list rendering | Flat list of `TaskCard` components, no nesting shown inline |
| Subtask rendering | Only inside `TaskDetailModal` via `SubtaskList` component |
| Ordering contexts | Inbox: `position ASC`, Project: `position ASC`, Today: `start_at ASC`, All: `position ASC` |

### Issues identified

1. **`move_task()` doesn't reorder siblings.** Setting `position=2` on a task doesn't shift the task currently at position 2 to position 3. Two tasks can share the same position, producing undefined sort order.

2. **Integer positions require O(n) updates per reorder.** Inserting between positions 2 and 3 requires either:
   - Shifting all tasks at position >= 3 (O(n) writes), or
   - Using gaps (fragile, needs rebalancing)

3. **Task list is flat — no inline subtask visibility.** Users can't see or interact with nesting from the main list. Dragging a task "into" another task has no visual context unless we show nesting inline.

4. **No drag affordance.** No drag handles, no cursor hints, no visual indicators for droppable zones.

5. **`parent_id` in `MoveTask` uses `Option<Option<i64>>` semantics awkwardly.** Currently `input.parent_id.or(existing.parent_id)` — meaning you can set a parent but never clear it back to `None` (top-level). This must be fixed for unnesting.

---

## Proposed Solution

### Position Strategy: Fractional Indexing

Replace `position: i32` with `sort_key: VARCHAR` using the [`fractional_index`](https://docs.rs/fractional_index/) crate (used by Figma, tldraw, and other collaborative apps).

**Why:**
- **1 row updated per move** — compute a string key between two adjacent keys
- **No sibling shifting** — no O(n) updates
- **Unlimited precision** — strings grow slightly with each bisection but never degrade
- **Simple SQL** — `ORDER BY sort_key` works directly in PostgreSQL
- **WASM-compatible** — pure Rust crate, runs in domain crate for client-side preview

**Migration path:**
1. Add `sort_key VARCHAR(64) NOT NULL DEFAULT ''` column
2. Backfill from existing `position` values (map to well-spaced fractional keys)
3. Update all `ORDER BY position` to `ORDER BY sort_key`
4. Drop `position` column after migration is verified

### Drag-and-Drop: Custom HTML5 DnD

Build a custom drag-and-drop system using the HTML5 Drag and Drop API via `web_sys::DragEvent`. No existing Leptos library supports both flat reordering and nesting.

**Drop zone detection (reorder vs. nest):**

```
┌─────────────────────────────┐
│  Top 25%    → insert ABOVE  │
│  Middle 50% → NEST inside   │
│  Bottom 25% → insert BELOW  │
└─────────────────────────────┘
```

When a task has no subtask_count (leaf task), the middle zone is split — top 50% = above, bottom 50% = below — since nesting into a leaf is less common and can be done via a modifier key (hold Shift) or by first hovering for 500ms.

### Inline Subtask Indicators

When a task has subtasks, show an expandable indicator in the task list (small triangle + count). Users can expand to see nested tasks inline, making the drag target for "nest inside" visually obvious.

### Optimistic UI

- Maintain a `local_task_order: RwSignal<Vec<TaskWithMeta>>` separate from the server `Resource`
- On drop: recompute `sort_key`, update local signal immediately, dispatch server action
- On server error: refetch resource and sync back to local signal
- While action is pending: skip resource → local sync to prevent flicker

---

## User Stories

### US-1: Reorder tasks in Inbox by dragging

**As a** user viewing my Inbox,
**I want to** drag a task up or down in the list to change its order,
**so that** I can prioritize tasks by importance.

**Acceptance criteria:**
- A drag handle icon appears on the left side of each task card on hover
- Dragging a task shows a semi-transparent ghost of the card
- A blue insertion line appears between tasks to indicate the drop position
- Dropping places the task at the indicated position
- The reorder persists after page refresh
- The reorder is reflected in the REST API (`GET /api/tasks?inbox=true` returns new order)

### US-2: Reorder tasks in a Project by dragging

**As a** user viewing a Project page,
**I want to** drag tasks to reorder them within the project,
**so that** I can organize my project tasks by priority or workflow order.

**Acceptance criteria:**
- Same drag-and-drop behavior as Inbox
- Ordering is scoped to the project (doesn't affect other projects or inbox)
- Works for top-level project tasks

### US-3: Nest a task under another task by dragging

**As a** user in Inbox or Project view,
**I want to** drag a task onto another task to make it a subtask,
**so that** I can organize related tasks hierarchically without opening the detail modal.

**Acceptance criteria:**
- Hovering over the middle 50% of a task card highlights it with a colored border/background indicating "nest inside"
- Dropping on the highlighted zone moves the task to become a child of the target task (appended at the end of the target's children)
- The source task disappears from the top-level list and the target task's subtask count updates
- The source task's `project_id` is updated to match the target's `project_id` (inherits parent's project)
- Nesting depth limit of 5 is enforced — if the target is already at depth 4, nesting is disallowed and a visual indicator shows it's not a valid target
- If the dragged task has its own subtasks, the entire subtree moves (existing behavior via `parent_id` change)

### US-4: Unnest a task by dragging it out

**As a** user viewing inline subtasks in Inbox or Project view,
**I want to** drag a subtask out to the top level of the list,
**so that** I can promote a subtask to a standalone task.

**Acceptance criteria:**
- When subtasks are expanded inline, they can be dragged
- Dragging a subtask to the insertion line between top-level tasks removes it from its parent
- The task's `parent_id` is set to `NULL` (top-level)
- The task retains its other properties (project, tags, dates, etc.)

### US-5: Reorder subtasks within a parent by dragging

**As a** user viewing inline subtasks,
**I want to** drag subtasks to reorder them within their parent,
**so that** I can control which subtasks are actionable (based on sequential_limit).

**Acceptance criteria:**
- Subtasks can be dragged up/down within their sibling group
- Reordering respects sequential_limit — the first N incomplete subtasks (by sort_key) remain actionable
- The reorder persists and affects the order shown in the task detail modal's SubtaskList

### US-6: No manual reordering in Today, All Tasks, Filters, or Review

**As a** user on the Today, All Tasks, Filter, or Review page,
**I expect** tasks to be ordered by their inherent criteria (start_at, filter sort, review_due),
**so that** the ordering reflects the page's purpose rather than manual arrangement.

**Acceptance criteria:**
- No drag handles appear on task cards in Today, All Tasks, Filter, or Review views
- Task cards are not draggable in these views
- The `draggable` prop is explicitly disabled

### US-7: Visual feedback during drag

**As a** user dragging a task,
**I want** clear visual feedback showing where the task will land,
**so that** I can confidently place it in the right position.

**Acceptance criteria:**
- **Drag ghost**: Semi-transparent copy of the task card follows the cursor
- **Source dimming**: The original card becomes semi-transparent (opacity 30%) while being dragged
- **Insert indicator**: A 2px blue horizontal line appears between tasks at the insertion point
- **Nest indicator**: Target task gets a blue left border and subtle background highlight when hovering the nest zone
- **Invalid target**: If nesting would exceed depth 5, show a red indicator / "not allowed" cursor
- **Smooth transitions**: Adjacent tasks animate smoothly (150ms) to make room for the insertion indicator

### US-8: Expand/collapse inline subtasks in task list

**As a** user viewing Inbox or Project tasks,
**I want to** expand a task to see its subtasks inline in the list,
**so that** I can see the hierarchy and drag subtasks for reordering or unnesting.

**Acceptance criteria:**
- Tasks with `subtask_count > 0` show a small expand/collapse chevron next to the checkbox
- Clicking the chevron loads and shows subtasks indented below the parent
- Subtasks are indented with visual nesting indicator (left border or padding)
- Expanded state is local (not persisted) — defaults to collapsed
- Subtasks show with reduced styling (slightly smaller, dimmed if not actionable)
- Maximum inline expansion depth: 2 levels (deeper subtasks accessible via detail modal)

---

## Technical Design

### New/Modified Crates & Files

**Domain (`crates/domain/`):**
- `task.rs`: Change `position: i32` → remove; add `sort_key: String` to `Task`, `CreateTask`, `MoveTask`
- Add `fractional_index` dependency

**DB (`crates/db/`):**
- New migration: `alter tasks add sort_key varchar(64), drop position`
- `models/task.rs`: Update `TaskRow`, `NewTask`, `TaskChangeset`
- `schema.rs`: Auto-updated by diesel

**Services (`crates/services/`):**
- `task_service.rs`:
  - All `ORDER BY position` → `ORDER BY sort_key`
  - New: `reorder_task(pool, user_id, task_id, sort_key, parent_id)` method
  - Fix `move_task()` to support clearing `parent_id` (unnesting)
  - Position calculation on create: use `FractionalIndex::new_after(last_sibling_key)`

**App (`crates/app/`):**
- New: `components/drag_drop/` module
  - `context.rs`: `DragDropContext` — provided via `provide_context`, holds drag state signals
  - `types.rs`: `DropZone { Above, Below, Nest }`, `DragPayload { task_id, source_parent_id }`
  - `hooks.rs`: `use_draggable_task()`, `use_drop_target()` — reusable hook-like functions
- Modified: `components/task_list/view.rs` — wrap each TaskCard with drop target logic
- Modified: `components/task_card/view.rs` — add drag handle, draggable attribute, drag event handlers
- Modified: `stores/task_store.rs` — add `local_task_order: RwSignal`, `reorder: Action`, optimistic sync
- New: `components/task_list/inline_subtasks.rs` — expandable inline subtask rendering

**Server (`crates/server/`):**
- `routes/tasks.rs`: Update `MoveTask` handler for new sort_key field

**UI (`crates/ui/`):**
- `icon.rs`: Add `DragHandle` icon variant (6-dot grip icon)

### Server Function

```rust
#[server]
pub async fn reorder_task(
    task_id: i64,
    sort_key: String,
    parent_id: Option<Option<i64>>, // None = no change, Some(None) = unnest, Some(Some(id)) = nest
) -> Result<(), ServerFnError> { ... }
```

### Migration

```sql
-- up.sql
ALTER TABLE tasks ADD COLUMN sort_key VARCHAR(64) NOT NULL DEFAULT '';

-- Backfill: convert integer positions to spaced fractional keys
-- (done in Rust migration seed, not SQL)

ALTER TABLE tasks DROP COLUMN position;

-- down.sql
ALTER TABLE tasks ADD COLUMN position INTEGER NOT NULL DEFAULT 0;
ALTER TABLE tasks DROP COLUMN sort_key;
```

### Depth Calculation

To enforce max nesting depth during drag, we need to know the depth of both the dragged task's subtree and the target. Two approaches:

**Option A (recommended):** Compute depth client-side from the task list data. Since we only expand 2 levels inline, we know the depth of visible tasks. For deeper nesting, the server validates and rejects.

**Option B:** Add a `depth` field to `TaskWithMeta` computed during `enrich()`. More accurate but adds a query.

---

## Implementation Phases

### Phase 1: Foundation (sort_key migration + basic reorder)
1. Add `fractional_index` to domain crate
2. Database migration: `position` → `sort_key`
3. Update services, models, and server functions
4. Add `reorder_task` server function
5. Fix `move_task()` to support `parent_id: Option<Option<i64>>`

### Phase 2: Drag-and-Drop UI (flat list reordering)
1. Build `drag_drop/` module (context, types, hooks)
2. Add drag handle icon to `TaskCard`
3. Implement drag events on task cards
4. Implement drop targets between cards (insertion indicators)
5. Add `DragHandle` icon to `north_ui`
6. Wire optimistic updates through `TaskStore`
7. Add `draggable` prop to `TaskList`/`TaskCard` (disabled for Today/Filter/Review)

### Phase 3: Nesting via Drag
1. Implement nest zone detection (middle 50% of card)
2. Add visual feedback for nest targets
3. Handle parent_id changes on drop
4. Validate depth limits
5. Update subtask_count reactively in the local signal

### Phase 4: Inline Subtask Expansion
1. Build `inline_subtasks.rs` component
2. Add expand/collapse chevron to `TaskCard`
3. Load subtasks on expand (or eagerly for small lists)
4. Support drag within expanded subtasks
5. Support drag to unnest (drag subtask to top-level insertion line)

---

## Open Questions

1. **Should expanded subtask state persist across page navigations?** Current proposal: no, always collapsed by default. Alternative: store in a `RwSignal<HashSet<i64>>` at the page level.

2. **What happens when dragging a task with subtasks into another task?** Current proposal: the entire subtree moves. The dragged task becomes a child of the target, and its children remain as grandchildren. Should we warn if this would exceed depth 5?

3. **Should we support keyboard-based reordering as an accessibility alternative?** (e.g., Alt+Up/Down to move a task). This would be a good complement to drag-and-drop.

4. **Fractional index key length**: The `fractional_index` crate's keys are typically 3-8 bytes for normal use. After extreme reordering (100+ operations in the same spot), keys can grow to 20+ bytes. Should we add a periodic rebalancing job, or is VARCHAR(64) sufficient?

---

## Risks & Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| HTML5 DnD doesn't work on mobile | High | Medium | Document as desktop-only; plan touch support in future phase |
| Fractional keys grow unbounded | Low | Low | VARCHAR(64) handles thousands of operations; add rebalance if needed |
| Drag between expanded subtrees gets confusing | Medium | Medium | Limit inline expansion to 2 levels; deeper via modal |
| Optimistic UI conflicts with server state | Low | Medium | Version-check on sync; server is source of truth on refetch |
| Performance with large task lists (100+) | Low | Medium | Virtual scrolling can be added later; DnD perf is DOM-bound not data-bound |
