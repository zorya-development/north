# Someday Feature Implementation Plan

## Context

GTD "Someday/Maybe" list — tasks you want to keep but aren't committed to doing now. Adding `someday` as a boolean column on the tasks table (default false). When true, the task is deferred: non-actionable, visually muted, sorted into its own section, and accessible via a dedicated page. No custom server functions — toggle via existing `update_task` with `UpdateTask { someday: Some(true/false) }`.

## Implementation (bottom-up through dependency graph)

### 1. Migration — `someday` column

Create `migrations/<timestamp>_add_someday/`:
- **up.sql**: `ALTER TABLE tasks ADD COLUMN someday BOOLEAN NOT NULL DEFAULT FALSE;`
- **down.sql**: `ALTER TABLE tasks DROP COLUMN someday;`
- Run `just migrate` to auto-update `schema.rs`

### 2. DB Models — `crates/db/src/models/task.rs`

Add `someday` field to:
- `TaskRow`: `pub someday: bool`
- `NewTask`: `pub someday: bool`
- `TaskChangeset`: `pub someday: Option<bool>`
- `From<TaskRow> for Task`: map `someday: row.someday`

### 3. DTO — `crates/dto/src/task.rs`

- `Task`: add `pub someday: bool` (with `#[serde(default)]`)
- `UpdateTask`: add `pub someday: Option<bool>`
- `TaskFilter`: add `pub someday: Option<bool>` for REST API filtering

### 4. Core — `crates/core/src/task_service.rs`

- **`list()`**: add someday filter: `someday == Some(true)` → `tasks::someday.eq(true)`, `Some(false)` → `tasks::someday.eq(false)`
- **`update_raw()`**: map `someday` from resolved_input to changeset
- **`create()` + `spawn_next_recurring()`**: add `someday: false` to `NewTask` constructions
- **Review filter**: in `review_due == Some(true)` block, exclude someday tasks: `query = query.filter(tasks::someday.eq(false))`

### 5. Field Registry — `crates/core/src/filter/field_registry.rs`

Add `someday` to the exhaustive `Task` destructure in `_assert_exhaustive()`.

### 6. Repository — `crates/repositories/src/`

- **`models/task_model.rs`**: add `pub someday: bool` to `TaskModel`, update `From<Task>`, update test helper `make_dto_task()`
- No new repository methods — uses existing `TaskRepository::update(id, UpdateTask { someday: Some(val), ..Default::default() })`

### 7. Store — `crates/stores/src/task_store.rs`

- **`TaskStoreFilter`**: add `pub is_someday: Option<bool>`
- **`filtered()`**: add filter clause for `is_someday`
- **`toggle_someday(id)`**: new method — optimistic update via `update_in_place` + async `TaskRepository::update()` with `UpdateTask { someday: Some(!was_someday) }`

### 8. Actionability — `crates/app/src/libs/actionable.rs`

Add at top of `is_actionable()`:
```rust
if task.someday {
    return false;
}
```

### 9. Tree Flattening — `crates/app/src/containers/traversable_task_list/tree.rs`

- Add `is_someday: bool` to `FlatNode`
- **`flatten_tree()`**: three-way partition (active/someday/completed) instead of two-way. Order: active first, then someday, then completed.
- **`flatten_subtree()`**: same three-way partition for children
- **`flatten_flat()`**: set `is_someday` on FlatNode construction
- Update all test `FlatNode` literals and `make_task()` helpers with `someday`

### 10. Drag-Drop — `crates/app/src/containers/traversable_task_list/view.rs`

In `handle_drop()`, add `&& !n.is_someday` to the three sibling filter chains — same pattern as `!n.is_completed`. Prevents normal tasks from interleaving with someday tasks.

### 11. Keyboard Handler — `crates/app/src/containers/traversable_task_list/controller.rs`

