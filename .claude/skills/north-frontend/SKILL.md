---
name: north-frontend
description: Frontend development guide for the North app crate — Tailwind v4, a11y, component architecture, and styling conventions.
trigger: When writing or modifying files in crates/app/, crates/ui/, crates/stores/, crates/repositories/, or style/main.css. Activated when working on Leptos views, components, containers, pages, stores, repositories, or CSS.
---

# North Frontend Development Guide

This skill covers styling, accessibility, component architecture, and Tailwind v4 conventions for the North app crate. Follow these rules when creating or modifying frontend code.

---

## 1. Component Architecture

### 1.1 Container / Controller / View Pattern

Pages and complex components follow a strict separation:

| File | Responsibility | Allowed dependencies |
|---|---|---|
| `container.rs` | Entry point. Wires controller to view with **inline** `Callback` props. No intermediate variables for callbacks. | Stores, controller |
| `controller.rs` | Business logic, data loading, `Memo` creation, store interaction. | Stores, repositories (via stores) |
| `view.rs` | Pure rendering. Receives **all** data and handlers as props. Zero business logic. | Props only (signals, memos, callbacks) |

**When to use each decomposition:**

| Complexity | Pattern | Files |
|---|---|---|
| Full page with data loading + filtering | Container + Controller + View | 4 files (+ mod.rs) |
| Complex stateful domain component (pickers, modals, task items) | Container + View | 3 files (+ mod.rs) |
| Simple presentational component | Single file | 1 file |

### 1.2 Containers vs Components

- **`containers/`** — Complex stateful domain components that wire stores, repositories, and rich interactions (pickers, sidebar, autocomplete, task list item, inline form, detail modal).
- **`components/`** — Simpler or more presentational components (task list, date picker, layout, filter autocomplete, task meta).

**Decision rule:** If the component manages store interaction, DOM positioning (Popover), or complex state machines, it belongs in `containers/`. If it primarily renders data it receives via props, it belongs in `components/`.

### 1.3 Prop Conventions

```rust
// View props — always reactive types
pub fn InboxView(
    active_task_ids: Memo<Vec<i64>>,        // Derived filtered data
    is_loaded: Signal<bool>,                 // Store-derived state
    is_form_open: ReadSignal<bool>,          // Local UI state
    set_form_open: WriteSignal<bool>,        // Local UI state writer
    on_task_click: Callback<i64>,            // Event handler
    on_reorder: Callback<(i64, String, Option<Option<i64>>)>,
) -> impl IntoView
```

**Rules:**
- Views receive `Memo<T>`, `Signal<T>`, `ReadSignal<T>`, `WriteSignal<T>`, and `Callback<T>` — never raw data or stores.
- Callbacks are inlined in the container: `on_click=Callback::new(move |id| ctrl.do_thing(id))` — no intermediate variables.
- Use `#[prop(into)]` for flexible signal acceptance. Use `#[prop(optional)]` for optional callbacks.
- Pickers support `icon_only: bool` prop for compact rendering in task card action bars.

### 1.4 Layer Rules

```
Page → Store → Repository → Server Function
 ↓
View (pure rendering)
```

- Pages talk to **Stores**, never deeper.
- Stores talk to **Repositories**, never server functions directly.
- Views talk to **nothing** — they receive everything via props.
- Context: use `provide_context()` directly in containers. Consume via `expect_context::<T>()` or typed helpers (`use_app_store()`).

### 1.5 Domain Models (repositories crate)

Domain models live in `repositories/src/models/`. They convert API DTOs into frontend-friendly structs — unfolding compressed fields (e.g. raw RRULE strings → a `Recurrence` struct) and adding methods that centralize entity logic in one place.

**When needed:** Writing controllers or store logic that works with data coming from the backend. If you find yourself parsing or interpreting DTO fields in multiple places, that logic belongs in a domain model.

**Pattern:** `From<Dto>` converts the DTO, optionally unfolding fields into richer types. Methods on the model keep domain logic co-located with the entity.

Check `repositories/src/models/` when working with frontend data models or creating new ones.

### 1.6 Reactive Callbacks

When a `Callback` prop's behavior depends on reactive state (toggles, settings, filters), wrap it in `Signal::derive` → `Signal<Callback<T, R>>`. The derive tracks the dependencies; the inner callback captures their snapshot as plain values. This eliminates the need for a separate "trigger" signal to force re-evaluation. Consumers call `signal.get()` inside a Memo to subscribe, then `callback.run(...)`.

