# CLAUDE.md

This file provides guidance to Claude Code when working with code in this repository.

## Build & Development Commands

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
just migrate           # Apply migrations (sqlx)
just migration name    # Create new migration
just build             # Release build
just sqlx-prepare      # Regenerate offline query cache
just seed              # Seed admin account
just check             # fmt + lint + test
```

For non-interactive / CI use: `docker compose exec app just <command>`

## Architecture Overview

GTD-inspired task management system. Single Rust binary serving SSR + WASM via Leptos 0.7 + Axum.

### Data Flow

```
Browser → Axum Router
  ├── /api/*  → REST API handlers (server crate, auth via JWT middleware)
  └── /*      → Leptos SSR → server functions (app crate) → DB
                           → hydrated WASM on client
```

## Project Structure

```
north/
├── Cargo.toml                  # Workspace config (resolver 2)
├── rust-toolchain.toml         # Stable + wasm32-unknown-unknown
├── justfile                    # Dev commands (see above)
├── docker-compose.yml          # app, PostgreSQL 17, Redis 7
├── docker/dev/Dockerfile       # Rust 1.93, cargo-leptos, sqlx-cli, tailwindcss v4
├── migrations/                 # sqlx migrations (single initial migration)
├── style/main.css              # TailwindCSS v4, dark theme, Inter + JetBrains Mono
├── public/                     # Static assets served at /public
├── uploads/                    # User-uploaded files (volume mount)
├── docs/                       # PRD.md, DESIGN.md
├── .github/workflows/ci.yml   # GitHub Actions CI
│
└── crates/
    ├── domain/                 # Pure data types (no IO)
    │   └── src/
    │       ├── lib.rs
    │       ├── column.rs       # Column, CreateColumn, UpdateColumn
    │       ├── project.rs      # Project, ProjectWithColumns, CreateProject, UpdateProject, ProjectViewType
    │       ├── tag.rs          # Tag, CreateTag, UpdateTag
    │       ├── task.rs         # Task, TaskWithMeta, CreateTask, UpdateTask, MoveTask, TaskFilter
    │       └── user.rs         # User, UserRole, UserSettings, DefaultColumn, auth DTOs
    │
    ├── app/                    # Leptos library (SSR + WASM hydration)
    │   └── src/
    │       ├── lib.rs          # hydrate() entry point
    │       ├── app.rs          # Shell, App component, router setup
    │       ├── pages/
    │       │   ├── login.rs    # LoginPage + login() server function
    │       │   └── inbox.rs    # InboxPage (main dashboard)
    │       ├── components/
    │       │   ├── layout.rs   # AppLayout (sidebar + main, auth guard)
    │       │   ├── nav.rs      # Sidebar navigation
    │       │   ├── task_card.rs
    │       │   └── task_form.rs
    │       └── server_fns/
    │           └── auth.rs     # check_auth(), get_auth_user_id()
    │
    └── server/                 # Axum binary
        └── src/
            ├── main.rs         # Tokio main, DB pool, migrations, Leptos SSR router
            ├── error.rs        # AppError (NotFound, Unauthorized, Forbidden, BadRequest, Internal, Sqlx)
            ├── seed.rs         # seed_admin() — admin@north.local / admin
            ├── auth/
            │   ├── jwt.rs      # create_token(), validate_token()
            │   └── middleware.rs  # JWT from cookie or Authorization header → AuthUser
            └── routes/
                ├── mod.rs      # public_api_router(), protected_api_router()
                ├── auth.rs     # POST /api/auth/login, /api/auth/logout
                ├── tasks.rs    # CRUD + move + review endpoints
                ├── projects.rs # CRUD + column management
                └── stats.rs    # GET /api/stats
```

### Crate Details

- **`domain`** — Pure data types with serde + chrono, no IO. Compiled for both server and WASM. Key types: `TaskFilter` (complex query object), `TaskWithMeta` (task + project_title, column_name, tags, subtask_count, actionable), `UserSettings` (review_interval_days, default_sequential_limit, default_columns).
- **`app`** — Leptos library crate. Features: `hydrate` (WASM client), `ssr` (server-side, pulls in sqlx/argon2/jsonwebtoken). Server functions use `#[server]` macro with `#[cfg(feature = "ssr")]` for DB access via `expect_context::<PgPool>()`.
- **`server`** — Axum binary. Depends on `north-app` with `ssr` feature. Auth middleware injects `AuthUser { id, role }` into request extensions.

### REST API Routes

```
POST   /api/auth/login       (public)
POST   /api/auth/logout       (public)
GET    /api/tasks             (protected)
POST   /api/tasks             (protected)
GET    /api/tasks/:id         (protected)
PATCH  /api/tasks/:id         (protected)
DELETE /api/tasks/:id         (protected)
PATCH  /api/tasks/:id/move    (protected)
PATCH  /api/tasks/:id/review  (protected)
GET    /api/projects          (protected)
POST   /api/projects          (protected)
GET    /api/projects/:id      (protected)
PATCH  /api/projects/:id      (protected)
DELETE /api/projects/:id      (protected)
POST   /api/projects/:id/columns (protected)
PATCH  /columns/:id           (protected)
DELETE /columns/:id           (protected)
GET    /api/stats             (protected)
```

## Data Models

```
users (email, password_hash, name, role ENUM, settings JSONB, created_at, updated_at)
├── projects (title, description, view_type ENUM, position, archived, created_at, updated_at)
│   ├── project_columns (name, color, position, is_done, created_at)
│   └── tasks (title, body, position, sequential_limit, start_date, due_date, completed_at, reviewed_at, ...)
│       ├── tasks (subtasks via parent_id self-reference)
│       └── task_tags → tags (join table)
├── tags (name, color, UNIQUE per user)
└── images (path, filename, content_type, size_bytes)
```

DB enums: `user_role` (admin, user), `project_view_type` (list, kanban).
Triggers: `update_updated_at()` on users, projects, tasks.

## Key Patterns

- **Auth:** JWT (7-day expiry) in httpOnly cookie. REST API also accepts `Authorization: Bearer` header. Server functions extract auth via `leptos_axum::extract()`. Password hashing with Argon2.
- **Sequential tasks:** `tasks.sequential_limit` controls how many subtasks are actionable. Computed at query time, not stored.
- **Custom columns:** Each project has its own columns (statuses). Created from user's `default_columns` setting.
- **Data access:** Runtime `sqlx::query_as::<_, RowType>()` with custom `FromRow` structs and `From<Row>` conversions to domain types.

## Code Conventions

- **Rust edition 2021**, stable toolchain
- **Type hints** on all public functions
- **Imports** — grouped: std → external crates → local crates, sorted alphabetically
- **Error handling** — use `thiserror` for domain errors, `anyhow` avoided. Map sqlx errors to specific `AppError` variants
- **Test files** — `_test.rs` suffix or `tests/` module, standard Rust conventions
- **Logging** — `tracing` crate, never `println!` for errors/info
- **Time** — always `chrono::Utc`, never naive datetimes
- **No inline comments for obvious code**
- **Line length** — 100 characters preferred

## Common Workflows

### Add New Domain Type
1. Create type file in `crates/domain/src/`
2. Export in `crates/domain/src/lib.rs`
3. Add migration in `migrations/`
4. `docker compose exec app just migrate`

### Add New Page
1. Create page component in `crates/app/src/pages/`
2. Export in `crates/app/src/pages/mod.rs`
3. Add server functions for data fetching
4. Register route in `crates/app/src/app.rs`
5. Add nav item in `crates/app/src/components/nav.rs`

### Add New REST Endpoint
1. Add handler in `crates/server/src/routes/`
2. Register in router in `crates/server/src/routes/mod.rs`
3. Protect with auth middleware if needed

## Infrastructure

- PostgreSQL 17, Redis 7 (Redis reserved for future use)
- Docker Compose for development (all commands via `docker compose exec app`)
- Images stored on filesystem (`./uploads/` volume mount)
- CI: GitHub Actions — fmt, clippy, test, cargo-leptos release build against PostgreSQL 17 service
