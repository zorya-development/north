# North — Product Requirements Document

## Vision

A GTD-inspired task management system for personal productivity. Sequential subtask workflows, configurable review cycles, kanban visualization, a JQL-like filter DSL, and rich markdown support — built as a single Rust binary.

## Tech Stack

| Layer | Choice | Rationale |
|-------|--------|-----------|
| Language | Rust | Compile-time safety, single binary deployment |
| Backend | Axum | Mature, tokio-based, ergonomic extractors |
| Frontend | Leptos 0.7+ | SSR + WASM hydration, server functions, reactive |
| Database | PostgreSQL 17 | JSONB for settings, enums, strong migration tooling |
| Migrations | sqlx-cli | Compile-time checked queries |
| Sessions | Redis 7 | Session store, rate limiting |
| Styling | TailwindCSS 4 | Utility-first, works with Leptos via cargo-leptos |
| Auth | argon2 + JWT (short-lived) + httpOnly refresh cookie | |
| Image storage | Filesystem (dev), S3-compatible (prod) | |
| Task runner | just | Consistent with existing workflow |
| Containers | Docker Compose | All development happens inside containers |
| CI/CD | GitHub Actions | fmt, clippy, sqlx check, test, build |

## Project Structure

```
north/
├── Cargo.toml                  # workspace
├── crates/
│   ├── domain/                 # types, validation, business rules (no IO)
│   │   └── src/
│   │       ├── task.rs
│   │       ├── project.rs
│   │       ├── column.rs
│   │       ├── user.rs
│   │       ├── tag.rs
│   │       ├── filter_dsl.rs   # DSL parser & evaluator
│   │       └── stats.rs
│   ├── server/                 # axum: routes, repos, middleware, auth
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── auth/
│   │       ├── repo/           # database repositories
│   │       ├── routes/         # REST API handlers
│   │       ├── middleware/
│   │       ├── storage/        # filesystem / S3 image storage
│   │       └── error.rs
│   └── app/                    # leptos: components, pages, styles
│       └── src/
│           ├── lib.rs
│           ├── pages/
│           │   ├── inbox.rs
│           │   ├── today.rs
│           │   ├── all_tasks.rs
│           │   ├── project.rs
│           │   ├── filter.rs
│           │   ├── review.rs
│           │   ├── stats.rs
│           │   ├── settings.rs
│           │   └── admin/
│           │       └── users.rs
│           ├── components/
│           │   ├── task_card.rs
│           │   ├── task_form.rs        # inline add form
│           │   ├── task_modal.rs       # global create modal
│           │   ├── kanban_column.rs
│           │   ├── markdown_editor.rs
│           │   ├── markdown_view.rs
│           │   ├── review_list.rs
│           │   ├── filter_bar.rs
│           │   └── nav.rs
│           └── app.rs          # router, layout, global header
├── migrations/
├── style/
│   └── main.css                # tailwind entry
├── public/                     # static assets
├── uploads/                    # image uploads (dev, gitignored)
├── docker/
│   ├── dev/Dockerfile
│   └── prod/Dockerfile
├── docker-compose.yml
├── justfile
├── CLAUDE.md
├── .github/workflows/
│   ├── ci.yml
│   └── deploy.yml
└── docs/
    └── PRD.md
```

## Build & Development Commands

All commands run inside the Docker container:

```bash
docker compose exec app just dev               # Run dev server (cargo-leptos watch)
docker compose exec app just test              # Run all tests
docker compose exec app just test crate_name   # Run tests for specific crate
docker compose exec app just fmt               # Format code
docker compose exec app just lint              # cargo clippy
docker compose exec app just migrate           # Apply migrations (sqlx)
docker compose exec app just migration name    # Create new migration
docker compose exec app just build             # Release build
docker compose exec app just sqlx-prepare      # Regenerate offline query cache
```

## Data Model

### users

| Column | Type | Notes |
|--------|------|-------|
| id | BIGSERIAL PK | |
| email | TEXT UNIQUE NOT NULL | |
| password_hash | TEXT NOT NULL | argon2 |
| name | TEXT NOT NULL | |
| role | user_role ENUM | `admin`, `user` |
| settings | JSONB NOT NULL DEFAULT '{}' | preferences (see below) |
| created_at | TIMESTAMPTZ NOT NULL | |
| updated_at | TIMESTAMPTZ NOT NULL | |

**User settings JSONB shape:**

```json
{
  "review_interval_days": 7,
  "default_sequential_limit": 1,
  "default_columns": [
    { "name": "To Do", "color": "#6b7280" },
    { "name": "In Progress", "color": "#3b82f6" },
    { "name": "Done", "color": "#22c55e", "is_done": true }
  ]
}
```

### projects