### 1.7 Data Loading

- Each page owns its data loading — call `refetch()` or create its own `Resource` on mount.
- `AppLayout` is purely structural (auth guard, context providers, sidebar + main shell). It does NOT pre-fetch data.
- Use `Resource` for SSR-compatible async data. Use `LocalResource` only for browser-only APIs.

---

## 2. Tailwind CSS v4 Conventions

### 2.1 Theme Architecture

The project uses a three-layer theme pattern in `style/main.css`:

```
:root { --bg-primary: #F9F8F6; }     ← Semantic CSS variables (light)
.dark { --bg-primary: #1C1D2B; }     ← Dark overrides
@theme { --color-bg-primary: var(--bg-primary); }  ← Tailwind utility bridge
```

**Rules:**
- Never use raw color values in templates. Always use semantic tokens: `bg-bg-primary`, `text-text-secondary`, `border-border`.
- Never use `dark:` prefix utilities — dark mode is handled by CSS variable switching.
- New colors: add to `:root`, `.dark`, AND `@theme` in `style/main.css`.
- New custom utilities: use the `@utility` directive (gets variant support for free).

### 2.2 Color Tokens

| Token | Usage |
|---|---|
| `bg-primary` | Main background |
| `bg-secondary` | Sidebar, cards |
| `bg-tertiary` | Hover backgrounds, code blocks |
| `bg-input` | Form input backgrounds |
| `text-primary` | Primary text |
| `text-secondary` | Secondary/muted text |
| `text-tertiary` | Placeholder, disabled text |
| `accent` / `accent-hover` | Primary action color |
| `danger` / `danger-hover` | Destructive actions |
| `warning` | Warning indicators |
| `success` | Success indicators |
| `on-accent` | Text on accent background |

### 2.3 Spacing & Layout

- Flexbox: `flex items-center justify-between gap-2`
- Vertical stacking: `space-y-4` or `flex flex-col gap-4`
- Padding: `px-3 py-1.5` (buttons), `p-1` (icon buttons), `p-4` (sections)
- Max widths: `max-w-md`, `max-w-lg`, `max-w-2xl` for content areas

### 2.4 Typography

- Sizes: `text-xs` (11px meta), `text-sm` (13px default), `text-base`, `text-lg`, `text-2xl` (headings)
- Weights: `font-medium` (labels), `font-semibold` (headings, emphasis)
- Colors: `text-text-primary` (main), `text-text-secondary` (labels), `text-text-tertiary` (hints)
- Font families: `font-sans` (Inter), `font-mono` (JetBrains Mono, for code)

### 2.5 Interaction States

Every interactive element MUST have visible state transitions:

```rust
// Standard hover + transition
class="hover:bg-bg-tertiary transition-colors"

// Text hover (secondary → primary)
class="text-text-secondary hover:text-text-primary transition-colors"

// Icon button hover
class="text-text-tertiary hover:text-text-secondary hover:bg-bg-input transition-colors"
```

**Always include `transition-colors` (or `transition-opacity`, `transition-all`) on elements with hover/focus state changes.**

### 2.6 Responsive Design

- Build mobile-first: unprefixed utilities apply to all sizes.
- Use Tailwind breakpoints (`sm:`, `md:`, `lg:`) for responsive overrides.
- Prefer container queries (`@container` / `@sm:`, `@md:`) for reusable components that need to adapt to parent size rather than viewport.

---

## 3. Accessibility (a11y)

### 3.1 Interactive Elements — Mandatory Rules

**Use semantic HTML. Always.**

```rust
// CORRECT: native button
<button on:click=handler>"Delete"</button>

// WRONG: div pretending to be a button
<div on:click=handler>"Delete"</div>
```

- Clickable actions → `<button>`
- Navigation → `<a href="...">`
- Never use `<div>` or `<span>` with `on:click` for interactive elements.

**`cursor-pointer` on all clickable non-link elements:**

```rust
// Buttons that perform actions
class="cursor-pointer ..."

// Clickable cards, rows, toggles
class="cursor-pointer ..."
```

Tailwind v4 defaults buttons to `cursor: default` (browser behavior). You MUST add `cursor-pointer` explicitly.

### 3.2 Focus Indicators

The project uses a global `focus-visible` style in `main.css`:

```css
*:focus-visible {
  outline: 2px solid var(--accent);
  outline-offset: 2px;
  border-radius: 4px;
}
```

