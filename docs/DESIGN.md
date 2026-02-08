# North — Design System

## Tooling

**TailwindCSS 4** via `cargo-leptos` built-in CSS processing. No component library — all components are custom, built with Tailwind utility classes against this design system.

## Color Palette

```css
/* style/main.css */
@import "tailwindcss";

@theme {
  --color-teal-950: #0a2d2e;
  --color-teal-900: #1c4e4f;
  --color-teal-700: #436e6f;
  --color-teal-500: #6a8e8f;
  --color-sage-400: #879693;
  --color-warm-400: #a49e97;
  --color-peach-300: #deae9f;
  --color-peach-200: #efd7cf;
  --color-peach-100: #f7ebe7;
  --color-white: #ffffff;
}
```

### Semantic Mapping

| Role | Token | Hex | Usage |
|------|-------|-----|-------|
| **Surface base** | `white` | #ffffff | Main page background |
| **Surface raised** | `peach-100` | #f7ebe7 | Cards, panels, modal backgrounds |
| **Surface hover** | `peach-200` | #efd7cf | Hovered cards, selected items |
| **Accent** | `peach-300` | #deae9f | Primary buttons, active indicators, badges |
| **Accent hover** | `warm-400` | #a49e97 | Hovered accent elements |
| **Text primary** | `teal-950` | #0a2d2e | Headings, body text, task titles |
| **Text secondary** | `teal-700` | #436e6f | Descriptions, metadata, timestamps |
| **Text muted** | `sage-400` | #879693 | Placeholders, disabled text, hints |
| **Border** | `peach-200` | #efd7cf | Card borders, dividers |
| **Border strong** | `teal-500` | #6a8e8f | Focused inputs, active borders |
| **Sidebar bg** | `teal-950` | #0a2d2e | Left navigation background |
| **Sidebar text** | `peach-100` | #f7ebe7 | Sidebar nav text |
| **Sidebar active** | `teal-900` | #1c4e4f | Active nav item background |
| **Sidebar hover** | `teal-700` | #436e6f | Hovered nav item background |
| **Danger** | `#c44` | — | Delete actions (only deviation from palette) |
| **Success** | `teal-700` | #436e6f | Completed state, review fresh |

## Typography

System font stack — no custom fonts to keep it fast and native-feeling.

```css
@theme {
  --font-sans: "Inter", ui-sans-serif, system-ui, -apple-system, sans-serif;
  --font-mono: "JetBrains Mono", ui-monospace, monospace;
}
```

| Element | Size | Weight | Color |
|---------|------|--------|-------|
| Page title | text-xl (20px) | font-semibold | teal-950 |
| Section heading | text-base (16px) | font-semibold | teal-950 |
| Task title | text-sm (14px) | font-medium | teal-950 |
| Body / description | text-sm (14px) | font-normal | teal-700 |
| Metadata (dates, counts) | text-xs (12px) | font-normal | sage-400 |
| Button label | text-sm (14px) | font-medium | — |

## Spacing

Base unit: 4px. Use Tailwind spacing scale (`p-1` = 4px, `p-2` = 8px, etc.)

| Context | Value |
|---------|-------|
| Card padding | `p-3` (12px) |
| Section gap | `gap-4` (16px) |
| Sidebar width | `w-56` (224px) |
| Page content max-width | `max-w-5xl` (1024px) |
| Page horizontal padding | `px-6` (24px) |

## Border Radius

Subtle rounding — not bubbly.

| Element | Radius |
|---------|--------|
| Cards | `rounded-lg` (8px) |
| Buttons | `rounded-md` (6px) |
| Inputs | `rounded-md` (6px) |
| Badges/tags | `rounded-full` (pill) |
| Avatars | `rounded-full` |

## Layout

```
┌──────────────────────────────────────────────────┐
│  Sidebar (teal-950)  │  Main content (white)     │
│                      │                           │
│  ┌────────────────┐  │  ┌─ Page header ────────┐ │
│  │  Logo / Name   │  │  │ Title        [+ New] │ │
│  ├────────────────┤  │  └──────────────────────┘ │
│  │  ▸ Inbox    12 │  │                           │
│  │  ▸ Today     5 │  │  ┌─ Content ────────────┐ │
│  │  ▸ All Tasks   │  │  │                      │ │
│  ├────────────────┤  │  │  Task cards / Kanban  │ │
│  │  Projects      │  │  │  / Review list / etc  │ │
│  │  ▸ Project A   │  │  │                      │ │
│  │  ▸ Project B   │  │  └──────────────────────┘ │
│  ├────────────────┤  │                           │
│  │  ▸ Review      │  │                           │
│  │  ▸ Filters     │  │                           │
│  │  ▸ Stats       │  │                           │
│  ├────────────────┤  │                           │
│  │  ▸ Settings    │  │                           │
│  └────────────────┘  │                           │
└──────────────────────────────────────────────────┘
```

## Components

### Sidebar Nav Item

```
Default:    bg-transparent  text-peach-100     px-3 py-2 rounded-md
Hover:      bg-teal-700     text-white
Active:     bg-teal-900     text-white         font-medium
```

Badge (task count) floats right, `text-xs text-sage-400`.

### Task Card