| Column | Type | Notes |
|--------|------|-------|
| id | BIGSERIAL PK | |
| user_id | BIGINT FK → users | |
| title | TEXT NOT NULL | |
| description | TEXT | markdown |
| view_type | project_view_type ENUM | `list`, `kanban` |
| position | INTEGER NOT NULL | sidebar ordering |
| archived | BOOLEAN NOT NULL DEFAULT false | soft delete |
| created_at | TIMESTAMPTZ NOT NULL | |
| updated_at | TIMESTAMPTZ NOT NULL | |

### project_columns

Custom statuses per project. Created from user's `default_columns` setting when a new project is made.

| Column | Type | Notes |
|--------|------|-------|
| id | BIGSERIAL PK | |
| project_id | BIGINT FK → projects | |
| name | TEXT NOT NULL | e.g. "To Do", "In Progress", "Done" |
| color | TEXT NOT NULL | hex, e.g. `#3b82f6` |
| position | INTEGER NOT NULL | column ordering |
| is_done | BOOLEAN NOT NULL DEFAULT false | tasks in this column count as completed |
| created_at | TIMESTAMPTZ NOT NULL | |

### tasks

| Column | Type | Notes |
|--------|------|-------|
| id | BIGSERIAL PK | |
| project_id | BIGINT FK → projects | nullable (inbox tasks have no project) |
| parent_id | BIGINT FK → tasks | nullable (top-level task if null) |
| column_id | BIGINT FK → project_columns | nullable (inbox tasks have no column) |
| user_id | BIGINT FK → users | |
| title | TEXT NOT NULL | |
| body | TEXT | markdown with image refs |
| position | INTEGER NOT NULL | ordering within parent/project |
| sequential_limit | SMALLINT NOT NULL DEFAULT 1 | how many subtasks are actionable (1–3) |
| start_date | DATE | task becomes actionable on this date |
| due_date | DATE | hard deadline |
| completed_at | TIMESTAMPTZ | set when moved to is_done column |
| reviewed_at | TIMESTAMPTZ | last GTD review timestamp |
| created_at | TIMESTAMPTZ NOT NULL | |
| updated_at | TIMESTAMPTZ NOT NULL | |

**Computed property — `actionable`:**

A task is actionable when ALL of these are true:

1. Not completed (`completed_at IS NULL`)
2. `start_date` is null or `start_date <= today`
3. If it has a parent: it is within the first N incomplete subtasks of that parent (where N = `parent.sequential_limit`)

This is computed at query time, not stored. Used by the Today page and filter DSL.

### tags

| Column | Type | Notes |
|--------|------|-------|
| id | BIGSERIAL PK | |
| user_id | BIGINT FK → users | |
| name | TEXT NOT NULL | supports namespaced tags like `work:urgent` |
| color | TEXT NOT NULL | hex |

### task_tags

| Column | Type | Notes |
|--------|------|-------|
| task_id | BIGINT FK → tasks | composite PK |
| tag_id | BIGINT FK → tags | composite PK |

### images

| Column | Type | Notes |
|--------|------|-------|
| id | BIGSERIAL PK | |
| user_id | BIGINT FK → users | |
| task_id | BIGINT FK → tasks | nullable |
| path | TEXT NOT NULL | filesystem path (dev) or S3 key (prod) |
| filename | TEXT NOT NULL | original filename |
| content_type | TEXT NOT NULL | MIME type |
| size_bytes | BIGINT NOT NULL | |
| created_at | TIMESTAMPTZ NOT NULL | |

## REST API

### Tasks

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/tasks` | List tasks with filters (see query params below) |
| POST | `/api/tasks` | Create task |
| GET | `/api/tasks/:id` | Get single task with subtasks |
| PATCH | `/api/tasks/:id` | Update task fields |
| DELETE | `/api/tasks/:id` | Delete task |
| PATCH | `/api/tasks/:id/move` | Reorder / move to column / reparent |
| PATCH | `/api/tasks/:id/review` | Mark as reviewed (sets `reviewed_at = now`) |

**GET `/api/tasks` query parameters:**

| Param | Type | Example | Notes |
|-------|------|---------|-------|
| project | i64 | `?project=1` | filter by project |
| parent | i64 | `?parent=5` | subtasks of a task |
| column | i64 | `?column=3` | filter by column |
| tag | string | `?tag=work:urgent` | filter by tag (repeatable for AND) |
| actionable | bool | `?actionable=true` | computed filter |
| review_due | bool | `?review_due=true` | tasks needing review |
| inbox | bool | `?inbox=true` | tasks with no project |
| completed | bool | `?completed=false` | default: false |
| q | string | `?q=search+term` | full-text search on title + body |
| sort | string | `?sort=position` | position, created_at, due_date, updated_at |
| limit | i64 | `?limit=50` | pagination |
| offset | i64 | `?offset=0` | pagination |

### Projects

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/projects` | List projects |
| POST | `/api/projects` | Create project (auto-creates default columns) |
| GET | `/api/projects/:id` | Get project with columns |
| PATCH | `/api/projects/:id` | Update project |
| DELETE | `/api/projects/:id` | Archive project |

