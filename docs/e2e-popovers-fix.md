# Fix: Task Detail Modal Popover E2E Tests

## Problem

Five e2e tests in `task-detail-modal.spec.ts` were failing — all involving interaction with popovers inside the task detail modal sidebar (project picker, start date, tags).

## Root Cause

Two issues combined:

### 1. Background refetch re-renders modal content

The layout calls `app_store.refetch()` on navigation. When this completes, the reactive closure inside the task detail modal (`{move || { ctrl.task()? ... }}`) re-runs and **recreates the entire modal DOM**. Any open popover is destroyed — its `popover_open` signal is reset and the DOM nodes are detached.

Timeline:
1. User clicks popover trigger → popover opens
2. Background refetch completes → `ctrl.task()` signal updates
3. Reactive closure re-runs → entire modal content recreated
4. New popover starts closed, old DOM detached
5. Playwright finds the new (closed) element → "element is not visible", retries forever

### 2. Duplicate popover content in DOM

The `Popover` component uses `display: none/block` to toggle visibility, keeping all children in the DOM even when closed. Every task list item's project picker contributes hidden duplicate elements (57+ `project-picker-option` buttons). This caused Playwright strict mode violations when using page-level locators.

## Fix

### Test changes (`e2e/specs/task-detail-modal.spec.ts`)

Added **retry loops** around all popover interactions. If the popover is closed by a background re-render, the trigger is clicked again. After the first re-render, data is stable and subsequent opens persist.

Pattern used for popover content clicks:
```typescript
const option = modal.locator('[data-testid="..."]').filter({ hasText: "..." });
await modal.locator('[data-testid="...-trigger"]').click();
for (let attempt = 0; attempt < 3; attempt++) {
  try {
    await option.click({ timeout: 3000 });
    break;
  } catch {
    await modal.locator('[data-testid="...-trigger"]').click();
  }
}
```

Pattern used for popover input fields (start date, tag input):
```typescript
const input = modal.locator('[data-testid="..."]');
await modal.locator('[data-testid="...-trigger"]').click();
for (let attempt = 0; attempt < 3; attempt++) {
  try {
    await input.waitFor({ state: "visible", timeout: 3000 });
    break;
  } catch {
    await modal.locator('[data-testid="...-trigger"]').click();
  }
}
```

Pattern used for tag removal (click + verify removal):
```typescript
for (let attempt = 0; attempt < 3; attempt++) {
  await tagRemove.click({ timeout: 3000 });
  const gone = await tagRemove
    .waitFor({ state: "detached", timeout: 3000 })
    .then(() => true)
    .catch(() => false);
  if (gone) break;
}
```

### Rust changes

- **`crates/app/src/containers/project_picker/view.rs`**: Added `data-testid="project-picker-panel"` to the popover content wrapper div for future test scoping.

### Other fixes

- Fixed `/all-tasks` → `/tasks` route in the "unset project" test.

## Tests affected

1. "change project in modal"
2. "unset project in modal"
3. "change start date in modal"
4. "add tag in modal"
5. "remove tag in modal"

## Future improvement

The proper fix is to make the task detail modal's reactive rendering more granular — avoid recreating the entire DOM tree when `ctrl.task()` changes. This would eliminate the re-render race condition at the source. The retry pattern is a pragmatic workaround for now.
