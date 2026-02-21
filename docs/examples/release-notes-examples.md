# Release Notes Examples

## Example 1: Feature Commit

### Before (raw commit)

```
Add TraversableTaskList with keyboard-driven tree navigation

Introduce a new TraversableTaskList component that builds a depth-aware
flat list from the task store tree and provides full keyboard navigation,
inline title editing, and inline task creation with indent/outdent.

Key changes:
- Tree utilities (flatten_tree, sibling/parent/child navigation, sort key
  computation) with unit tests
- Cursor-based keyboard model: Up/Down for siblings, Left/Right for
  parent/child, Enter to edit, Ctrl+Enter/Shift+Enter to create,
  Tab/Shift+Tab to indent/outdent, E to open detail modal
- Keyed <For> rendering for efficient DOM updates
- Add sort_key field to CreateTask DTO for positional task creation
- Add flat_mode prop to TaskListItem to skip inline subtask rendering
```

### After (with User-facing section)

```
feat: add keyboard-driven task navigation

User-facing: Navigate your task tree without touching the mouse.
Use arrow keys to move between tasks (Up/Down for siblings, Left to
jump to parent, Right to drill into subtasks). Press Enter to edit,
Ctrl+Enter to create a new task, and Tab to indent/outdent.

Technical details:
- Add TraversableTaskList component with cursor-based navigation
- Implement tree utilities (flatten_tree, sibling/parent/child nav)
- Cursor model: Up/Down=siblings, Left=parent, Right=first child
- Enter=edit, Ctrl+Enter/Shift+Enter=create, Tab=indent/outdent
- Add sort_key field to CreateTask DTO for positional insertion
- Keyed <For> rendering for efficient updates
```

### Release Note Output

```markdown
### ✨ New Features

- **Keyboard navigation** - Navigate your task tree without touching
  the mouse. Use arrow keys to move between tasks (Up/Down for siblings,
  Left to jump to parent, Right to drill into subtasks). Press Enter
  to edit, Ctrl+Enter to create a new task, and Tab to indent/outdent.
```

---

## Example 2: Bug Fix Commit

### Before

```
Fix orphaned cursor after completing a task in TraversableTaskList

Advance cursor to next sibling, prev sibling, or next flat-list neighbor
before completing a task so it doesn't point at a disappeared/moved item.
Also recover orphaned cursors in move_up/move_down as a safety net for
checkbox clicks that bypass toggle_complete().
```

### After

```
fix: preserve cursor when completing tasks

User-facing: When you complete a task, the cursor now automatically
moves to the next available task instead of disappearing.

Technical details:
- Advance cursor before completing (next sibling → prev sibling →
  next flat neighbor)
- Add safety net in move_up/move_down for checkbox clicks
- Prevents cursor pointing at disappeared items
```

### Release Note Output

```markdown
### 🐛 Bug Fixes

- When you complete a task, the cursor now automatically moves to
  the next available task instead of disappearing.
```

---

## Example 3: Breaking Change

### Commit

```
feat: replace modal task creation with inline input

User-facing: The "+ Add task" button now opens an input field at the
top of your list instead of a modal. After creating your first task,
the input stays open below it so you can quickly add multiple tasks.

BREAKING CHANGE: Removed TaskCreateModal component and store.
Any custom code that references TaskCreateModalStore will need to be updated.

Technical details:
- Remove TaskCreateModal container and TaskCreateModalStore
- Add CreateTop inline mode to TraversableTaskList
- Add TtlHandle.start_create_top() for imperative triggering
- Fix blur handling with mode snapshot to prevent premature close
- Update all page controllers to use TtlHandle instead of modal store
```

### Release Note Output

```markdown
### ✨ New Features

- ⚠️ **BREAKING:** The "+ Add task" button now opens an input field
  at the top of your list instead of a modal. After creating your
  first task, the input stays open below it so you can quickly add
  multiple tasks.

### 📦 Upgrade Guide

This release contains breaking changes:

- **Removed TaskCreateModal** - Any custom code referencing
  `TaskCreateModalStore` should be updated to use `TtlHandle`
  methods instead. See migration guide in docs.
```

---

## Example 4: Performance Improvement

### Commit

```
perf: batch load task metadata to eliminate N+1 queries

User-facing: Task lists now load instantly, even with hundreds of
tasks. Previously, each task triggered separate database queries for
project/tag data.

Technical details:
- Add TaskService::enrich() to batch-load metadata
- Single query with LEFT JOINs for projects, tags, subtask counts
- Reduces 300 queries → 1 query for 100-task list
- ~500ms → ~50ms load time improvement
```

### Release Note Output

```markdown
### ⚡ Performance

- Task lists now load instantly, even with hundreds of tasks.
  Load times improved from ~500ms to ~50ms by eliminating redundant
  database queries.
```

---

## Example 5: Internal Refactor (No User-Facing Impact)

### Commit

```
refactor: extract TaskCheckbox into container/controller/view

Split TaskCheckbox into three files following our architecture pattern:
- container.rs: props + callback wiring
- controller.rs: completion logic + store interaction
- view.rs: pure rendering

No functional changes, just organizational cleanup.
```

### Release Note Output

This commit appears ONLY in the collapsed "Technical Details" section:

```markdown
<details>
<summary>🔧 Technical Details (for contributors)</summary>

### Refactor

- Extract TaskCheckbox into container/controller/view
  Split into three files following architecture pattern.
  No functional changes.

</details>
```

**Not shown in main release notes** because there's no user-facing impact.