```html
<div class="bg-peach-100 rounded-lg p-3 border border-peach-200
            hover:border-teal-500 hover:shadow-sm transition-colors cursor-pointer">
  <div class="flex items-center gap-2">
    <button class="w-4 h-4 rounded-full border-2 border-teal-500
                   hover:bg-teal-500 transition-colors" />  <!-- complete toggle -->
    <span class="text-sm font-medium text-teal-950">Task title here</span>
  </div>
  <div class="mt-1 ml-6 flex items-center gap-2 text-xs text-sage-400">
    <span class="text-teal-700">Project Name</span>
    <span>·</span>
    <span>Due tomorrow</span>
    <span class="bg-peach-200 text-teal-700 px-2 py-0.5 rounded-full">tag</span>
  </div>
</div>
```

- Completed task: title gets `line-through text-sage-400`
- Non-actionable subtask: `opacity-50` overlay
- Expanded (editing): border becomes `border-teal-500`, card grows to show body/dates

### Kanban Column

```html
<div class="flex-shrink-0 w-72">
  <div class="flex items-center justify-between px-1 mb-3">
    <div class="flex items-center gap-2">
      <span class="w-2 h-2 rounded-full" style="background: {column.color}" />
      <span class="text-sm font-semibold text-teal-950">{column.name}</span>
      <span class="text-xs text-sage-400">{count}</span>
    </div>
    <button class="text-sage-400 hover:text-teal-700">+</button>
  </div>
  <div class="flex flex-col gap-2">
    <!-- task cards -->
  </div>
</div>
```

Columns separated by `gap-4`. Board scrolls horizontally on overflow.

### Inline Task Form

```html
<div class="flex items-center gap-2 p-3 bg-white border border-peach-200
            rounded-lg focus-within:border-teal-500">
  <span class="text-sage-400">+</span>
  <input type="text" placeholder="Add a task..."
         class="flex-1 text-sm bg-transparent outline-none
                placeholder:text-sage-400 text-teal-950" />
</div>
```

Sits at the top of Inbox, Project, and Kanban columns. Submits on Enter.

### Primary Button

```
Default:    bg-peach-300 text-teal-950     px-4 py-2 rounded-md font-medium text-sm
Hover:      bg-peach-200
Active:     bg-warm-400
```

### Secondary Button

```
Default:    bg-transparent text-teal-700   px-4 py-2 rounded-md font-medium text-sm border border-peach-200
Hover:      bg-peach-100
```

### Ghost Button

```
Default:    bg-transparent text-sage-400   px-2 py-1 rounded-md text-sm
Hover:      text-teal-700 bg-peach-100
```

Used for icon buttons, secondary actions within cards.

### Input Field

```
Default:    bg-white border border-peach-200 rounded-md px-3 py-2 text-sm text-teal-950
Focus:      border-teal-500 ring-1 ring-teal-500/20
Placeholder: text-sage-400
```

### Badge / Tag

```
Default:    bg-peach-200 text-teal-700 text-xs px-2 py-0.5 rounded-full font-medium
```

Tag color is a 3px left border or a small dot before the name, using the tag's `color` field.

### Review Indicators

```
Fresh (< 50% interval):    text-teal-700     bg-teal-700/10   "Reviewed 2d ago"
Due soon (50-100%):         text-peach-300    bg-peach-300/10  "Review due in 1d"
Overdue (> 100%):           text-[#c44]       bg-[#c44]/10     "3d overdue"
```

### Modal (Global Task Create)

```html
<div class="fixed inset-0 bg-teal-950/40 flex items-start justify-center pt-24">
  <div class="bg-white rounded-lg shadow-xl w-full max-w-lg p-6">
    <h2 class="text-lg font-semibold text-teal-950 mb-4">New Task</h2>
    <!-- form fields -->
    <div class="flex justify-end gap-2 mt-6">
      <button class="...secondary">Cancel</button>
      <button class="...primary">Create</button>
    </div>
  </div>
</div>
```

Backdrop uses `teal-950/40` for a dark teal overlay that stays in palette.

### Stats Card

```html
<div class="bg-peach-100 rounded-lg p-4">
  <span class="text-xs font-medium text-sage-400 uppercase tracking-wide">Completed today</span>
  <span class="text-2xl font-semibold text-teal-950 mt-1 block">12</span>
</div>
```

### Empty State

```html
<div class="flex flex-col items-center justify-center py-16 text-center">
  <span class="text-sage-400 text-sm">No tasks here yet</span>
  <button class="mt-2 text-sm text-teal-700 hover:text-teal-950">+ Add a task</button>
</div>
```

## Transitions

All interactive elements use `transition-colors duration-150`. No heavy animations — keep it snappy.

Drag-and-drop uses a subtle `shadow-lg` on the dragged card with `opacity-90`.

## Dark Mode

Not in scope for Phase 1–4. The palette naturally supports a future dark mode by swapping surface/text mappings.

## Responsive Behavior

- **Desktop (≥1024px):** Sidebar visible, full layout
- **Tablet (768–1023px):** Sidebar collapsed to icons, expandable on hover
- **Mobile (<768px):** Sidebar as slide-out drawer, kanban scrolls horizontally
