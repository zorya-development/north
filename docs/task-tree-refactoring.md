# Task Tree Refactoring

Replaced O(n) linear scans with an indexed `TaskTree` and unified page controllers behind a single `TaskTreeView` abstraction.

## What Changed

### New: `TaskTree` (crates/stores/src/task_tree.rs)

Indexed tree built from `Vec<TaskModel>` with:

- **O(1) lookups**: `get(id)`, `children_of(parent_id)`, `parent_of(id)`, `sort_key(id)`
- **Domain methods**: `is_actionable(id)` (replaces old `ActionableController::is_actionable`), `count_matching(pred)`, `ancestors(id)`, `is_descendant(ancestor, task)`
- **Flattening with ancestor preservation**: `flatten(filter, extra_visible)` — if a nested task matches the filter, intermediate ancestors are shown even if they fail the filter. Root tasks must independently match.
- **Flat mode**: `flatten_flat(ids)` — preserves input order, no tree expansion (used by filter page)

Children are pre-sorted into `ChildGroup { active, someday, completed }` vectors. Display order: active → someday → completed.

Lives as `Memo<TaskTree>` on `TaskStore` — single shared instance, rebuilt when tasks signal changes.

### New: `TaskTreeView` (crates/app/src/controllers/task_tree_view.rs)

Reactive filtered view over the shared tree:

```rust
pub struct TaskTreeView {
    pub tree: Memo<TaskTree>,
    pub filter: Signal<Callback<TaskModel, bool>>,
    pub extra_visible: RwSignal<Vec<i64>>,
    pub is_loaded: Signal<bool>,
}
```

Pages create a `TaskTreeView` with a filter callback that encapsulates all visibility rules (root selection, completed toggle, actionable check). TTL consumes it for flattening and rendering.

### Migrated Pages

Each page controller was rewritten from ~60-100 lines of boilerplate (TaskStoreFilter + ActionableController + NodeFilterController + build_toolbar) to ~40-60 lines with a single filter callback + TaskTreeView.

| Page | Root gating | Filter logic |
|------|------------|-------------|
| **Inbox** | `project_id.is_none()` | completed toggle + actionable + keep_completed pinning + auto-detect disappeared tasks |
| **AllTasks** | all roots | completed toggle + actionable |
| **Today** | `start_at <= now` | completed toggle + actionable |
| **Someday** | `someday && !completed` | actionable (no completed toggle) |
| **Project** | `project_id == pid` | completed toggle + actionable + auto-detect + clear on navigation |
| **Review** | two views (pending/reviewed) | pending: needs_review + actionable; reviewed: recently_reviewed |

### Deleted

| File | Replacement |
|------|------------|
| `controllers/actionable.rs` | `TaskTree::is_actionable()` + inline `browser_storage.toggle_bool()` |
| `controllers/node_filter.rs` | Filter callback composed by each page controller |
| `controllers/toolbar.rs` | `ToolbarConfig` built inline by each page controller |
| `tree.rs: flatten_tree()` | `TaskTree::flatten()` |
| `tree.rs: flatten_flat()` | `TaskTree::flatten_flat()` |
| `tree.rs: compute_sort_key()` | `compute_sort_key_from_tree()` using `TaskTree::sort_key()` |
| `tree.rs: is_descendant_of()` | `TaskTree::is_descendant()` |

### TTL Changes

TraversableTaskList accepts `view: Option<TaskTreeView>` as the primary interface. When set, it uses `tree.flatten()` for rendering and `TaskTree` methods for O(1) sort key lookups, tag collection, and drag-drop cycle prevention.

**Kept for legacy consumers:**
- `root_task_ids` + `node_filter` + `flat` props — used by filter page (flat DSL results) and task detail modal (subtask display starting from non-root parent)

### Behavior Change: Ancestor Preservation

Previously, hiding a parent (e.g. via completed toggle) hid its entire subtree. Now, if a nested task matches the filter, its ancestor path to the root is preserved and shown. Root tasks must still independently match the filter — ancestor preservation only applies within a matched root's subtree.

Example: Root A (active) → Child B (completed, hidden) → Grandchild C (active, matches). With the new behavior, B is shown as a path node between A and C.

## Test Coverage

- 31 unit tests in `crates/stores/src/task_tree.rs` covering: build/indexing, lookups, ancestors, is_descendant, is_actionable (sequential_limit edge cases), count_matching, flatten (ancestor preservation, root gating, extra_visible, display ordering)
- 2 navigation tests in `crates/app/src/containers/traversable_task_list/tree.rs`
- E2E tests validate end-to-end behavior