**Rules:**
- Never remove focus indicators without providing an alternative.
- Use `outline` (not `ring`) for focus indicators — `ring` uses `box-shadow` which is invisible in Windows High Contrast Mode.
- The `.no-focus-ring` class exists for special cases — use sparingly.
- Hover and focus-visible states should be visually consistent. If a button changes background on hover, apply the same on `focus-visible:`.

### 3.3 Hover/Focus State Consistency

Every interactive element needs BOTH hover and focus-visible states:

```rust
class="hover:bg-bg-tertiary focus-visible:bg-bg-tertiary transition-colors cursor-pointer"
```

### 3.4 Icon-Only Buttons

Every button with only an icon MUST have an accessible name:

```rust
// Option 1: aria-label (preferred for brevity)
<button aria-label="Delete task" class="p-1 cursor-pointer ...">
    <Icon kind=IconKind::Trash class="w-4 h-4" />
</button>

// Option 2: sr-only text
<button class="p-1 cursor-pointer ...">
    <Icon kind=IconKind::Trash class="w-4 h-4" />
    <span class="sr-only">"Delete task"</span>
</button>
```

Decorative icons alongside text get `aria-hidden="true"` (already handled by the `Icon` component for most cases).

### 3.5 ARIA for Complex Widgets

**Dropdowns / Menus:**
```rust
<button aria-expanded=is_open aria-haspopup="menu">"Options"</button>
// When open:
<div role="menu">
    <button role="menuitem">"Edit"</button>
    <button role="menuitem">"Delete"</button>
</div>
```

**Modals:**
```rust
<div role="dialog" aria-modal="true" aria-labelledby="modal-title">
    <h2 id="modal-title">"Edit Task"</h2>
    // ...
</div>
```

**Checkboxes (custom):**
```rust
<button role="checkbox" aria-checked=is_checked aria-label="Complete task">
```

### 3.6 Dynamic Content — aria-live Regions

When content changes without a page reload (optimistic updates, completions, errors), announce it:

```rust
// Non-urgent status updates
<div aria-live="polite">{status_message}</div>

// Errors and critical alerts
<div aria-live="assertive" role="alert">{error_message}</div>
```

- Use `polite` for task completions, save confirmations, filter results.
- Use `assertive` only for errors and critical alerts.
- The `aria-live` container must exist in the DOM BEFORE the content changes.

### 3.7 Forms

```rust
// Always associate labels with inputs
<label for="task-title" class="text-sm font-medium text-text-secondary">
    "Task Title"
</label>
<input id="task-title" type="text" ... />

// When no visible label, use aria-label
<input type="search" aria-label="Search tasks" placeholder="Search..." ... />

// Error messages: link with aria-describedby
<input id="email" aria-invalid="true" aria-describedby="email-error" ... />
<span id="email-error" role="alert" class="text-sm text-danger">
    "Invalid email"
</span>
```

**Never rely on `placeholder` alone as a label.** Placeholders disappear on input and have poor contrast.

### 3.8 Disabled States

```rust
// Native disabled (removes from tab order)
<button disabled class="disabled:opacity-50 disabled:cursor-not-allowed">

// aria-disabled (stays focusable — good for submit buttons that trigger validation)
<button aria-disabled=is_disabled
    class="aria-disabled:opacity-50 aria-disabled:cursor-not-allowed"
    on:click=move |_| {
        if is_disabled.get() { return; }
        // proceed
    }
>
```

### 3.9 Reduced Motion

Respect user preferences:
```rust
class="transition-transform motion-reduce:transition-none"
```

### 3.10 Color Contrast

- Normal text: **4.5:1** minimum contrast ratio (WCAG AA)
- Large text (>=24px or >=18.5px bold): **3:1** minimum
- UI components (borders, icons): **3:1** minimum
- Never use color as the sole indicator of state — always pair with icons, text, or other visual cues.

---

## 4. UI Kit Atoms (`app/src/atoms/`)

Semantic, multi-dimension prop components in `crates/app/src/atoms/`. Generic UI primitives (Icon, Modal, Popover, etc.) remain in `north-ui`. See `docs/UI_KIT.md` for full catalog and design rationale.

**Rules:**
- Use atoms instead of raw Tailwind for text, buttons, badges, inputs
- Prefer `<Text variant=TextVariant::HeadingLg>` over `<h1 class="text-2xl font-semibold ...">`
- Import via `use crate::atoms::{Text, TextVariant, TextColor, TextTag};`
- Variant controls structure (size/weight/transform). Color is always a separate prop
- Each variant has a default HTML tag. Override with `tag` prop when needed

