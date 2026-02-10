# North

GTD-inspired task management system built with Rust. Sequential subtask workflows, configurable review cycles, kanban visualization, and a JQL-like filter DSL.

## Stack

- **Backend + Frontend:** Rust — [Axum](https://github.com/tokio-rs/axum) + [Leptos](https://github.com/leptos-rs/leptos) (SSR + WASM hydration)
- **Database:** PostgreSQL 17, [Diesel](https://diesel.rs/) ORM with [diesel-async](https://github.com/weiznich/diesel_async)
- **Sessions:** Redis 7
- **Styling:** TailwindCSS 4
- **Migrations:** Diesel CLI (reversible up/down migrations)

## Prerequisites

- Docker & Docker Compose

## Getting Started

```bash
# Start db and redis
docker compose up -d db redis

# Enter the app container (interactive shell)
docker compose run --rm -ti --service-ports app bash

# Inside the container:
just migrate          # Run database migrations
just seed             # Seed admin account (admin@north.local / admin)
just dev              # Start dev server (http://localhost:3000)
```

## Development Commands

Run inside the app container (`docker compose run --rm -ti --service-ports app bash`):

```bash
just dev              # Dev server with hot reload
just test             # Run tests
just fmt              # Format code
just lint             # Clippy
just check            # fmt + lint + test
just migrate          # Apply migrations
just migration name   # Create new migration
just migrate-revert   # Revert last migration
just migrate-redo     # Revert + reapply (test reversibility)
just build            # Release build
```

For CI or non-interactive use:

```bash
docker compose exec app just test
```

## Project Structure

```
north/
├── crates/
│   ├── domain/     # Shared types (Task, Project, User, etc.) — no IO
│   ├── db/         # Diesel schema, models, connection pool
│   ├── services/   # Business logic (TaskService, ProjectService, etc.)
│   ├── app/        # Leptos components, pages, server functions
│   └── server/     # Axum binary, REST API, auth, middleware
├── migrations/     # Diesel reversible migrations (up.sql + down.sql)
├── style/          # TailwindCSS entry point
├── public/         # Static assets
├── docker/         # Dockerfiles
└── docs/           # PRD, design system
```

## Features

- **Inbox** — capture tasks, process later
- **Today** — actionable tasks (start_at <= today)
- **All Tasks** — overview of every task across projects
- **Sequential tasks** — subtasks with configurable N-next visibility
- **Projects** — list or kanban view, custom columns per project, dedicated project pages
- **Archive** — archive/unarchive/delete projects; archived project tasks hidden from Today and All Tasks
- **Reviews** — GTD-style, per-task reviewed_at tracking with configurable interval
- **Tags** — per-user tags with inline `#tag` parsing in task titles
- **Project references** — inline `@project` parsing to assign tasks to projects
- **Settings** — configurable review interval, default columns, sequential limits
- **Filter DSL** — JQL-like query language (`actionable = true & tags in ["work:*"]`)
- **Statistics** — open/closed today, week, total
- **Markdown & images** — full CommonMark with image upload

## Auth

No self-registration. Default admin account is seeded via `just seed`. Admin creates all other accounts.

## License

Private — Zorya Development
