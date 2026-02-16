# CLAUDE.md

This file provides guidance to Claude Code when working with code in this repository.

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
just migration-diff n  # Create migration via schema diff
just migrate-revert    # Revert last migration
just migrate-redo      # Revert + reapply last migration
just build             # Release build
just seed              # Seed admin account
just check             # fmt + lint + test
```

For non-interactive / CI use: `docker compose exec app just <command>`

## Architecture Overview

GTD-inspired task management system. Single Rust binary serving SSR + WASM via Leptos 0.7 + Axum.

### Dependency Graph (crates)

```
dto           (pure data types, no IO — compiled for both server and WASM)
  ↑
  db          (Diesel schema, models, pool type)
  ↑
core          (business logic layer — all services, filter DSL, Diesel queries)
  ↑
server-fns    (Leptos #[server] functions — RPC boundary, delegates to core)
  ↑
repositories  (thin async facade, hides transport)
  ↑
stores        (reactive state, optimistic updates)
  ↑
  app         (pages, containers, components, atoms, views)
  ↑
  ui          (generic UI components — leptos only, no dto deps)

server        (Axum binary, REST API routes — calls core directly)
```

### Client-Server Architecture

```
┌──────────┐    ┌────────────┐    ┌──────────────┐    ┌──────────────┐  ║  ┌──────────────┐    ┌──────────┐    ┌──────────┐
│          │    │            │    │              │    │  Server Fn   │  ║  │  Server Fn   │    │          │    │          │
│   Page   │───▶│   Store    │───▶│  Repository  │───▶│  (client)    │══RPC═▶│  (server)    │───▶│  Service │───▶│ DB/Diesel│──▶ PG
│          │    │            │    │              │    │              │  ║  │              │    │          │    │          │
└──────────┘    └────────────┘    └──────────────┘    └──────────────┘  ║  └──────────────┘    └──────────┘    └──────────┘
                                                                       ║
 container/      reactive state,   thin facade,        #[server] macro ║   auth extraction,    business        models,
 controller/     optimistic        hides transport     generates HTTP   ║   context wiring      logic,          schema,
 view            updates,          details             POST stub        ║                       queries,        mappings
                 client-side                                            ║                       enrichment
                 filtering                             ◄── WASM ───────║────── Server ──►

        ◄──────────────────── DTO types (pure data, no IO) shared across all layers ──────────────────────►
```

Layer rules:
- Pages talk to Stores, never deeper
- Stores talk to Repositories, never server functions directly
- Repositories talk to Server Functions, nothing else
- Core is reused by both Server Functions and REST API routes
- DTO types are the shared language across every layer and both runtimes

Why each layer exists:

| Layer | Why |
|---|---|
| **Page** | UI composition — wires store to view via callbacks |
| **Store** | Reactive state (RwSignal) so UI updates automatically. Optimistic updates for instant feedback. Client-side filtered Memos let multiple pages share one dataset. |
| **Repository** | Decouples stores from transport. Stores don't know #[server] exists. Swappable for testing. |
| **Server Fn** | Leptos RPC boundary — #[server] macro generates a client stub (serializes args, HTTP POST) and a server handler (deserializes, executes). Neither side sees HTTP directly. |
| **Core** | Single home for business logic. All services (Task, Project, Tag, User, Filter, Stats). Reused by server fns AND REST API routes. No duplication. |
| **DB Layer** | Type-safe Diesel schema, model structs, enum mappings. Core builds queries against these types. |
| **DTO** | Pure data types compiled to both WASM and server. The contract everyone agrees on. |

### Data Flow

Read (e.g. loading inbox):
```
Page → Controller → Store.refetch() → Repository → ServerFn ──RPC──→ Service → Diesel → PG
                                                                          │
View ← Memo(filter) ← RwSignal ← store.load() ←──────────────────────────┘
```

Write (e.g. updating a task):
```
View → Callback → Store.update_task()
                    ├─→ update_in_place()  (optimistic UI update)
                    └─→ Repository → ServerFn ──RPC──→ Service → Diesel → PG
                          └─→ refetch_async()  (full reload to sync)
```

REST API (used by external clients):
```
HTTP Request → Axum Router → Auth Middleware → Route Handler → Core → Diesel → PG
```

## Project Structure

```
north/
├── Cargo.toml                  # Workspace config (resolver 2)
├── rust-toolchain.toml         # Stable + wasm32-unknown-unknown
├── diesel.toml                 # Diesel CLI config (schema.rs path)
├── justfile                    # Dev commands (see above)
├── docker-compose.yml          # app (depends on base), PostgreSQL 17, Redis 7
├── docker/
│   ├── base/Dockerfile         # Base image: Rust 1.93, cargo-leptos, diesel_cli, wasm target
│   ├── base/VERSION            # Semver for base image (source of truth)
│   ├── dev/Dockerfile          # Dev image: extends base, adds just + tailwindcss v4
│   └── prod/Dockerfile         # Prod image: multi-stage (base → debian:bookworm-slim)
├── cliff.toml                  # git-cliff changelog config
├── migrations/                 # Diesel reversible migrations (up.sql + down.sql)
├── style/main.css              # TailwindCSS v4, dark theme, Inter + JetBrains Mono
├── public/                     # Static assets served at /public
├── uploads/                    # User-uploaded files (volume mount)
├── docs/                       # PRD.md, DESIGN.md
├── .github/workflows/
│   ├── test.yml                # CI: fmt, clippy, test (conditionally builds base image)
│   └── release.yml             # Release: prod Docker image + GitHub release on master
│
└── crates/
    ├── dto/                    # Pure data types (no IO, formerly "domain")
    │   └── src/
    │       ├── lib.rs
    │       ├── project.rs      # Project, ProjectStatus, ProjectViewType, CreateProject, UpdateProject, ProjectFilter
    │       ├── tag.rs          # Tag, CreateTag, UpdateTag, TagInfo
    │       ├── task.rs         # Task, CreateTask, UpdateTask, TaskFilter
    │       ├── dsl_suggestion.rs # DslSuggestion — autocomplete suggestion DTO
    │       ├── saved_filter.rs # SavedFilter, CreateSavedFilter, UpdateSavedFilter
    │       ├── serde_helpers.rs # double_option module for three-state Option<Option<T>> serialization
    │       ├── sort_key.rs     # sort_key_between(), sort_key_after() — fractional indexing for item ordering
    │       └── user.rs         # User, UserRole, UserSettings, CreateUser, UpdateUser, UpdateSettings, LoginRequest, AuthResponse
    │
    ├── db/                     # Diesel infrastructure (schema, models, pool type)
    │   └── src/
    │       ├── lib.rs          # DbPool type alias, DbError, DbResult
    │       ├── schema.rs       # Auto-generated by diesel print-schema
    │       ├── sql_types.rs    # PG enum mappings (UserRoleMapping, ProjectViewTypeMapping, ProjectStatusMapping)
    │       └── models/
    │           ├── mod.rs
    │           ├── user.rs     # UserRow, NewUser
    │           ├── project.rs  # ProjectRow, NewProject, ProjectChangeset
    │           ├── task.rs     # TaskRow, NewTask, TaskChangeset
    │           ├── tag.rs      # TagRow, NewTag
    │           ├── task_tag.rs # TaskTagRow, NewTaskTag
    │           ├── image.rs    # ImageRow, NewImage
    │           └── saved_filter.rs # SavedFilterRow, NewSavedFilter, SavedFilterChangeset
    │
    ├── core/                   # Business logic layer (north-core)
    │   └── src/
    │       ├── lib.rs          # ServiceError, ServiceResult, re-exports (DbPool, UserRow, all services)
    │       ├── task_service.rs # CRUD, enrich(), actionable computation, filtering, batch operations
    │       ├── project_service.rs # CRUD, find_by_title, archive/unarchive
    │       ├── tag_service.rs  # CRUD, sync_task_tags (full replace), add_task_tags (additive)
    │       ├── user_service.rs # Auth lookup, settings, admin creation
    │       ├── stats_service.rs # Aggregated statistics
    │       └── filter/         # Filter DSL subsystem
    │           ├── mod.rs      # Re-exports: parse_filter, FilterService, TaskFieldRegistry, eval_expr, AST types
    │           ├── dsl.rs      # FilterQuery, FilterExpr, Condition, FilterField, FilterOp, FilterValue, OrderBy, SortDirection
    │           ├── parser.rs   # parse_filter(), FilterParseError — recursive descent parser
    │           ├── context.rs  # DslCompletionContext, detect_completion_context() — autocomplete context detection
    │           ├── field_registry.rs # TaskFieldRegistry — compile-time safe field mapping with exhaustive Task destructure
    │           ├── autocomplete.rs # get_dsl_suggestions() — server-side suggestion generation (queries DB for tags/projects)
    │           ├── service.rs  # FilterService: SavedFilter CRUD with query validation
    │           ├── translator.rs # eval_expr() — AST → HashSet<i64> two-pass filter evaluation
    │           └── text_parser.rs # parse_tokens(), ParsedText — extracts #tags and @project from text
    │
    ├── stores/                 # Reactive client state (north-stores)
    │   └── src/
    │       ├── lib.rs          # Re-exports AppStore, TaskStore, etc.
    │       ├── app_store.rs    # AppStore { tasks, projects, tags, saved_filters, task_detail_modal, filter_dsl }
    │       ├── task_store.rs   # TaskStore: RwSignal<Vec<Task>>, optimistic updates, IdFilter, TaskStoreFilter
    │       ├── project_store.rs # ProjectStore: reactive project state
    │       ├── tag_store.rs    # TagStore: cached reactive tag state
    │       ├── saved_filter_store.rs # SavedFilterStore: CRUD + reactive state for saved filters
    │       ├── filter_dsl_store.rs # FilterDslStore: DSL query state, validation, suggestions, execution results
    │       ├── task_detail_modal_store.rs # TaskDetailModalStore: modal state, navigation, subtask handling
    │       └── hooks.rs        # use_app_store(), use_task_detail_modal_store()
    │
    ├── repositories/           # Thin async facade over server functions (north-repositories)
    │   └── src/
    │       ├── lib.rs          # Re-exports all repositories
    │       ├── task_repo.rs    # TaskRepository: list, get, create, update, delete, set_tags, complete, uncomplete, review_all
    │       ├── project_repo.rs # ProjectRepository: list, get, create, update, delete
    │       ├── filter_repo.rs  # FilterRepository: list, get, create, update, delete, execute, validate_query, get_completions
    │       ├── tag_repo.rs     # TagRepository: list
    │       └── settings_repo.rs # SettingsRepository: get, update
    │
    ├── server-fns/             # Leptos #[server] functions — RPC boundary (north-server-fns)
    │   └── src/
    │       ├── lib.rs          # Module declarations
    │       ├── auth.rs         # check_auth(), get_auth_user_id() — JWT extraction
    │       ├── tasks.rs        # list_tasks, get_task, create_task, update_task, delete_task, complete_task, uncomplete_task, set_task_tags, review_all_tasks
    │       ├── projects.rs     # list_projects, get_project, create_project, update_project, delete_project
    │       ├── filters.rs      # list_saved_filters, get_saved_filter, create_saved_filter, update_saved_filter, delete_saved_filter, execute_filter, validate_filter_query, get_dsl_completions
    │       ├── tags.rs         # list_tags
    │       └── settings.rs     # get_user_settings, update_settings
    │
    ├── ui/                     # Generic UI component library (north-ui)
    │   └── src/
    │       ├── lib.rs              # Re-exports all components
    │       ├── icon.rs             # Icon, IconKind (SVG icon enum)
    │       ├── dropdown.rs         # DropdownMenu, DropdownItem
    │       ├── popover.rs          # Popover (trigger + overlay + positioned panel)
    │       ├── checkbox.rs         # Checkbox (checked + on_toggle callback)
    │       ├── markdown.rs         # MarkdownView + render_markdown()
    │       ├── modal.rs            # Modal (backdrop + centered panel)
    │       ├── autocomplete.rs     # AutocompleteDropdown, SuggestionItem
    │       └── spinner.rs          # Spinner (loading animation)
    │
    ├── app/                    # Leptos library (SSR + WASM hydration)
    │   └── src/
    │       ├── lib.rs          # hydrate() entry point, recursion_limit = 256
    │       ├── app.rs          # Shell, App component, router setup
    │       ├── atoms/                  # UI Kit atoms (semantic, multi-dimension prop components)
    │       │   ├── mod.rs              # Re-exports all atoms
    │       │   └── text.rs             # Text (TextVariant, TextColor, TextTag — MD3-based typography)
    │       ├── pages/
    │       │   ├── login.rs    # LoginPage + login() server function
    │       │   ├── inbox/      # InboxPage (container/controller/view)
    │       │   ├── today/      # TodayPage (container/controller/view)
    │       │   ├── all_tasks/  # AllTasksPage (container/controller/view)
    │       │   ├── project/    # ProjectPage (container/controller/view, :id param)
    │       │   ├── archive/    # ArchivePage (container/controller/view)
    │       │   ├── review/     # ReviewPage (container/controller/view)
    │       │   ├── settings/   # SettingsPage (container/controller/view)
    │       │   ├── filter/     # FilterPage (container/controller/view, saved filters + DSL search)
    │       │   └── filter_help.rs # FilterHelpPage (DSL syntax reference, single file)
    │       ├── containers/             # Complex stateful domain components
    │       │   ├── autocomplete/       # Container/view pattern (tag/project autocomplete)
    │       │   ├── inline_task_input/  # Container/view pattern (borderless inline input for rapid subtask creation)
    │       │   ├── project_picker/     # Container/view pattern (supports icon_only prop)
    │       │   ├── sidebar/            # Container/view pattern (sidebar navigation)
    │       │   ├── tag_picker/         # Container/view pattern (supports icon_only prop)
    │       │   ├── task_create_modal/  # Container/view pattern (modal for task creation)
    │       │   ├── task_detail_modal/  # Container/view pattern (modal for task details)
    │       │   ├── task_list/          # Container/view pattern (provides ExtraVisibleIds context)
    │       │   └── task_list_item/     # Container/controller/view (single task row + inline subtask list)
    │       └── components/
    │           ├── date_picker/        # Container/view pattern (supports icon_only prop)
    │           ├── filter_autocomplete/ # DSL autocomplete for filter page
    │           ├── task_meta.rs        # Pure view (date, project, tags display)
    │           ├── drag_drop.rs        # Drag and drop utilities
    │           ├── theme_toggle.rs     # Dark/light theme toggle
    │           └── layout.rs           # AppLayout (auth guard, context providers, sidebar + main shell)
    │
    └── server/                 # Axum binary
        └── src/
            ├── main.rs         # Tokio main, Diesel pool, Leptos SSR router
            ├── error.rs        # AppError (NotFound, Unauthorized, Forbidden, BadRequest, Internal, Service)
            ├── seed.rs         # seed_admin() via UserService
            ├── auth/
            │   ├── mod.rs      # AuthUser { id, role } struct
            │   ├── jwt.rs      # create_token(), validate_token()
            │   └── middleware.rs  # JWT from cookie or Authorization header → AuthUser
            └── routes/
                ├── mod.rs      # public_api_router(), protected_api_router()
                ├── auth.rs     # POST /api/auth/login, /api/auth/logout
                ├── tasks.rs    # CRUD + review endpoints → north_core::TaskService
                ├── projects.rs # CRUD → north_core::ProjectService
                └── stats.rs    # GET /api/stats → north_core::StatsService
```

### Crate Details

- **`dto`** — Pure data types with serde + chrono, no IO. Compiled for both server and WASM. Key types: `Task` (includes enrichment fields: project_title, tags, subtask_count, completed_subtask_count, actionable), `TaskFilter` (complex query object), `ProjectFilter`, `ProjectStatus` (Active, Archived), `UserSettings` (review_interval_days, default_sequential_limit), `SavedFilter`, `DslSuggestion` (autocomplete suggestion DTO with label, value, color, start position). Utility modules: `serde_helpers` (three-state Option serialization), `sort_key` (fractional indexing for item ordering).
- **`db`** — Diesel infrastructure: `schema.rs` (auto-generated by `diesel print-schema`), model structs (`XxxRow` for reading, `NewXxx` for inserting, `XxxChangeset` for updating), PG enum mappings via `diesel-derive-enum` (`UserRoleMapping`, `ProjectViewTypeMapping`, `ProjectStatusMapping`), `DbPool` type alias for `diesel_async::deadpool::Pool<AsyncPgConnection>`.
- **`core`** — Full-featured business logic layer (`north-core`). Contains all services: `TaskService`, `ProjectService`, `TagService`, `UserService`, `StatsService`, `FilterService`. The `filter/` submodule consolidates the entire filter DSL subsystem: AST types (`dsl.rs`), recursive descent parser (`parser.rs`), autocomplete context detection (`context.rs`), server-side suggestion generation (`autocomplete.rs`), AST evaluation (`translator.rs`), and `TaskFieldRegistry` (`field_registry.rs`) with compile-time exhaustive `Task` destructure for field safety. Each service is a struct with static async methods that use Diesel's query builder directly. Key patterns: `TaskService::enrich()` for batch metadata loading (projects, tags, subtask counts), `compute_actionable()` for sequential task logic in Rust, `into_boxed()` for dynamic filtering, `execute_dsl_filter()` for filter DSL evaluation via `filter::eval_expr`. Re-exports `DbPool` and `UserRow`.
- **`stores`** — Reactive client state (`north-stores`). `AppStore` wraps `TaskStore` + `ProjectStore` + `TagStore` + `SavedFilterStore` + `TaskDetailModalStore` + `FilterDslStore`, provided globally via context. `TaskStore` holds tasks in `RwSignal<Vec<Task>>` — pages create filtered `Memo`s over the shared data. Supports optimistic updates (immediate UI, async sync). `FilterDslStore` manages DSL query text, async server-side validation, autocomplete suggestions, and execution results — the filter page controller delegates all DSL state to this store. Individual stores (`TagStore`, `SavedFilterStore`) cache domain data for pickers and navigation.
- **`repositories`** — Thin async facade (`north-repositories`). Decouples stores from server function details. No business logic — pure pass-through. Makes transport swappable for testing. Includes `TaskRepository`, `ProjectRepository`, `FilterRepository`, `TagRepository`, `SettingsRepository`.
- **`server-fns`** — Leptos `#[server]` RPC boundary (`north-server-fns`). Each function extracts `DbPool` from context and `user_id` from JWT, then delegates to core. The `#[server]` macro generates client stubs (HTTP POST) and server handlers automatically. Covers tasks, projects, filters, tags, and settings.
- **`ui`** — Generic UI component library (`north-ui`). No dto dependencies — only `leptos`, `pulldown-cmark`, `ammonia`. Components: `Icon`/`IconKind`, `DropdownMenu`/`DropdownItem`, `Popover`, `Modal`, `Checkbox`, `MarkdownView`/`render_markdown()`, `AutocompleteDropdown`/`SuggestionItem`, `Spinner`. Used by `app` crate for reusable UI primitives.
- **`app`** — Leptos library crate. Features: `hydrate` (WASM client), `ssr` (server-side, pulls in north-core/north-server-fns/argon2/jsonwebtoken). Pages follow container/controller/view pattern and interact with stores for data. Complex stateful domain components live in `containers/` (pickers, sidebar, autocomplete, task list, task list item, inline task input, create modal, detail modal). Simpler/presentational components live in `components/`. **`atoms/`** contains UI Kit atoms — semantic multi-dimension prop components (Text, Button, Badge, etc.) based on Material Design 3 type scale. Atoms use `enum.classes() -> &'static str` pattern for variant/color/size mapping. Prefer atoms over raw Tailwind for text, buttons, badges.
- **`server`** — Axum binary. Depends on `north-app` with `ssr` feature. Auth middleware injects `AuthUser { id, role }` into request extensions. Route handlers delegate to `north-core` for all services (tasks, projects, stats).

### REST API Routes

```
POST   /api/auth/login        (public)
POST   /api/auth/logout        (public)
GET    /api/tasks              (protected, supports TaskFilter query params)
POST   /api/tasks              (protected)
GET    /api/tasks/:id          (protected)
PATCH  /api/tasks/:id          (protected)
DELETE /api/tasks/:id          (protected)
PATCH  /api/tasks/:id/review   (protected)
GET    /api/projects           (protected, supports ProjectFilter query params)
POST   /api/projects           (protected)
GET    /api/projects/:id       (protected)
PATCH  /api/projects/:id       (protected)
GET    /api/stats              (protected)
```

## Data Models

```
users (email, password_hash, name, role ENUM, settings JSONB, created_at, updated_at)
├── projects (title, description, color, view_type ENUM, status ENUM, position, created_at, updated_at)
│   └── tasks (title, body, sort_key, sequential_limit, start_at, due_date, completed_at, reviewed_at, ...)
│       ├── tasks (subtasks via parent_id self-reference)
│       └── task_tags → tags (join table)
├── tags (name, color, UNIQUE per user)
├── saved_filters (title, query, position, created_at, updated_at)
└── images (path, filename, content_type, size_bytes)
```

DB enums: `user_role` (admin, user), `project_view_type` (list, kanban), `project_status` (active, archived).
Triggers: `update_updated_at()` on users, projects, tasks.

## Key Patterns

- **Auth:** JWT (7-day expiry) in httpOnly cookie. REST API also accepts `Authorization: Bearer` header. Server functions extract auth via `leptos_axum::extract()`. Password hashing with Argon2.
- **Sequential tasks:** `tasks.sequential_limit` controls how many subtasks are actionable. Computed in Rust via `compute_actionable()`, not SQL window functions.
- **Project status:** Projects use a `status` enum (active/archived) instead of a boolean `archived` field.
- **Sort keys:** Tasks use fractional indexing (`sort_key` varchar) for ordering instead of integer `position`. Utilities in `dto/sort_key.rs`.
- **Data access:** Diesel ORM with `diesel-async` for async PostgreSQL. Service layer uses Diesel query builder directly (no repository abstraction at the DB level). Batch metadata loading via `enrich()` to avoid N+1 queries.
- **AppLayout:** Purely structural — auth guard (redirects to `/login`), provides `AppStore` and `TaskDetailModalStore` contexts, renders sidebar + main shell. No data fetching — each page is responsible for loading its own data.
- **Context providers:** Use `provide_context()` directly in containers/controllers — no wrapper methods like `.provide()`. Views and child components consume via `expect_context::<T>()` or typed helpers like `use_app_store()`.
- **Page data ownership:** Each page owns its data loading. Pages call `refetch()` or create their own `Resource` on mount. The layout does not pre-fetch data for pages.
- **Container/controller/view pattern:** Pages with state management use a three-file pattern: `container.rs` (component entry, wires controller to view via inline `Callback` props), `controller.rs` (business logic, data loading, store interaction), `view.rs` (pure rendering). Simpler components use two-file container/view. Pure presentational components stay as single files. Callbacks are inlined directly into view props — no intermediate variables. Picker components (date, project, tag) support `icon_only` prop for compact action bar rendering in task cards.
- **Containers vs components:** `containers/` holds complex stateful domain components that wire together stores, repositories, and rich interactions (pickers, sidebar, autocomplete, task list, task list item, inline task input, create modal, detail modal). `components/` holds simpler or more presentational components (date picker, layout, filter autocomplete, drag_drop, task_meta).
- **Atoms:** `atoms/` contains UI Kit atom components — semantic, multi-dimension prop components based on Material Design 3 type scale. Each atom enum prop has `fn classes(self) -> &'static str` mapping Tailwind classes. Variant never includes color — color is always a separate prop. Each variant declares `default_tag()` for HTML element, overridable via `tag` prop. Prefer `<Text variant=TextVariant::HeadingLg>` over raw `<h1 class="text-2xl font-semibold ...">`. See `docs/UI_KIT.md` for the full component catalog.
- **Three-layer client architecture:** `server-fns` (RPC boundary) → `repositories` (thin facade) → `stores` (reactive state + business logic). Stores call repositories, never server-fns directly. Pages/controllers call stores, never repositories directly.
- **TaskStore:** Reactive store (`stores/task_store.rs`) that owns task state and mutations (complete, delete, update, set/clear start_at, refetch). `AppStore` wraps `TaskStore` for global context. Inbox uses `AppStore`; other pages create local stores with their own `Resource`. Two creation methods: `create_task()` (fire-and-forget, updates parent's subtask_count for modal use) and `create_task_async()` (async, skips parent update to avoid re-rendering the parent task item — used by inline input).
- **TagStore / SavedFilterStore:** Individual reactive stores for tags and saved filters, cached globally via `AppStore`. Used by pickers and navigation.
- **TaskDetailModalStore:** Manages task detail modal state, navigation between tasks, and subtask handling. Provided via `AppStore`.
- **FilterDslStore:** Manages filter DSL query lifecycle: text input, server-side validation, autocompletion suggestions, query execution, and results. Provided via `AppStore`.
- **Token parsing:** `parse_tokens()` in core crate extracts `#tags` and `@project` references from task title/body text. Core resolves these to DB records.
- **Filter DSL:** JQL-like query language with full subsystem in `core/filter/`. Recursive descent parser (`parse_filter()`), AST types (`FilterQuery`, `FilterExpr`, `Condition`), and two-pass evaluation (`eval_expr`) — parse → AST → `HashSet<i64>` of matching task IDs (AND=intersection, OR=union, NOT=difference). Supports fields (title, body, project, tags, status, due_date, start_at, created, updated), operators (`=`, `!=`, `=~` glob, `>`, `<`, `>=`, `<=`, `is null`, `in [...]`), logical operators (`AND`, `OR`, `NOT`, parentheses), and `ORDER BY`. `TaskFieldRegistry` provides compile-time safe field mapping via exhaustive `Task` struct destructure (no `..` rest pattern — adding a field to `Task` forces review of filter field coverage).
- **DSL autocomplete:** Server-side suggestion generation via `core/filter/autocomplete.rs` — queries DB for tags/projects, suggests field names, status values, and keywords. `FilterAutocompleteTextarea` component reads/writes through `FilterDslStore` via context. Uses `on_submit` callback for Enter-to-search.
- **FilterDslStore:** Centralized reactive store for DSL query state (query text, parse error, suggestions, execution results, loading state). Delegates async validation and autocompletion to server via `FilterRepository`. The filter page controller is thin — manages only saved filter CRUD and UI state (title editing, save modal), delegating all DSL state to this store.
- **Filter page search bar:** Filter results execute when the user explicitly clicks Search or presses Enter, not on every keystroke. Save modal (`Modal` component) prompts for title when creating new filters.
- **Completed tasks toggle:** Task list pages (inbox, today, all_tasks, project) pass an optional `completed_resource` to `TaskList`. The `CompletedSection` component renders a toggle button with count and dimmed completed tasks below the active list.
- **ExtraVisibleIds:** Context type (`ExtraVisibleIds(RwSignal<Vec<i64>>)`) provided by `TaskList` container and `TaskDetailModal`. Keeps inline-created subtasks visible even when they exceed the sequential_limit. Scoped to the container's lifecycle — clears automatically on page navigation or modal close.
- **Inline subtask input:** `InlineTaskInput` container renders a borderless input inside the subtask list for rapid task creation. Enter creates a subtask (inheriting parent's project_id), clears input, keeps it open and focused. Escape or blur hides the input. Input value is stored in the parent `InlineSubtaskList` so it persists across open/close. Created task IDs are pushed to `ExtraVisibleIds` to stay visible.

## Code Conventions

- **Rust edition 2021**, stable toolchain
- **Type hints** on all public functions
- **Imports** — grouped: std → external crates → local crates, sorted alphabetically
- **Error handling** — use `thiserror` for domain errors, `anyhow` avoided. Services return `ServiceResult<T>`, routes convert via `From<ServiceError> for AppError`
- **Test files** — `_test.rs` suffix or `tests/` module, standard Rust conventions
- **Logging** — `tracing` crate, never `println!` for errors/info
- **Time** — always `chrono::Utc`, never naive datetimes
- **No inline comments for obvious code**
- **Line length** — 100 characters preferred
- **Diesel models** — `XxxRow` (Queryable, Selectable), `NewXxx` (Insertable), `XxxChangeset` (AsChangeset). `From<XxxRow> for DomainType` conversions.

## Common Workflows

### Add New DTO Type
1. Create type file in `crates/dto/src/`
2. Export in `crates/dto/src/lib.rs`
3. Create migration: `just migration name`
4. Write `up.sql` and `down.sql`
5. Run `just migrate` (auto-updates `schema.rs`)
6. Add model structs in `crates/db/src/models/`
7. Add service methods in `crates/core/src/`

### Add New Page
1. Create page directory in `crates/app/src/pages/<name>/` with `container.rs`, `controller.rs`, `view.rs`, `mod.rs`
2. Controller loads its own data (calls `refetch()` or creates `Resource`) — layout does not pre-fetch
3. Container wires controller to view with inline `Callback` props
4. Export in `crates/app/src/pages/mod.rs`
5. Add server functions in `crates/server-fns/src/`
6. Register route in `crates/app/src/app.rs`
7. Add nav item in `crates/app/src/containers/sidebar/view.rs`

### Add New Atom (UI Kit component)
1. Create file in `crates/app/src/atoms/<name>.rs`
2. Define enums with `fn classes(self) -> &'static str` for each dimension (variant, color, size)
3. Add `fn default_tag(self) -> Tag` if the atom renders HTML elements
4. Export in `crates/app/src/atoms/mod.rs`
5. Use `use crate::atoms::{ComponentName, ...}` in views

### Add New UI Primitive (generic, no domain deps)
1. Create component file in `crates/ui/src/`
2. Export in `crates/ui/src/lib.rs`
3. No dto dependencies — only `leptos` and rendering libs

### Add New Container (complex stateful domain component)
1. Create directory `crates/app/src/containers/<name>/`
2. `container.rs` — component entry, wires data + inline `Callback` props to view (no intermediate variables)
3. `controller.rs` — (optional, for complex logic) business logic, data loading, store interaction
4. `view.rs` — pure rendering, receives data + handlers as props
5. `mod.rs` — `pub use container::ComponentName;`
6. Export in `crates/app/src/containers/mod.rs`
7. Import generic UI primitives from `north_ui` (Icon, Dropdown, Popover, etc.)

### Add New Component (simpler/presentational)
1. Create directory `crates/app/src/components/<name>/`
2. `container.rs` — component entry, wires data + inline `Callback` props to view
3. `view.rs` — pure rendering, receives data + handlers as props
4. `mod.rs` — `pub use container::ComponentName;`
5. Export in `crates/app/src/components/mod.rs`

Pure presentational components (no internal state management) stay as single files.

### Add New REST Endpoint
1. Add handler in `crates/server/src/routes/`
2. Register in router in `crates/server/src/routes/mod.rs`
3. Protect with auth middleware if needed

### Add New Migration
```bash
just migration add_due_reminders  # Creates migrations/YYYY-MM-DD-NNNNNN_add_due_reminders/
# Edit up.sql and down.sql
just migrate                       # Applies migration, updates schema.rs
just migrate-redo                  # Test reversibility
```

## Infrastructure

- PostgreSQL 17, Redis 7 (Redis reserved for future use)
- Docker Compose for development (all commands via `docker compose exec app`)
- Diesel ORM with diesel-async for async PostgreSQL access
- Images stored on filesystem (`./uploads/` volume mount)

### Docker Images

Three Dockerfiles, layered:

| Image | Path | Purpose |
|---|---|---|
| **base** | `docker/base/Dockerfile` | Rust toolchain, cargo-leptos, diesel_cli, wasm32 target, clippy, rustfmt |
| **dev** | `docker/dev/Dockerfile` | Extends base, adds `just` and `tailwindcss` CLI |
| **prod** | `docker/prod/Dockerfile` | Runtime-only: `debian:bookworm-slim` with pre-built binary (built in CI container job) |

- `docker/base/VERSION` is the single source of truth for base image version
- `docker-compose.yml` builds base locally as `north-base:<version>`, dev image extends it
- CI and release pull base from `ghcr.io/zorya-development/north/base:<version>`
- Use `just bump-base {major,minor,patch}` to bump base version (updates VERSION, dev Dockerfile, docker-compose.yml)
- Use `just bump-version {major,minor,patch}` to bump app version in root `Cargo.toml`

### CI/CD

Two GitHub Actions workflows:

**test.yml** (push to master + all PRs):
1. `resolve` — reads `docker/base/VERSION`, detects changes in `docker/base/**`
2. `build-base` — conditional: builds and pushes base image to ghcr.io only if base files changed
3. `check` — runs in base container: `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test`

**release.yml** (push to master only):
1. `resolve` — reads app version from `Cargo.toml`, checks if git tag `v<version>` exists
2. `build` — if tag doesn't exist: builds release binary in base container (reuses `north-cargo` cache from test workflow), uploads artifacts
3. `package` — packages binary into `debian:bookworm-slim` image, pushes to ghcr.io (`:v<version>` + `:latest`), generates changelog via git-cliff, creates GitHub release with tag

### Release Process

To release a new version:
1. Bump version: `just bump-version {major,minor,patch}`
2. Commit and push to master (via PR or direct push)
3. The release workflow automatically:
   - Builds the release binary in the base container (reuses cargo cache from test workflow)
   - Packages into a slim runtime Docker image and pushes to `ghcr.io/zorya-development/north:v<version>` and `:latest`
   - Generates changelog from commits since last tag using git-cliff
   - Creates a GitHub release with the changelog

To bump the base Docker image (when toolchain or dependencies change):
1. Modify `docker/base/Dockerfile` as needed
2. Run `just bump-base {major,minor,patch}`
3. Commit and push — the test workflow will build and push the new base image
