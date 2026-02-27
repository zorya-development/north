# Regression Checklist

Human-driven pre-release checklist. Tick each box during a testing session before tagging a release.

---

## 1. Auth _(covered by `e2e/specs/login.spec.ts` + `e2e/specs/auth-session.spec.ts`)_

- [x] Navigate to `/login` — login form renders
- [x] Submit with wrong credentials — error message shown, no redirect
- [x] Submit with correct credentials — redirect to Inbox
- [x] Reload page — session persists (stays logged in)
- [x] Click Logout — redirect to `/login`, session cleared, back-button does not restore session

---

## 2. Sidebar _(covered by `e2e/specs/sidebar.spec.ts`)_

- [x] Sidebar is visible on first load
- [x] Press `Ctrl+B` — sidebar collapses; press again — expands
- [x] Click collapse icon — same toggle behavior
- [x] **Projects section:** Create project — appears in sidebar
- [ ] Rename project via sidebar — title updates in sidebar and page header
- [ ] Archive project via sidebar — disappears from sidebar, appears on Archive page
- [ ] **Saved Filters section:** Saved filters appear; clicking one navigates to Filter page with query pre-filled
- [x] Theme toggle switches between light and dark; preference persists on reload
- [x] Nav links (Inbox, Today, All Tasks, Review, Archive, Settings) all navigate to correct pages

---

## 3. Inbox _(covered by `e2e/specs/inbox.spec.ts`)_

- [x] Page loads with task list
- [x] Type in inline input at top, press `Enter` — task created, appears in list
- [x] Press `Ctrl+Enter` after a task — new task inserted below; `Shift+Enter` inserts above
- [x] Click checkbox on a task — task marked complete, moves to completed section (or disappears per toggle)
- [x] Click checkbox again — task uncompleted
- [x] Press `Delete` on selected task — confirmation prompt, then task deleted
- [x] Toggle "Hide non-actionable" — non-actionable tasks hide/show; state persists on reload
- [x] Sequential project: only first N tasks shown as actionable when limit is set

---

## 4. Task Detail Modal _(covered by `e2e/specs/task-detail-modal.spec.ts`)_

- [x] Click a task title — modal opens with correct task data
- [x] Edit title inline — change persists on close and reopen
- [x] Edit body (Markdown) — change persists; Markdown preview renders correctly
- [ ] Add a tag via tag picker — tag appears in task meta
- [ ] Remove a tag — tag disappears
- [ ] Assign a project via project picker — project shown in modal header and task meta
- [ ] Set due date — date appears in task meta; past dates show in danger color
- [ ] Set start date — date appears in task meta
- [ ] Set recurrence — recurrence summary shown; completing task creates next occurrence
- [ ] Clear recurrence — recurrence summary disappears
- [x] Create subtask via inline input — subtask appears under parent; parent shows progress indicator
- [ ] Complete subtask — parent progress updates
- [x] Navigate between tasks with arrow buttons (or keyboard `J`/`K`) — modal content updates without close/reopen
- [x] Press `Escape` — modal closes

---

## 5. Task Inline Autocomplete _(covered by `e2e/specs/task-autocomplete.spec.ts`)_

- [ ] In task title input, type `#` — tag suggestions appear; selecting one appends tag and removes `#tag` token from title
- [x] Type `@` — project suggestions appear; selecting one assigns project and removes `@project` token
- [ ] Tag/project tokens in body field are also extracted on save
- [ ] Autocomplete dropdown dismisses on `Escape`

---

## 6. Keyboard Navigation _(covered by `e2e/specs/keyboard-nav.spec.ts`)_

- [x] `Down`/`Up` arrows move cursor between tasks in list
- [x] `Right` arrow on a parent task expands / moves into first child
- [x] `Left` arrow on a child task moves to parent
- [x] `Enter` on selected task opens inline editor; `Enter` saves; `Escape` cancels
- [x] `Space` toggles completion on selected task
- [x] `E` opens Task Detail Modal for selected task
- [x] `Delete` / `Backspace` on selected task triggers delete flow
- [x] `Shift+Down` / `Shift+Up` reorders task within siblings
- [x] `Shift+Right` indents task (makes subtask of previous sibling); `Shift+Left` unindents
- [x] Keyboard shortcuts are suppressed when any modal is open

---

## 7. Drag & Drop

- [ ] Drag a task — ghost preview follows cursor
- [ ] Drop **above** a task — task moves to position above target
- [ ] Drop **below** a task — task moves to position below target
- [ ] Drop **nested** onto a task — task becomes child of target
- [ ] Cycle prevention: cannot drag a parent onto one of its own descendants (drop rejected)
- [ ] Drag a task onto a project in the sidebar — task reassigned to that project

---

## 8. Today Page _(covered by `e2e/specs/today.spec.ts`)_

- [x] Tasks with `start_at` ≤ now appear on Today page
- [x] Tasks with future `start_at` do not appear
- [x] Completing a task on Today page removes it from the list (or moves to completed section)
- [ ] "Hide non-actionable" toggle works independently of Inbox toggle

---

## 9. All Tasks Page _(covered by `e2e/specs/all-tasks.spec.ts`)_

- [x] Tasks from all projects are shown
- [ ] Filter by project using DSL (`project = "Name"`) — list narrows
- [x] Creating a task here without a project lands in Inbox (no project assigned)
- [ ] Task detail modal opens and works as in Inbox

---

## 10. Project Page _(covered by `e2e/specs/project.spec.ts`)_

- [x] Navigating to a project shows only that project's tasks
- [x] Creating a task on the project page assigns it to that project automatically
- [x] Project title shown in page header
- [x] Completing / deleting tasks works same as Inbox

---

## 11. Review Page _(covered by `e2e/specs/review.spec.ts`)_

- [x] Overdue-for-review tasks appear (based on `review_interval_days` from settings)
- [x] Press `R` on a selected task — task marked as reviewed, removed from list
- [x] Click "Mark All Reviewed" — all visible tasks marked reviewed, list empties
- [x] "Recently Reviewed" toggle shows tasks reviewed within the interval
- [x] Reviewed tasks no longer appear after toggling Recently Reviewed off

---

## 12. Archive Page _(covered by `e2e/specs/archive.spec.ts`)_

- [x] Archived projects appear in list
- [x] Click Unarchive — project moves back to sidebar active projects
- [x] Delete archived project — project and its tasks removed; confirm prompt shown first

---

## 13. Filter Page _(covered by `e2e/specs/filter.spec.ts`)_

- [x] Free-text DSL input accepts queries (e.g. `status = active`)
- [ ] Autocomplete suggestions appear as you type field names, operators, and values
- [ ] Selecting a suggestion inserts it into the query
- [x] Valid query executed — matching tasks listed below
- [x] Invalid DSL — syntax error message shown, no crash
- [x] Click "Save Filter" — modal prompts for name; saved filter appears in sidebar
- [x] Navigate away and back via saved filter — query pre-filled, results shown
- [ ] Delete saved filter from sidebar — disappears from sidebar and Saved Filters list
- [ ] `ORDER BY` clause sorts results correctly (e.g. `ORDER BY due_date ASC`)

---

## 14. Settings Page _(covered by `e2e/specs/settings.spec.ts`)_

- [x] Current `review_interval_days` value pre-filled
- [x] Current timezone pre-filled
- [x] Change review interval, save — toast "Saved" appears; value persists on reload
- [x] Change timezone, save — value persists on reload
- [ ] Invalid review interval (e.g. `0` or non-numeric) — validation error shown, no save