### 4.1 Text

```rust
use crate::atoms::{Text, TextVariant, TextColor, TextTag};

<Text variant=TextVariant::HeadingLg>"Inbox"</Text>           // <h1>
<Text variant=TextVariant::HeadingMd>"Section"</Text>         // <h2>
<Text variant=TextVariant::TitleMd>{title}</Text>             // task title
<Text variant=TextVariant::LabelMd color=TextColor::Secondary>"Projects"</Text>  // sidebar
<Text variant=TextVariant::LabelMd tag=TextTag::Label color=TextColor::Tertiary>"Due date"</Text>
<Text>"Body text"</Text>                                       // defaults: BodyMd, <span>, Primary
<Text variant=TextVariant::BodySm color=TextColor::Danger>{overdue_date}</Text>
<Text variant=TextVariant::CodeMd color=TextColor::Accent>"query"</Text>  // <code>
<Text truncate=true>{long_text}</Text>
<Text variant=TextVariant::TitleMd color=TextColor::Tertiary line_through=true>{done_task}</Text>
```

Variants: `HeadingLg/Md/Sm`, `TitleLg/Md/Sm`, `BodyLg/Md/Sm`, `LabelLg/Md/Sm`, `CodeMd/Sm`.
Colors: `Primary`, `Secondary`, `Tertiary`, `Accent`, `Danger`, `OnAccent`, `Inherit`.
Tags: `H1`..`H4`, `P`, `Span`, `Label`, `Code` — only specify when overriding variant's default.

### 4.2 Button Variants (raw patterns, atom pending)

```rust
// Primary action
class="px-4 py-1.5 text-sm font-medium bg-accent text-on-accent \
       rounded cursor-pointer hover:bg-accent-hover \
       focus-visible:bg-accent-hover transition-colors"

// Secondary / ghost
class="px-3 py-1.5 text-sm text-text-secondary cursor-pointer \
       hover:text-text-primary hover:bg-bg-tertiary \
       focus-visible:bg-bg-tertiary transition-colors"

// Icon button
class="p-1 rounded text-text-tertiary cursor-pointer \
       hover:text-text-secondary hover:bg-bg-input \
       focus-visible:bg-bg-input transition-colors"

// Danger
class="px-3 py-1.5 text-sm text-danger cursor-pointer \
       hover:bg-bg-tertiary focus-visible:bg-bg-tertiary \
       transition-colors"
```

### 4.3 Form Inputs (raw pattern, atom pending)

```rust
class="w-full bg-bg-input border border-border rounded px-3 py-1.5 \
       text-sm text-text-primary placeholder:text-text-tertiary \
       focus:outline-none focus:border-accent transition-colors"
```

### 4.4 Modal Backdrop

```rust
class="fixed inset-0 z-50 bg-backdrop flex items-center justify-center"
```

### 4.5 Sidebar Navigation Item

```rust
class="flex items-center gap-2 px-3 py-2 rounded-lg text-sm \
       text-text-primary cursor-pointer \
       hover:bg-bg-tertiary transition-colors"
```

### 4.6 Hover-Show Action Buttons (task list items)

```rust
// Container: show children on hover via group
class="group ..."

// Action buttons: hidden until parent hover
class="opacity-0 group-hover:opacity-100 transition-opacity"
```

---

## 5. Quick Checklist

Before submitting frontend code, verify:

- [ ] Use `<Text>` atom for all text — never raw `<span class="text-sm ...">` for new code
- [ ] Interactive elements use `<button>` or `<a>`, never `<div>`/`<span>` with `on:click`
- [ ] All clickable elements have `cursor-pointer`
- [ ] All interactive elements have hover AND focus-visible states
- [ ] All state changes include `transition-colors` (or appropriate transition)
- [ ] Icon-only buttons have `aria-label` or `sr-only` text
- [ ] Colors use semantic tokens (`bg-bg-primary`), never raw values or `dark:` prefixes
- [ ] Form inputs have associated labels (visible or `aria-label`)
- [ ] Modals have `role="dialog"` and `aria-modal="true"`
- [ ] Views are pure — no store access, no business logic
- [ ] Callbacks are inlined in container props
- [ ] New colors added to all three layers (`:root`, `.dark`, `@theme`)
