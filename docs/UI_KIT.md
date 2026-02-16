# North UI Kit

Component library for the North GTD task management system. Built on Leptos + Tailwind CSS v4.

## Current State

The `north-ui` crate has 7 generic UI components: Icon, Dropdown, Popover, Checkbox, Modal, Autocomplete, Spinner, Markdown. Semantic atoms live in `crates/app/src/atoms/` — currently: **Text** (done, migrated across app).

**Key pain points (remaining):**
- Button styling repeated in 6+ patterns across files (primary, icon, text, danger, nav, small)
- Badge/tag display is ad-hoc `<span>` with inline color styles
- Form inputs have 3 incompatible patterns (bordered, transparent, date)
- Empty states, loading states, and form fields are all inline one-offs

## Design Philosophy

1. **Own the code** (shadcn/ui model) — each component is a file we control, not an external dependency
2. **Variant + Size pattern** — universal across mature kits (Ant Design, Chakra, shadcn). Apply consistently
3. **Composition over configuration** — prefer `<Modal><ModalHeader>...</ModalHeader></Modal>` over `<Modal title="..." footer="...">`
4. **CSS custom properties stay** — atoms use the existing `--accent`, `--text-primary`, etc.
5. **Atomic Design** (Brad Frost) — Atoms → Molecules → Organisms → Templates → Pages
6. **Constraints breed consistency** — limited variants defined once, inherited everywhere

## Atom Pattern (established with Text)

All atoms follow these conventions:

1. **Multi-dimension enum props** — each visual axis is a separate enum (e.g. `TextVariant`, `TextColor`)
2. **`fn classes(self) -> &'static str`** — each enum maps to Tailwind classes, component concatenates via `format!()`
3. **`Default` derive** — every enum has a sensible default; components work with zero props
4. **`class: &'static str`** escape hatch — for one-off Tailwind overrides
5. **Variant owns structural styling only** — size, weight, spacing, transform. Color is separate where it makes sense (Text) or baked in where variants are inherently colored (Button)
6. **`default_tag()`** — where applicable, variants determine the default HTML element; `tag: Option<Tag>` overrides
7. **Location:** `crates/app/src/atoms/` — imported via `use crate::atoms::{Component, ...};`
8. **Generic UI primitives** (Icon, Modal, Popover) stay in `north-ui` crate

---

## Component Catalog

### Priority & Status

