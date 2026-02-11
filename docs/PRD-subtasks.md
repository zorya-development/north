# PRD: Subtasks & Task Detail Modal

## Context

North is a GTD-inspired task management app. The data layer already supports subtasks (parent_id, sequential_limit, cascade delete), but the UI has zero subtask support. Users currently see tasks as a flat list with no way to break work into smaller pieces or view full task details.

This PRD introduces two connected features: **subtask management** and a **task detail modal**. Together they let users decompose tasks, track progress on sub-items, and edit all task properties from a single view.

---

## Scope

**In scope:**
- Task detail modal for viewing/editing all task properties
- Subtask creation, editing, completion, deletion within the modal
- Recursive nesting (subtasks can have their own subtasks, up to 5 levels deep)
- Outline-style keyboard navigation (Tab to indent, Shift+Tab to outdent)
- Subtask count indicator on task cards in lists
- Subtask progress display (completed/total)

**Out of scope (follow-up features):**
- Drag-and-drop to convert tasks into subtasks
- Kanban board view
- New data fields (assignee, reminders, location, deadline)
- Subtask support in filter DSL

---

## User Stories

### US-1: Open task detail modal

**As a** user viewing a task list,
**I want to** click on a task to open a detail modal,
**so that** I can see and edit all task properties in one place.