### Columns

| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/api/projects/:id/columns` | Add column |
| PATCH | `/api/columns/:id` | Update column (name, color, position) |
| DELETE | `/api/columns/:id` | Remove column (must reassign tasks first) |

### Tags

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/tags` | List tags |
| POST | `/api/tags` | Create tag |
| PATCH | `/api/tags/:id` | Update tag |
| DELETE | `/api/tags/:id` | Delete tag |

### Images

| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/api/images` | Upload image (multipart) |
| GET | `/api/images/:id` | Serve image |

### Auth

| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/api/auth/login` | Login → access token + refresh cookie |
| POST | `/api/auth/refresh` | Refresh access token |
| POST | `/api/auth/logout` | Revoke refresh token |

### Admin

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/admin/users` | List users (admin only) |
| POST | `/api/admin/users` | Create user (admin only) |
| PATCH | `/api/admin/users/:id` | Update user name/password (admin only) |
| DELETE | `/api/admin/users/:id` | Delete user (admin only) |

### User

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/me` | Current user profile + settings |
| PATCH | `/api/me` | Update own name/password |
| PATCH | `/api/me/settings` | Update settings (columns defaults, review interval, etc.) |

### Stats

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/stats` | Aggregated stats (today/week/total open/closed) |

## Filter DSL

The filter page accepts a text query in a JQL-like syntax:

```
actionable = true & created_at < $today & tags in ["work:*"]
```

### Supported operators

| Operator | Example | Notes |
|----------|---------|-------|
| `=` | `actionable = true` | equality |
| `!=` | `status != done` | inequality |
| `<`, `>`, `<=`, `>=` | `created_at < $today` | comparison (dates, numbers) |
| `in` | `tags in ["work:*", "personal"]` | set membership, supports `*` glob |
| `not in` | `tags not in ["archive"]` | exclusion |
| `&` | combines conditions | logical AND |
| `\|` | alternative | logical OR |
| `()` | grouping | precedence |

### Variables

| Variable | Resolves to |
|----------|-------------|
| `$today` | current date |
| `$week_start` | monday of current week |
| `$month_start` | first of current month |

### Queryable fields

`actionable`, `completed`, `project`, `column`, `tags`, `title`, `created_at`, `updated_at`, `due_date`, `start_date`, `reviewed_at`

The DSL is parsed in `crates/domain/src/filter_dsl.rs` using `nom` or `pest`, then translated to SQL WHERE clauses in the repository layer.

## Features

### F1: Auth & Admin

- No self-registration. Default seed account: `admin@north.local` / `admin`
- Admin can change own name + password
- Admin can create/edit/delete user accounts on `/admin/users`
- JWT access token (15min) + httpOnly refresh token cookie (7d)

### F2: Inbox

- Default landing page after login
- Inline task form at top (title, submit with Enter)
- List of tasks with `project_id = NULL`
- Each task: title, body preview, quick actions (assign to project, set dates)
- Drag-and-drop reorder

### F3: Today

- Tasks where `actionable = true` (see computed property definition)
- Grouped by project (with "No Project" section for inbox actionable tasks)
- Each task: title, project label, due date badge, tag chips
- Tasks are editable inline (click to expand)

### F4: All Tasks

- Flat list of all tasks across all projects
- Each task shows: title, project label (clickable), column, tags, dates
- Tasks are editable inline
- Sortable by: position, created, updated, due date
- Filterable by: project, column, tag, completed status

### F5: Project Page

- Two view modes: **list** and **kanban** (togglable per project)
- **List view:** ordered task list with subtask expansion, inline edit, inline add form at top
- **Kanban view:** columns from `project_columns`, drag-and-drop between columns, inline add per column
- Subtask expand: clicking a parent task expands its subtasks, shows which are actionable (visual indicator for the first N)
- Project settings panel: title, description, view type, manage columns (add/rename/reorder/remove/set color)

### F6: Filter Page

- Text input at top for DSL query
- Query results displayed as a task list below
- Saved filters (name + query string, stored in user settings JSONB)
- Sidebar shows saved filters for quick access

### F7: Reviews (GTD)

- Review page shows all tasks where: `reviewed_at IS NULL` OR `reviewed_at < now() - user.review_interval_days`
- Sorted by: most overdue first (longest since last review)
- Each task: title, project label, last reviewed date, days overdue
- Review action: button to mark as reviewed → sets `reviewed_at = now()`
- Visual indicators: green (fresh), yellow (due soon), red (overdue)
- Bulk "mark all reviewed" per project group

### F8: Statistics

- **Today:** tasks created, tasks completed
- **This week:** tasks created, tasks completed (Mon–Sun)
- **All time:** total open, total completed
- **Completion rate:** percentage + trend
- **Per-project breakdown:** open task count per project
- Data computed from task timestamps at query time — no separate analytics table

### F9: Markdown & Images

- Task body supports full CommonMark markdown
- Rendered with syntax highlighting for code blocks
- Image upload: drag-and-drop onto editor or paste from clipboard
- Images stored on filesystem (dev: `./uploads/`, prod: S3-compatible)
- Referenced in markdown as `![alt](/api/images/:id)`
- Image size limit: 10MB
- Supported formats: PNG, JPEG, WebP, GIF

### F10: Global Task Creation

- **Header button:** persistent "+" button in the navigation bar, opens a modal
- **Modal form:** title, project (dropdown), column (dropdown, filtered by selected project), body (markdown editor), tags, dates
- **Inline form:** appears at the top of Inbox, Today, Project pages — just a title input, creates task with Enter, inherits page context (project, column)

## Pages & Routes

| Route | Page | Description |
|-------|------|-------------|
| `/inbox` | Inbox | Unassigned tasks, inline add |
| `/today` | Today | Actionable tasks grouped by project |
| `/tasks` | All Tasks | Flat list, filterable, sortable |
| `/projects/:id` | Project | List or kanban view, inline edit + add |
| `/filter` | Filter | DSL query input + results |
| `/filter/:saved_id` | Saved Filter | Pre-filled DSL query |
| `/review` | Review | GTD review queue |
| `/stats` | Stats | Dashboard |
| `/settings` | Settings | User preferences (default columns, review interval) |
| `/admin/users` | Admin: Users | Manage accounts (admin only) |
| `/login` | Login | |

## Docker Compose Services (Development)

| Service | Image | Port | Purpose |
|---------|-------|------|---------|
| app | north:dev (built from docker/dev/Dockerfile) | 3000 | Leptos SSR + WASM, cargo-leptos watch |
| db | postgres:17-alpine | 5432 | Primary database |
| redis | redis:7-alpine | 6379 | Sessions, rate limiting |

No MinIO — images stored on filesystem via volume mount in development.

## CI/CD Pipeline

### ci.yml (on every PR / push)

1. `cargo fmt --check`
2. `cargo clippy -- -D warnings`
3. `cargo sqlx prepare --check` (verify offline query cache)
4. `cargo test`
5. `cargo leptos build --release` (verify full build)

### deploy.yml (on merge to main)

1. Build Docker image from `docker/prod/Dockerfile`
2. Push to container registry
3. Deploy to target environment (TBD)

## Seed Data

On first run (via migration or startup check):

- Create admin user: `admin@north.local` / `admin` / role: admin
- No sample projects or tasks — clean start

## Non-Functional Requirements

- **Performance:** < 200ms TTFB for SSR pages, < 50ms for API responses
- **Bundle size:** WASM < 500KB gzipped
- **Database:** all queries indexed, no N+1, `actionable` computed efficiently with window functions
- **Security:** CSRF protection, rate limiting on auth, input sanitization, admin-only routes guarded
- **Accessibility:** semantic HTML, keyboard navigation, ARIA labels

## Milestones

### Phase 1: Foundation

- [ ] Project scaffolding (workspace, crates, docker-compose, justfile, CLAUDE.md)
- [ ] Database schema + migrations
- [ ] Admin seed account
- [ ] Auth (login, refresh, logout, middleware)
- [ ] Basic task CRUD (REST API + simple list view)
- [ ] Basic project CRUD with columns
- [ ] CI pipeline

### Phase 2: Core GTD

- [ ] Subtask support (parent_id, sequential_limit)
- [ ] Actionable computed property (query logic)
- [ ] Inbox page with inline add
- [ ] Today page (actionable tasks grouped by project)
- [ ] Review system (reviewed_at tracking, review page, mark as reviewed)
- [ ] Global task creation (header button + modal)

### Phase 3: Views & Rich Content

- [ ] Project page — list view with inline edit
- [ ] Project page — kanban view with drag-and-drop
- [ ] Column management (add/remove/rename/reorder per project)
- [ ] All Tasks page
- [ ] Markdown rendering in task body
- [ ] Image upload (filesystem) + clipboard paste
- [ ] Tags CRUD + assignment + filtering

### Phase 4: Filters & Analytics

- [ ] Filter DSL parser (nom/pest)
- [ ] Filter page with saved filters
- [ ] Statistics dashboard (today/week/total)
- [ ] Admin user management page
- [ ] User settings page (default columns, review interval)
- [ ] Responsive design pass
- [ ] Deploy pipeline