| # | Component | Level | Impact | Effort | Status |
|---|-----------|-------|--------|--------|--------|
| 1 | [Text](#1-text) | Atom | High | Low | **Done** |
| 2 | [Button](#2-button) | Atom | Highest | Low | Next |
| 3 | [Badge](#3-badge) | Atom | Medium | Low | Planned |
| 4 | [Input](#4-input) | Atom | Medium | Low | Planned |
| 5 | [Textarea](#5-textarea) | Atom | Medium | Low | Planned |
| 6 | [Modal](#6-modal-enhanced) | Molecule | Medium | Medium | Exists (basic) |
| 7 | [FormField](#7-formfield) | Molecule | Medium | Low | Planned |
| 8 | [EmptyState](#8-emptystate) | Molecule | Low | Low | Planned |
| 9 | [Card](#9-card) | Molecule | Low | Low | Planned |
| 10 | [Tooltip](#10-tooltip) | Atom | Medium | Medium | Planned |
| 11 | [Separator](#11-separator) | Atom | Low | Low | Planned |
| 12 | [Skeleton](#12-skeleton) | Molecule | Low | Medium | Planned |
| 13 | [ActionBar](#13-actionbar) | Organism | Medium | Medium | Planned |
| 14 | [SidebarNav](#14-sidebarnav) | Organism | Low | Medium | Planned |

---

## Atoms

### 1. Text

**Status: Done** — `crates/app/src/atoms/text.rs`

**Role:** Unified typography with semantic variants based on Material Design 3 type scale (industry standard across MD3, Apple HIG, Fluent, Carbon, Atlassian, Polaris). Eliminates ~50+ scattered text class combinations.

**Reference implementations:**
- Material Design 3: Display/Headline/Title/Body/Label categories, each with Large/Medium/Small
- Ant Design: `Typography.Title` (levels 1-5), `Typography.Text`, `Typography.Paragraph`
- Fluent UI: Display/LargeTitle/Title/Subtitle/Body/Caption hierarchy
- Chakra: `Heading` + `Text` with size scale

**Architecture decision:** Variant owns only structural classes (size, weight, transform, spacing). Color is always a separate prop — no color baked into variants, no overlap, no Tailwind specificity conflicts.

**Props:**

```rust
#[component]
pub fn Text(
    /// Semantic text style (size + weight + transform). Default: BodyMd.
    #[prop(optional)]
    variant: TextVariant,
    /// HTML element override. Default: determined by variant.
    #[prop(optional)]
    tag: Option<TextTag>,
    /// Semantic text color. Default: Primary.
    #[prop(optional)]
    color: TextColor,
    /// Single-line truncation with ellipsis.
    #[prop(optional)]
    truncate: bool,
    /// Strikethrough (e.g. completed tasks).
    #[prop(optional)]
    line_through: bool,
    /// Additional Tailwind classes.
    #[prop(optional)]
    class: &'static str,
    children: Children,
) -> impl IntoView
```

**Variants (MD3-based semantic scale):**

| Variant | Tailwind classes | Default tag | Use case |
|---------|-----------------|-------------|----------|
| `HeadingLg` | `text-2xl font-semibold tracking-tight` | `<h1>` | Page titles |
| `HeadingMd` | `text-xl font-semibold` | `<h2>` | Section titles |
| `HeadingSm` | `text-lg font-semibold` | `<h3>` | Subsection titles |
| `TitleLg` | `text-base font-semibold` | `<span>` | Modal/card titles |
| `TitleMd` | `text-sm font-medium` | `<span>` | List headers, toolbar text |
| `TitleSm` | `text-sm font-medium` | `<span>` | Tab labels, small headers |
| `BodyLg` | `text-base` | `<p>` | Emphasized body text |
| `BodyMd` | `text-sm` | `<span>` | Default paragraph text |
| `BodySm` | `text-xs` | `<span>` | Secondary/compact text |
| `LabelLg` | `text-sm font-medium` | `<span>` | Button text, prominent labels |
| `LabelMd` | `text-xs font-medium uppercase tracking-wide` | `<span>` | Section headers, nav labels |
| `LabelSm` | `text-[11px] font-medium uppercase tracking-wide` | `<span>` | Helper text, timestamps |
| `CodeMd` | `text-sm font-mono` | `<code>` | Code blocks, filter DSL |
| `CodeSm` | `text-xs font-mono` | `<code>` | Compact code |

**Tags (HTML element):**

| Tag | Element | When to use |
|-----|---------|-------------|
| `H1` | `<h1>` | Page title (one per page) |
| `H2` | `<h2>` | Section title |
| `H3` | `<h3>` | Subsection title |
| `H4` | `<h4>` | Group header |
| `P` | `<p>` | Paragraph block |
| `Span` | `<span>` | Inline text (default) |
| `Label` | `<label>` | Form labels |
| `Code` | `<code>` | Code snippets |

**Colors:**

| Color | Class | Use case |
|-------|-------|----------|
| `Primary` | `text-text-primary` | Headings, main content |
| `Secondary` | `text-text-secondary` | Less prominent text |
| `Tertiary` | `text-text-tertiary` | Placeholders, metadata |
| `Accent` | `text-accent` | Links, highlights |
| `Danger` | `text-danger` | Errors, overdue dates |
| `OnAccent` | `text-on-accent` | Text on accent backgrounds |
| `Inherit` | _(none)_ | Inherits from parent |

**Implementation pattern:**

```rust
impl TextVariant {
    fn classes(self) -> &'static str {
        match self {
            Self::HeadingLg => "text-2xl font-semibold tracking-tight",
            Self::HeadingMd => "text-xl font-semibold",
            Self::HeadingSm => "text-lg font-semibold",
            Self::TitleLg   => "text-base font-semibold",
            Self::TitleMd   => "text-sm font-medium",
            Self::TitleSm   => "text-sm font-medium",
            Self::BodyLg    => "text-base",
            Self::BodyMd    => "text-sm",
            Self::BodySm    => "text-xs",
            Self::LabelLg   => "text-sm font-medium",
            Self::LabelMd   => "text-xs font-medium uppercase tracking-wide",
            Self::LabelSm   => "text-[11px] font-medium uppercase tracking-wide",
            Self::CodeMd    => "text-sm font-mono",
            Self::CodeSm    => "text-xs font-mono",
        }
    }

    fn default_tag(self) -> TextTag {
        match self {
            Self::HeadingLg                     => TextTag::H1,
            Self::HeadingMd                     => TextTag::H2,
            Self::HeadingSm                     => TextTag::H3,
            Self::CodeMd | Self::CodeSm         => TextTag::Code,
            Self::BodyLg                        => TextTag::P,
            _ => TextTag::Span, // TitleLg..Sm, BodyMd, BodySm, LabelLg..Sm
        }
    }
}

impl TextColor {
    fn classes(self) -> &'static str {
        match self {
            Self::Primary   => "text-text-primary",
            Self::Secondary => "text-text-secondary",
            Self::Tertiary  => "text-text-tertiary",
            Self::Accent    => "text-accent",
            Self::Danger    => "text-danger",
            Self::OnAccent  => "text-on-accent",
            Self::Inherit   => "",
        }
    }
}

// In the component — tag prop overrides variant default:
let effective_tag = tag.unwrap_or_else(|| variant.default_tag());

let classes = format!(
    "{} {}{}{} {}",
    variant.classes(),
    color.classes(),
    if truncate { " truncate" } else { "" },
    if line_through { " line-through" } else { "" },
    class,
);

let children = children();
match effective_tag {
    TextTag::H1    => view! { <h1 class=classes>{children}</h1> }.into_any(),
    TextTag::H2    => view! { <h2 class=classes>{children}</h2> }.into_any(),
    TextTag::H3    => view! { <h3 class=classes>{children}</h3> }.into_any(),
    TextTag::H4    => view! { <h4 class=classes>{children}</h4> }.into_any(),
    TextTag::P     => view! { <p class=classes>{children}</p> }.into_any(),
    TextTag::Span  => view! { <span class=classes>{children}</span> }.into_any(),
    TextTag::Label => view! { <label class=classes>{children}</label> }.into_any(),
    TextTag::Code  => view! { <code class=classes>{children}</code> }.into_any(),
}
```

**Usage examples:**

```rust
// Page title — HeadingLg defaults to <h1>, no tag needed
<Text variant=TextVariant::HeadingLg>"Inbox"</Text>

// Section title — HeadingMd defaults to <h2>
<Text variant=TextVariant::HeadingMd>"Saved Filters"</Text>

// Modal title — HeadingSm defaults to <h3>
<Text variant=TextVariant::HeadingSm>"Fix login bug"</Text>

// Task title in list — TitleMd defaults to <span>
<Text variant=TextVariant::TitleMd>{title}</Text>

// Completed task title
<Text variant=TextVariant::TitleMd color=TextColor::Tertiary line_through=true>{title}</Text>

// Sidebar section header
<Text variant=TextVariant::LabelMd color=TextColor::Secondary>"Projects"</Text>

// Form label — override default <span> to <label>
<Text variant=TextVariant::LabelMd tag=TextTag::Label color=TextColor::Tertiary>"Due date"</Text>

// Default body text — BodyMd + <span> + Primary, all defaults
<Text>"Some task description"</Text>

// Metadata / caption
<Text variant=TextVariant::BodySm color=TextColor::Tertiary>{due_date}</Text>

// Overdue date
<Text variant=TextVariant::BodySm color=TextColor::Danger>{due_date}</Text>

// Code in filter help — CodeMd defaults to <code>
<Text variant=TextVariant::CodeMd color=TextColor::Accent>"title =~ \"*report*\""</Text>

// Truncated project name in sidebar
<Text truncate=true>{project.title}</Text>

// HeadingMd but as <h4> instead of default <h2> — tag overrides
<Text variant=TextVariant::HeadingMd tag=TextTag::H4>"Subsection"</Text>
```

---

### 2. Button

**Status: Next**

**Role:** Replaces 6 ad-hoc button patterns. Every clickable action goes through this.

**Reference implementations:**
- shadcn/ui: `variant` (default/destructive/outline/secondary/ghost/link) + `size` (default/sm/lg/icon)
- Ant Design: `type` (primary/default/dashed/text/link) + `size` + `loading` + `icon` + `shape`
- Chakra: `variant` (solid/subtle/outline/ghost/plain) + `colorPalette` + `size`

**Current patterns in North (to be replaced):**

```rust
// Primary — save, create, submit
"px-3 py-1 text-sm bg-accent text-on-accent rounded hover:bg-accent-hover transition-colors"

// Secondary — cancel, less important
"px-3 py-1 text-sm text-text-secondary hover:text-text-primary transition-colors"

// Ghost — icon buttons in action bars
"p-1 rounded hover:bg-bg-input text-text-tertiary hover:text-text-secondary transition-colors"

// Danger — delete actions
"text-danger hover:text-danger-hover"

// Nav — sidebar navigation items
"flex items-center gap-2 px-3 py-2 rounded-lg text-sm text-text-primary hover:bg-bg-tertiary transition-colors"

// Small — inline, picker controls
"px-1.5 py-0.5 text-xs rounded"
```

**Implementation pattern** (follows Text atom convention):
- `ButtonVariant` enum with `fn classes(self) -> &'static str` — structural styling (bg, text color, hover states)
- `ButtonSize` enum with `fn classes(self) -> &'static str` — dimensions (padding, font size)
- Color is baked into variant (unlike Text where color is separate) — buttons have fixed color per variant
- All enums derive `Default, Clone, Copy, PartialEq`
- `class: &'static str` escape hatch for additional Tailwind

**Props:**

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `variant` | `ButtonVariant` | `Primary` | Visual style |
| `size` | `ButtonSize` | `Md` | Dimensions |
| `on_click` | `Callback<ev::MouseEvent>` | required | Click handler |
| `disabled` | `MaybeProp<bool>` | `false` | Disabled state |
| `loading` | `MaybeProp<bool>` | `false` | Shows spinner, disables interaction |
| `icon` | `Option<IconKind>` | `None` | Leading icon |
| `children` | `Option<Children>` | `None` | Label text (None = icon-only) |
| `class` | `&'static str` | `""` | Additional Tailwind classes |
| `node_ref` | `Option<NodeRef<html::Button>>` | `None` | Ref for focus management |

**Variants:**

| Variant | Classes | Use case |
|---------|---------|----------|
| `Primary` | `bg-accent text-on-accent hover:bg-accent-hover` | Save, create, submit |
| `Secondary` | `text-text-secondary hover:text-text-primary hover:bg-bg-tertiary` | Cancel, back, less important |
| `Ghost` | `text-text-tertiary hover:text-text-secondary hover:bg-bg-input` | Icon buttons, action bars |
| `Danger` | `text-danger hover:text-danger-hover hover:bg-bg-tertiary` | Delete, destructive actions |
| `Nav` | `text-text-primary hover:bg-bg-tertiary` | Sidebar navigation items |

**Sizes:**

| Size | Classes | Use case |
|------|---------|----------|
| `Xs` | `px-1.5 py-0.5 text-xs` | Inline controls, picker buttons |
| `Sm` | `px-2 py-1 text-sm` | Compact buttons |
| `Md` | `px-3 py-1.5 text-sm` | Default — most buttons |
| `Lg` | `px-4 py-2 text-base` | Prominent CTAs |
| `Icon` | `p-1` | Icon-only buttons (square) |

**Usage examples:**

```rust
// Primary save button
<Button on_click=on_save>"Save"</Button>

// Ghost icon button
<Button variant=ButtonVariant::Ghost size=ButtonSize::Icon icon=IconKind::Edit on_click=on_edit />

// Danger button with loading
<Button variant=ButtonVariant::Danger loading=deleting on_click=on_delete>"Delete"</Button>

// Nav button with icon
<Button variant=ButtonVariant::Nav size=ButtonSize::Md icon=IconKind::Inbox on_click=nav>
    "Inbox"
</Button>
```

---

### 3. Badge

**Role:** Tags, status indicators, project labels. Replaces ad-hoc `<span>` with inline styles.

**Reference implementations:**
- Ant Design: `Badge` (count overlay) + `Tag` (colored label, closable, checkable)
- Chakra: `Badge` (solid/subtle/outline) + `Tag` (with close button)
- shadcn/ui: `Badge` (variant: default/secondary/destructive/outline)

**Current patterns in North (to be replaced):**

```rust
// Tag display in task meta
<span class="inline-flex items-center gap-0.5 text-xs" style=format!("color: {}", tag.color)>
    {tag.name}
</span>

// Project indicator with color dot
<span class="w-2 h-2 rounded-full" style=format!("background-color: {}", color)></span>
<span class="text-sm">{project.title}</span>

// Subtask count
<span class="text-xs text-text-secondary">{count}</span>
```

**Props:**

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `variant` | `BadgeVariant` | `Subtle` | Visual style |
| `color` | `Option<String>` | `None` | Custom color (for tag/project colors) |
| `closable` | `bool` | `false` | Shows x button |
| `on_close` | `Option<Callback<()>>` | `None` | Close handler |
| `children` | `Children` | required | Label content |
| `class` | `&'static str` | `""` | Additional classes |

**Variants:**

| Variant | Description | Use case |
|---------|-------------|----------|
| `Solid` | Filled background, contrast text | Status indicators, counts |
| `Subtle` | Tinted background, colored text | Tags on tasks |
| `Outline` | Border only, colored text | Secondary tags |
| `Dot` | Small colored circle (no text layout) | Project color indicators |

**Usage examples:**

```rust
// Task tag
<Badge color=tag.color.clone()>{tag.name}</Badge>

// Closable tag in picker
<Badge color=tag.color.clone() closable=true on_close=remove_tag>{tag.name}</Badge>

// Project dot
<Badge variant=BadgeVariant::Dot color=project.color.clone() />

// Subtask count
<Badge variant=BadgeVariant::Solid>{format!("{}/{}", completed, total)}</Badge>
```

---

### 4. Input

**Role:** Unified text input. Replaces 3 inconsistent patterns (bordered, transparent, date).

**Reference implementations:**
- Ant Design: `Input` with `variant` (outlined/filled/borderless), prefix/suffix slots
- Chakra: `Input` + `InputGroup` + `InputAddon`
- shadcn/ui: Thin wrapper with Tailwind styling

**Current patterns in North (to be replaced):**

```rust
// Bordered — modals, settings forms
"bg-bg-input border border-border rounded px-2 py-1.5 text-sm text-text-primary
 placeholder:text-text-tertiary focus:outline-none focus:border-accent"

// Transparent — inline editing, task title/body
"bg-transparent outline-none no-focus-ring placeholder:text-text-secondary text-text-primary"

// Ghost — inline rename (sidebar project edit)
"bg-transparent border-b border-transparent focus:border-accent"
```

**Props:**

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `variant` | `InputVariant` | `Bordered` | Visual style |
| `value` | `Signal<String>` | required | Controlled value |
| `on_input` | `Callback<String>` | required | Input handler |
| `placeholder` | `&'static str` | `""` | Placeholder text |
| `size` | `InputSize` | `Md` | Height/font sizing |
| `input_type` | `&'static str` | `"text"` | HTML input type |
| `node_ref` | `Option<NodeRef<html::Input>>` | `None` | For focus management |
| `on_keydown` | `Option<Callback<ev::KeyboardEvent>>` | `None` | Key handler |
| `class` | `&'static str` | `""` | Additional classes |

**Variants:**

| Variant | Classes | Use case |
|---------|---------|----------|
| `Bordered` | `bg-bg-input border border-border rounded focus:border-accent` | Forms, modals, settings |
| `Transparent` | `bg-transparent outline-none no-focus-ring` | Inline editing, task forms |
| `Ghost` | `bg-transparent border-b border-transparent focus:border-accent` | Inline rename |

**Sizes:**

| Size | Classes |
|------|---------|
| `Sm` | `px-1.5 py-1 text-xs` |
| `Md` | `px-2 py-1.5 text-sm` |
| `Lg` | `px-3 py-2 text-base` |

---

### 5. Textarea

**Role:** Multi-line text input. Same variant system as Input.

**Props:** Same as Input, plus:

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `rows` | `Option<u32>` | `None` | Initial row count |
| `auto_resize` | `bool` | `false` | Grow with content |

---

### 6. Tooltip

**Role:** Hover info for icon buttons, truncated text. Currently missing.

**Reference implementations:**
- Radix: Tooltip with Root, Trigger, Content, Arrow — delay, collision avoidance
- Ant Design: `title`, `placement` (12 positions), `trigger` (hover/focus/click)

**Props:**

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `content` | `&'static str` | required | Tooltip text |
| `side` | `TooltipSide` | `Top` | Placement |
| `delay_ms` | `u32` | `300` | Show delay |
| `children` | `Children` | required | Trigger element |

**Sides:** `Top`, `Bottom`, `Left`, `Right`

**Usage examples:**

```rust
<Tooltip content="Edit task">
    <Button variant=ButtonVariant::Ghost size=ButtonSize::Icon icon=IconKind::Edit on_click=on_edit />
</Tooltip>
```

---

### 7. Separator

**Role:** Horizontal/vertical dividers. Replaces `border-t border-border` repetitions.

**Props:**

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `orientation` | `Orientation` | `Horizontal` | Direction |
| `class` | `&'static str` | `""` | Additional classes |

Renders: `<div role="separator" class="border-t border-border">` (horizontal) or `border-l h-full` (vertical).

---

## Molecules

### 6. Modal (enhanced)

**Role:** Replace current minimal Modal with structured compound component.

**Reference implementations:**
- shadcn/ui: Dialog + DialogHeader + DialogTitle + DialogDescription + DialogFooter + DialogClose
- Ant Design: Modal with `title`, `footer`, `closable`, `centered`
- Chakra: Dialog with Root, Header, Body, Footer, CloseTrigger

**Current Modal (to be enhanced):**
- Controlled open state, size prop (md/lg/xl), backdrop, Esc/backdrop close delegated to parent

**Enhanced Props:**

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `open` | `ReadSignal<bool>` | required | Visibility |
| `on_close` | `Callback<()>` | required | Close handler |
| `size` | `ModalSize` | `Md` | Width constraint |
| `title` | `Option<View>` | `None` | Header content |
| `footer` | `Option<View>` | `None` | Footer with action buttons |
| `closable` | `bool` | `true` | Show x button in header |
| `children` | `Children` | required | Body content |

**Sizes:** `Sm` (max-w-sm), `Md` (max-w-md), `Lg` (max-w-3xl), `Xl` (max-w-5xl), `Full` (max-w-6xl)

**Built-in behavior:** Esc key close, backdrop click close, focus trap, body scroll lock.

**Usage example:**

```rust
<Modal
    open=show_modal.read()
    on_close=Callback::new(move |_| set_show_modal.set(false))
    size=ModalSize::Lg
    title=view! { <Text variant=TextVariant::Title>"Edit Task"</Text> }.into()
    footer=view! {
        <Button variant=ButtonVariant::Secondary on_click=on_cancel>"Cancel"</Button>
        <Button on_click=on_save>"Save"</Button>
    }.into()
>
    // body content
</Modal>
```

---

### 7. FormField

**Role:** Label + input + error/help text. Wraps any input atom.

**Reference implementations:**
- Ant Design: `Form.Item` with `label`, `name`, `rules`, `validateStatus`, `help`
- Chakra: `Field` with Root, Label, Input, HelpText, ErrorText
- shadcn/ui: `FormField` + `FormItem` + `FormLabel` + `FormMessage`

**Props:**

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `label` | `&'static str` | required | Field label text |
| `error` | `Option<String>` | `None` | Validation error message |
| `help` | `Option<&'static str>` | `None` | Help text below input |
| `required` | `bool` | `false` | Shows required indicator |
| `children` | `Children` | required | Input component |

**Renders:**
```
Label (Text variant=Label)
├── Children (Input/Textarea/Select)
├── Error text (Text variant=Caption color=Danger)  — if error
└── Help text (Text variant=Caption color=Tertiary)  — if help
```

**Usage example:**

```rust
<FormField label="Project title" error=title_error.get()>
    <Input value=title on_input=set_title placeholder="Enter title..." />
</FormField>
```

---

### 8. EmptyState

**Role:** Placeholder for empty lists. Replaces ad-hoc "No items" text.

**Reference implementations:**
- Ant Design: `Empty` (image + description + optional CTA)
- Chakra: `EmptyState` (icon + title + description + action)

**Props:**

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `icon` | `Option<IconKind>` | `None` | Illustrative icon |
| `title` | `&'static str` | required | Main message |
| `description` | `Option<&'static str>` | `None` | Secondary message |
| `action` | `Option<Children>` | `None` | CTA button |

**Usage example:**

```rust
<EmptyState
    icon=IconKind::Inbox
    title="No tasks"
    description="Create a task to get started"
    action=view! {
        <Button icon=IconKind::Plus on_click=on_create>"Add task"</Button>
    }.into()
/>
```

---

### 9. Card

**Role:** Container for grouped content. Replaces `border border-border rounded-xl p-3 shadow-sm`.

**Props:**

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `variant` | `CardVariant` | `Bordered` | Visual style |
| `padding` | `CardPadding` | `Md` | Internal padding |
| `children` | `Children` | required | Content |
| `class` | `&'static str` | `""` | Additional classes |

**Variants:**

| Variant | Classes |
|---------|---------|
| `Bordered` | `border border-border rounded-xl` |
| `Elevated` | `border border-border rounded-xl shadow-sm` |
| `Ghost` | `rounded-xl` (no border) |

**Padding:** `None`, `Sm` (p-2), `Md` (p-3), `Lg` (p-4)

---

### 10. Skeleton

**Role:** Loading placeholders for lists. Supplement to existing Spinner.

**Reference implementations:**
- Ant Design: `Skeleton` with avatar + title + paragraph shapes
- Chakra: `Skeleton` with pulse/shine animation, wraps children

**Props:**

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `variant` | `SkeletonVariant` | `Line` | Shape |
| `width` | `Option<&'static str>` | `None` | CSS width |
| `height` | `Option<&'static str>` | `None` | CSS height |
| `count` | `u32` | `1` | Repeat for lists |

**Variants:** `Line` (full-width bar), `Circle` (avatar), `Rect` (card placeholder)

---

## Organisms

### 11. ActionBar

**Role:** The icon button row that appears on task list items on hover. Currently ~40 lines of repeated markup.

**Props:**

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `actions` | `Vec<ActionBarItem>` | required | Action buttons |
| `visible` | `MaybeProp<bool>` | `true` | Visibility control |

**ActionBarItem:**

| Field | Type | Description |
|-------|------|-------------|
| `icon` | `IconKind` | Button icon |
| `tooltip` | `&'static str` | Hover text |
| `on_click` | `Callback<ev::MouseEvent>` | Click handler |
| `danger` | `bool` | Danger styling |

---

### 12. SidebarNav

**Role:** Navigation group in sidebar. Replaces repeated nav item markup.

**Props:**

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `items` | `Vec<NavItem>` | required | Navigation entries |

**NavItem:**

| Field | Type | Description |
|-------|------|-------------|
| `icon` | `IconKind` | Nav icon |
| `label` | `String` | Display text |
| `href` | `String` | Route path |
| `badge` | `Option<u32>` | Count badge |

---

## Existing Components (Keep As-Is)

These components in `north-ui` are adequate and don't need redesign:

| Component | Notes |
|-----------|-------|
| `Icon` / `IconKind` | Solid SVG icon system, works well |
| `Checkbox` | Signal-based, accessible |
| `Popover` | Intelligent positioning, click-outside close |
| `AutocompleteDropdown` | Suggestion list with keyboard navigation |
| `MarkdownView` | pulldown-cmark + ammonia sanitization |
| `Spinner` | Simple loading indicator |

The existing `DropdownMenu` and `Modal` will be enhanced (not replaced) as described above.

---

## Implementation Plan

Atoms live in `crates/app/src/atoms/` and are exported from `crates/app/src/atoms/mod.rs`. Generic UI primitives (Icon, Modal, Popover, etc.) stay in `crates/ui/`. After each atom is built, existing usage across the app crate is migrated.

**Pattern:** Each atom uses multi-dimension enum props. Each enum has `fn classes(self) -> &'static str`. The component concatenates dimensions via `format!()`. See `text.rs` as the reference implementation.

| # | Component | Status |
|---|-----------|--------|
| 1 | Text | **Done** |
| 2 | Button | Next |
| 3 | Badge | Planned |
| 4 | Input | Planned |
| 5 | Textarea | Planned |
| 6 | Modal (enhance) | Planned |
| 7 | FormField | Planned |
| 8 | EmptyState | Planned |
| 9 | Card | Planned |
| 10 | Tooltip | Planned |
| 11 | Separator | Planned |
| 12 | Skeleton | Planned |
| 13 | ActionBar | Planned |
| 14 | SidebarNav | Planned |
