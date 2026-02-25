---
name: north-e2e
description: North-specific conventions for writing Playwright e2e tests — auth fixture, API helper, selectors, and known pitfalls. Use when writing or editing tests in e2e/.
trigger: When editing files in e2e/ — specs or fixtures.
---

# North E2E Conventions

Read `e2e/specs/inbox.spec.ts` and `e2e/specs/keyboard-nav.spec.ts` as reference implementations.

## Running Tests

The e2e stack uses `docker-compose.test.yml` with separate test containers.

- **Interactive (human):** `just playwright` — starts all test containers and opens Playwright UI mode on port 8080.
- **Headless (Claude Code):** `just playwright-exec` — runs tests inside already-running containers. The user must have started the stack first with `just playwright`.
- **Cleanup:** `just playwright-down` — tears down all test containers and volumes.

## Auth & Setup

Import `test` from `../fixtures/auth` (not `@playwright/test`) to get the `authenticatedPage` fixture pre-logged-in. Use `ApiHelper` to set up and tear down data:

```ts
import { test } from "../fixtures/auth";
import { ApiHelper } from "../fixtures/api";

let api: ApiHelper;

test.describe("...", () => {
  test.beforeEach(async ({ authenticatedPage }) => {
    api = new ApiHelper(authenticatedPage.context());
    await api.deleteAllTasks();
  });
  test.afterEach(async () => { await api.deleteAllTasks(); });
});
```

## sort_key — Never assign manually

`sort_key` uses `FractionalIndex` (a binary-encoded format). Raw strings like `"a"`, `"b"`, `"c"` are **invalid** — `from_string()` falls back to the same default for all, breaking ordering. **Always omit `sort_key`** and let the API assign it:

```ts
// CORRECT — insertion order is preserved automatically
await api.createTask({ title: "Task A" });
await api.createTask({ title: "Task B" });

// WRONG
await api.createTask({ title: "Task A", sort_key: "a" });
```

## Selectors

Always use `data-testid` attributes. Never select by CSS class, tag name, or other non-test-selector attributes — they are styling concerns, not test contracts.

- **Search first:** Before writing a test, grep for existing `data-testid` values in `crates/app/` to find selectors you can reuse.
- **Add when missing:** If a needed `data-testid` doesn't exist, add it to the relevant Rust component in `crates/app/` before writing the test.
- **Data attributes for state:** Use `data-*` attributes (e.g. `data-focused="true"`) to assert element state, not CSS classes.