**Acceptance criteria:**
- Clicking a task title (or a dedicated "open" action) opens a centered modal overlay
- Modal displays: task title (editable), body/description (editable, markdown), and a right sidebar with metadata
- Sidebar shows: Project (with picker), Column/Status (with picker), Tags (with picker), Start date (with date picker), Due date (with date picker)
- User can edit title and body inline (click to edit)
- Changes save on blur or explicit save action
- Modal can be closed with X button, Escape key, or clicking the backdrop
- Up/down arrow navigation between tasks (like Todoist's chevrons) to browse through the current list without closing the modal

### US-2: Create subtasks

**As a** user viewing a task in the detail modal,
**I want to** add subtasks to break work into smaller pieces,
**so that** I can track progress on individual steps.

**Acceptance criteria:**
- Detail modal shows a "Sub-tasks" section below the task title/body
- An "+ Add sub-task" button appears at the bottom of the subtask list
- Clicking it reveals an inline form with title input field
- Pressing Enter creates the subtask and keeps the form open for the next one (rapid entry)
- Pressing Escape closes the inline form
- New subtask inherits the parent's project (if any)
- Subtask appears immediately in the list
- Nesting is allowed up to 5 levels deep; the "+ Add sub-task" button is hidden at the depth limit

### US-3: Outline-style subtask creation

**As a** user rapidly entering subtasks,
**I want to** use Tab and Shift+Tab to control nesting level,
**so that** I can build a task hierarchy quickly without leaving the keyboard.

**Acceptance criteria:**
- While the inline subtask creation form is focused:
  - **Tab** indents the current input one level deeper (makes it a child of the sibling above)
  - **Shift+Tab** outdents the current input one level (moves it up to the parent's level)
  - Visual indentation updates immediately to show the target nesting level
- Tab does nothing if there is no sibling above to nest under
- Tab does nothing if already at maximum depth (5 levels)
- Shift+Tab does nothing if already at the top level of the current subtask list
- After pressing Enter to create, the next input stays at the same nesting level

### US-4: View and manage subtasks

**As a** user with a task that has subtasks,
**I want to** see, complete, and manage subtasks,
**so that** I can track progress through the sub-items.

**Acceptance criteria:**
- Subtasks are listed in position order, with nested subtasks indented under their parents
- Each subtask shows: checkbox (complete/uncomplete), title, and a subtle child count if it has its own subtasks
- Clicking a subtask title opens that subtask in the same modal (replaces parent content), with a breadcrumb chain back through ancestors to the root task
- Completed subtasks show with strikethrough and dimmed styling
- "Hide completed" / "Show completed" toggle in the subtask section header
- Subtask section header shows progress: "Sub-tasks N/M" (completed/total, counting only direct children)
- Subtask tree is shown expanded by default; collapsible via a toggle arrow on subtasks that have children

### US-5: Edit a subtask

**As a** user managing subtasks,
**I want to** edit subtask properties,
**so that** I can update titles, set dates, and manage subtask details.

**Acceptance criteria:**
- Clicking a subtask title navigates into it within the modal (breadcrumb chain to ancestors)
- Subtask detail view shows same editable fields as a regular task: title, body, project, tags, start date, due date, column
- If the subtask has its own children, they appear in a "Sub-tasks" section (recursive)
- If at max depth (5 levels), "+ Add sub-task" is hidden
- User can complete/delete the subtask from this view

### US-6: Delete a subtask

**As a** user managing subtasks,
**I want to** delete subtasks I no longer need,
**so that** I can keep my task list clean.

**Acceptance criteria:**
- Each subtask row has a delete action (kebab menu or direct icon)
- Deletion asks for no confirmation (consistent with current task delete behavior)
- Deleting a subtask also deletes all its children (cascade)
- Subtask disappears from the list immediately

### US-7: See subtask count on task cards

**As a** user browsing a task list,
**I want to** see how many subtasks a task has and their completion status,
**so that** I can gauge progress at a glance without opening the task.

**Acceptance criteria:**
- Task cards in any list (inbox, today, all tasks, project, filter results) show a subtask indicator when the task has direct children
- Indicator format: icon + "completed/total" (e.g., "1/3") counting only direct children
- Indicator appears in the task metadata row alongside existing meta (dates, project, tags)
- Clicking the indicator opens the task detail modal

### US-8: Complete a parent task with incomplete subtasks

**As a** user completing a parent task,
**I want to** understand what happens to incomplete subtasks,
**so that** I don't lose track of unfinished work.

**Acceptance criteria:**
- Completing a parent task completes all its incomplete descendants (recursive cascade)
- The completion is reflected immediately in the UI
- Uncompleting a parent task does NOT automatically uncomplete descendants

### US-9: Sequential subtask ordering

**As a** user with ordered subtasks,
**I want to** control how many subtasks are "actionable" at once,
**so that** I can focus on a limited number of next actions (GTD sequential processing).

**Acceptance criteria:**
- Parent task detail modal exposes a "Sequential limit" setting (already exists in data model as `sequential_limit`)
- Default value: shows all subtasks as actionable (limit = 0 or equal to total count)
- When set to N, only the first N incomplete direct children are marked as actionable
- Non-actionable subtasks appear visually dimmed in the subtask list
- Sequential limit applies per-level (each parent controls its own children's actionability independently)

---

## UI Specifications

### Task Detail Modal Layout

```
+-------------------------------------------------------+
| [Project / Column breadcrumb]       [<] [>] [...] [X] |
|-------------------------------------------------------|
|                                    |                   |
| (o) Task Title (editable)          | Project    [pick] |
|   = Description (editable, md)     | Column     [pick] |
|                                    | Tags       [pick] |
| v Sub-tasks  (i) 2/5   Hide done  | Start date [pick] |
|   [x] Completed subtask           | Due date   [pick] |
|   v ( ) Active subtask 1          | Seq. limit [edit] |
|       ( ) Nested child A          |                   |
|       ( ) Nested child B          |                   |
|   ( ) Active subtask 2            |                   |
|   ( ) Dimmed subtask (blocked)    |                   |
|   + Add sub-task                   |                   |
|                                    |                   |
+-------------------------------------------------------+
```

### Outline-style Creation (Tab/Shift+Tab)

```
Sub-tasks section during rapid entry:

  ( ) Buy groceries
  ( ) Plan meals for the week
      ( ) Monday dinner             ← Tab indented under "Plan meals"
      ( ) Tuesday dinner            ← Enter keeps same level
      [New subtask title___]        ← cursor here, indented
  ( ) Clean kitchen                 ← Shift+Tab would outdent back
```

### Subtask Breadcrumb (when viewing a nested subtask)

```
+-------------------------------------------------------+
| [Project / Column breadcrumb]       [<] [>] [...] [X] |
|-------------------------------------------------------|
| ( ) Root Task  |  3 >  ( ) Parent Sub  |  2 >         |
|                                    |                   |
| (o) Nested Subtask (editable)      | Project    [pick] |
|   = Description (editable, md)     | Column     [pick] |
|                                    | ...               |
| v Sub-tasks 0/2                    |                   |
|   ( ) Deeper child 1              |                   |
|   ( ) Deeper child 2              |                   |
|   + Add sub-task                   |                   |
+-------------------------------------------------------+
```

### Task Card Subtask Indicator

```
( ) Task title
    Jan 15  |  Project Name  |  #tag1  |  [subtask-icon] 2/5
```

---

## Behavioral Rules

1. **Recursive nesting up to 5 levels**: The DB structure is recursive (parent_id). Nesting is soft-limited to 5 levels in the UI. The "+ Add sub-task" button is hidden when at depth 5.
2. **Project inheritance**: New subtasks inherit the parent's `project_id` and `column_id`. Changing a parent's project moves all descendants too.
3. **Cascade delete**: Deleting a task deletes all its descendants (handled by DB foreign key ON DELETE CASCADE).
4. **Cascade complete**: Completing a parent completes all incomplete descendants (recursive). Uncompleting does NOT cascade.
5. **Position ordering**: Subtasks are ordered by `position` field within their parent. Each nesting level has independent position ordering.
6. **Subtasks in list views**: Only root-level tasks (parent_id IS NULL) appear in inbox, today, all-tasks, and project list views. Subtasks only appear within their parent's detail modal.
7. **Filter DSL**: Subtasks remain excluded from filter DSL results (existing behavior, out of scope to change).
8. **Depth counting**: Depth is determined by traversing parent_id chain. Root task = depth 0, its child = depth 1, etc. Max allowed = depth 5.
9. **Sequential limit per level**: Each task's `sequential_limit` controls actionability of its direct children only, not grandchildren.
10. **Outline keyboard shortcuts**: Tab/Shift+Tab only work within the inline subtask creation form context, not in normal text editing.

---

## Implementation Phases

### Phase 1: Task Detail Modal (foundation)
- Modal component with task display and inline editing (title, body)
- Right sidebar with metadata pickers (project, column, tags, dates)
- Open/close from task card click
- Up/down navigation between tasks in current list

### Phase 2: Subtask Display & Management
- Subtask tree display in modal (recursive, indented)
- Completion toggles on subtasks
- Subtask progress indicator (N/M) in modal header
- Hide/show completed subtasks toggle
- Navigate into subtask detail (breadcrumb chain)
- Collapse/expand subtask groups with children

### Phase 3: Subtask Creation
- Inline subtask creation form ("+ Add sub-task")
- Rapid entry mode (Enter to create and stay open)
- Outline-style Tab/Shift+Tab nesting control
- Depth limit enforcement (max 5 levels)

### Phase 4: List Integration
- Subtask count badge on task cards
- Cascade completion behavior (recursive)
- Sequential limit UI control in modal

---

## Verification

- Create a task, open its detail modal, verify all fields are editable
- Add 3 subtasks, complete 1, verify progress shows "1/3"
- Toggle "Hide completed", verify completed subtask is hidden
- Click into a subtask, verify breadcrumb navigation back through ancestors
- Add a sub-subtask (level 2), verify it appears indented under its parent
- Navigate to depth 5, verify "+ Add sub-task" is hidden
- Test Tab key during creation: verify indentation increases and subtask is created under sibling
- Test Shift+Tab key during creation: verify outdentation works
- Test Enter during creation: verify rapid entry stays at same level
- Complete parent task, verify all descendants are marked complete recursively
- Check task cards in inbox/today/project views show subtask count indicator
- Set sequential_limit to 1, verify only first incomplete subtask is actionable at each level
