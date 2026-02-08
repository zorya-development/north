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

GTD-inspired task management system. Single Rust binary serving SSR + WASM via Leptos + Axum.

### Crate Structure

- **`crates/domain/`** — Shared types (Task, Project, Column, User, Tag). Pure data types with serde + chrono, no IO. Compiled for both server and WASM client.
- **`crates/app/`** — Leptos library crate. Components, pages, server functions. Compiled to WASM (hydrate feature) and native (ssr feature). Server functions include inline sqlx queries under `#[cfg(feature = "ssr")]`.
- **`crates/server/`** — Axum binary crate. `main.rs` sets up the server, REST API routes, auth middleware, admin seed. Depends on `north-app` with ssr feature.

### Data Flow

```
Browser → Axum Router
  ├── /api/*  → REST API handlers (server crate, auth via JWT middleware)
  └── /*      → Leptos SSR → server functions (app crate) → DB
                           → hydrated WASM on client
```

### Key Patterns

- **Auth:** JWT in httpOnly cookie. REST API also accepts Authorization header. Server functions extract auth via `leptos_axum::extract()`.
- **Sequential tasks:** `tasks.sequential_limit` controls how many subtasks are actionable. Computed at query time, not stored.
- **Custom columns:** Each project has its own columns (statuses). Created from user's `default_columns` setting.

## Data Models

```
users (email, password_hash, name, role, settings JSONB)
├── projects (title, description, view_type, position, archived)
│   ├── project_columns (name, color, position, is_done)
│   └── tasks (title, body, position, sequential_limit, start_date, due_date, ...)
│       ├── tasks (subtasks via parent_id)
│       └── task_tags → tags
└── images (path, filename, content_type, size_bytes)
```

## Code Conventions

- **Rust edition 2021**, stable toolchain
- **Type hints** on all public functions
- **Imports** — grouped: std → external crates → local crates, sorted alphabetically
- **Error handling** — use `thiserror` for domain errors, `anyhow` avoided. Map sqlx errors to specific app errors
- **Test files** — `_test.rs` suffix or `tests/` module, standard Rust conventions
- **Logging** — `tracing` crate, never `println!` for errors/info
- **SQL** — runtime `sqlx::query_as::<_, T>()` for now, will migrate to compile-time macros when schema stabilizes
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
2. Add server functions for data fetching
3. Register route in `crates/app/src/app.rs`
4. Add nav item in `crates/app/src/components/nav.rs`

### Add New REST Endpoint
1. Add handler in `crates/server/src/routes/`
2. Register in router in `crates/server/src/routes/mod.rs`
3. Protect with auth middleware if needed

## Infrastructure

- PostgreSQL 17, Redis 7
- Docker Compose for development (all commands via `docker compose exec app`)
- Images stored on filesystem (`./uploads/` volume mount)
- CI: GitHub Actions (fmt, clippy, test, build)
