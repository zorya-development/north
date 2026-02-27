# CLAUDE.md

## Build & Development Commands

First-time setup (builds base image + dev image via compose):

```bash
docker compose build
```

Start dependencies, then enter the app container:

```bash
docker compose up -d db redis
docker compose run --rm -ti --service-ports app bash
```

Inside the container, use `just` commands:

```bash
just dev               # Run dev server (cargo-leptos watch)
just test              # Run all tests
just test crate_name   # Run tests for specific crate
just fmt               # Format code
just lint              # cargo clippy
just migrate           # Apply migrations (diesel)
just migration name    # Create new migration
just migrate-revert    # Revert last migration
just migrate-redo      # Revert + reapply last migration
just build             # Release build
just seed              # Seed admin account
just check             # fmt + lint + test
```

E2E tests (Playwright, run from host — not inside app container):

```bash
just playwright              # Launch Playwright UI mode (port 8080)
just playwright-exec         # Run tests in already-running containers (used by Claude Code)
just playwright-down         # Tear down test containers
```

For non-interactive / CI use: `docker compose exec app just <command>`

## Architecture Overview

GTD-inspired task management system. Single Rust binary serving SSR + WASM via Leptos 0.7 + Axum.

### Dependency Graph

```
dto → db → core → server-fns → repositories → stores → app
                                                        ↑
                                                       ui
server (Axum binary, calls core directly)
```

### Layer Rules (strict boundaries — never skip layers)

- **Page** (container/controller/view) → talks to **Stores** only
- **Store** (RwSignal, optimistic updates) → talks to **Repositories** only (never server-fns)
- **Repository** (thin facade) → talks to **Server Functions** only
- **Server Fn** (`#[server]` RPC boundary) → delegates to **Core**
- **Core** (business logic, Diesel queries) → reused by both Server Functions and REST API

### Data Flow

```
Read:  Page → Store → Repository → ServerFn ──RPC──→ Service → Diesel → PG
       View ← Memo(filter) ← RwSignal ← store.load()

Write: View → Callback → Store.update()
         ├─→ update_in_place()   (optimistic)
         └─→ Repository → ServerFn ──RPC──→ Service → Diesel → PG

REST:  HTTP → Axum Router → Auth Middleware → Route Handler → Core → Diesel → PG
```

## Key Decisions

These are non-obvious conventions. Violating them causes bugs or inconsistency.

- **Views never access stores.** Controller is the view-model. Views receive data (Memo/Signal) and callbacks; they only own pure UI state (toggles, input refs, local flags).
- **Pages own data loading.** Layout is purely structural (auth guard + context providers). Each page calls `refetch()` or creates its own `Resource`.
- **Container/controller/view pattern:** `container.rs` (entry, wires controller to view via inline `Callback` props), `controller.rs` (logic, store interaction), `view.rs` (pure rendering). Simpler components use two-file container/view.
- **Sub-components:** Extract inline `#[component]`s into a `components/` subdirectory (one file per component, `mod.rs` re-exports).
- **containers/ vs components/:** `containers/` = complex stateful domain components (pickers, task list, sidebar). `components/` = simpler/presentational (date picker, layout, modals).
- **Context:** Use `provide_context()` directly — no `.provide()` wrappers. Consume via `expect_context::<T>()` or `use_app_store()`.
- **Sequential tasks:** `compute_actionable()` in Rust, not SQL window functions.
- **Sort keys:** Fractional indexing (`sort_key` varchar), not integer `position`. See `dto/sort_key.rs`.
- **Diesel models:** `XxxRow` (Queryable), `NewXxx` (Insertable), `XxxChangeset` (AsChangeset).
- **Error handling:** `thiserror` for domain errors, `ServiceResult<T>`, no `anyhow`.
- **Time:** Always `chrono::Utc`, never naive datetimes.
- **Logging:** `tracing` crate, never `println!`.
- **Atoms:** Prefer `<Text variant=TextVariant::HeadingLg>` over raw Tailwind for typography. See `docs/UI_KIT.md`.

### Reactive Safety (avoiding disposed-signal panics)

Signals created inside a `<Show>` or conditional scope are disposed when that scope is torn down. Accessing them after disposal panics in WASM. Follow these rules:

1. **Never wrap `<For>` in a `{move || { ... }}` reactive closure.** The closure re-runs on signal changes, destroying and recreating the entire `<For>` and all child components. Instead, use stable `<Show>` components for loading/empty states and let `<For>` always be mounted (hide with `style:display`). The `<For>` handles its own keyed diffing.

2. **Blur before disposing focused elements.** When closing a modal/panel that may contain a focused input, call `html_el.blur()` on the active element *before* triggering the close signal. Otherwise the browser fires `blur` events into handlers with disposed signals. See `TaskDetailModal` container for the pattern.

3. **Use `try_get`/`try_set` in code that may run during disposal.** Any signal `.get()` or `.set()` that could execute during scope teardown (blur handlers, effects tracking external signals, fallback closures) must use `try_get()`/`try_set()` instead. Common hot spots:
   - `on:blur` handlers on inputs inside `<Show>` scopes
   - `prop:value` closures reading signals from a parent scope
   - Effect closures that track signals from both inside and outside the scope
   - `<Show when=...>` and `fallback=...` closures reading scoped signals

4. **Store-level signals are safe.** Signals on `AppStore`, `TaskStore`, `ModalStore`, etc. (created at app startup) are never disposed. Only signals created inside `<Show>`, `<For>` children, or component bodies inside conditional scopes are at risk.

## Common Workflows

### Add New DTO Type
1. Create type in `crates/dto/src/`, export in `lib.rs`
2. Create migration: `just migration name`, write `up.sql` + `down.sql`
3. Run `just migrate` (auto-updates `schema.rs`)
4. Add model structs in `crates/db/src/models/`
5. Add service methods in `crates/core/src/`

### Add New Page
1. Create `crates/app/src/pages/<name>/` with `container.rs`, `controller.rs`, `view.rs`, `mod.rs`
2. Controller loads its own data — layout does not pre-fetch
3. Container wires controller to view with inline `Callback` props
4. Export in `crates/app/src/pages/mod.rs`
5. Add server functions in `crates/server-fns/src/`
6. Register route in `crates/app/src/app.rs`

### Add New Container
1. Create `crates/app/src/containers/<name>/` with `container.rs`, `view.rs`, `mod.rs`
2. Optional `controller.rs` for complex logic
3. Export in `crates/app/src/containers/mod.rs`
4. Import UI primitives from `north_ui`

### Add New UI Primitive
1. Create component in `crates/ui/src/`, export in `lib.rs`
2. No dto dependencies — only `leptos` and rendering libs

### Add New REST Endpoint
1. Add handler in `crates/server/src/routes/`
2. Register in `crates/server/src/routes/mod.rs`

### Add New Migration
```bash
just migration add_due_reminders
# Edit up.sql and down.sql
just migrate       # Applies + updates schema.rs
just migrate-redo  # Test reversibility
```

## Infrastructure

- PostgreSQL 17, Redis 7 (reserved for future use)
- Docker Compose for dev — three layered images: base → dev → prod
- `just bump-base {major,minor,patch}` to bump base Docker image version
- `just bump-version {major,minor,patch}` to bump app version

## References

- @docs/ARCHITECTURE.md — Full project structure, crate details, data models, REST API, store/component details, CI/CD
- @docs/UI_KIT.md — UI component catalog and atom conventions