Add `"s" | "S"` case in `handle_keydown_normal()` after the `"r"` block:
```rust
"s" | "S" => {
    ev.prevent_default();
    if let Some(task_id) = self.cursor_task_id.get_untracked() {
        self.app_store.tasks.toggle_someday(task_id);
    }
}
```

### 12. Someday Prefix — `crates/app/src/containers/task_list_item/components/`

New file `someday_prefix.rs`: renders `@Someday` as a clickable link to `/someday` with `text-text-secondary` color. Pattern follows `project_prefix.rs`.

Export from `components/mod.rs`.

### 13. Task List Item — `crates/app/src/containers/task_list_item/view.rs`

- Add `is_someday` memo (like `is_completed`)
- Show `<SomedayPrefix />` before `<ProjectPrefix />` when `is_someday` is true
- Muted text: color = `TextColor::Secondary` for someday tasks (between Primary and Tertiary)

### 14. Icon — `crates/ui/src/icon.rs`

Add `Someday` variant with hourglass SVG path.

### 15. Someday Page — `crates/app/src/pages/someday/`

New page (container/controller/view/mod.rs) following `all_tasks` pattern:
- Controller filters: `is_someday: Some(true)`, `is_completed: Some(false)`, `parent_id: IsNull`
- View: heading "Someday", TraversableTaskList with `draggable: true`, `show_inline_project: true`, `allow_create: false`
- Export in `pages/mod.rs`

### 16. Route — `crates/app/src/app.rs`

Add `/someday` route after `/tasks` route.

### 17. Sidebar — `crates/app/src/containers/sidebar/view.rs`

Add NavItem after "All Tasks": `<NavItem href="/someday" label="Someday" icon=IconKind::Someday collapsed=collapsed/>`

### 18. Keybindings Help — `crates/app/src/components/keybindings_modal.rs`

Add `("S", "Toggle someday")` to the keybindings list.

### 19. Review Page — `crates/app/src/pages/review/`

- **controller.rs**: add `.filter(|t| !t.someday)` to both `review_task_ids` and `reviewed_task_ids` Memos
- **view.rs**: add `show_inline_project: true` to `review_config` ItemConfig so both @project and @someday prefixes show

## Files Summary

**New files (7):**
- `migrations/<ts>_add_someday/up.sql`, `down.sql`
- `crates/app/src/containers/task_list_item/components/someday_prefix.rs`
- `crates/app/src/pages/someday/{mod,container,controller,view}.rs`

**Modified files (~18):**
- `crates/db/src/models/task.rs`
- `crates/dto/src/task.rs`
- `crates/core/src/task_service.rs`
- `crates/core/src/filter/field_registry.rs`
- `crates/repositories/src/models/task_model.rs`
- `crates/stores/src/task_store.rs`
- `crates/app/src/libs/actionable.rs`
- `crates/app/src/containers/traversable_task_list/tree.rs`
- `crates/app/src/containers/traversable_task_list/view.rs`
- `crates/app/src/containers/traversable_task_list/controller.rs`
- `crates/app/src/containers/task_list_item/components/mod.rs`
- `crates/app/src/containers/task_list_item/view.rs`
- `crates/ui/src/icon.rs`
- `crates/app/src/pages/mod.rs`
- `crates/app/src/app.rs`
- `crates/app/src/containers/sidebar/view.rs`
- `crates/app/src/components/keybindings_modal.rs`
- `crates/app/src/pages/review/controller.rs`
- `crates/app/src/pages/review/view.rs`

## Verification

1. `just migrate` — migration applies cleanly
2. `just fmt && just lint` — no warnings
3. `just test` — all existing tests pass + new tree tests for three-way partition
4. Manual: press S on a task → task moves to someday section with muted text and @Someday prefix
5. Manual: /someday page shows only someday tasks
6. Manual: sidebar shows Someday nav item
7. Manual: review page excludes someday tasks, shows @project + @someday prefixes when applicable
8. Manual: drag-drop keeps someday tasks separate from normal tasks
