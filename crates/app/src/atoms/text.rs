use leptos::prelude::*;

/// Semantic text variant based on Material Design 3 type scale.
/// Controls size, weight, transform, and spacing. Never includes color.
#[derive(Default, Clone, Copy, PartialEq)]
pub enum TextVariant {
    // Heading — page and section titles
    /// text-2xl font-semibold tracking-tight → default <h1>
    HeadingLg,
    /// text-xl font-semibold → default <h2>
    HeadingMd,
    /// text-lg font-semibold → default <h3>
    HeadingSm,

    // Title — card, dialog, and component titles
    /// text-base font-semibold → default <span>
    TitleLg,
    /// text-sm font-medium → default <span>
    TitleMd,
    /// text-sm font-medium → default <span>
    TitleSm,

    // Body — paragraph and running text
    /// text-base → default <p>
    BodyLg,
    /// text-sm → default <span>
    #[default]
    BodyMd,
    /// text-xs → default <span>
    BodySm,

    // Label — form labels, metadata, badges, buttons
    /// text-sm font-medium → default <span>
    LabelLg,
    /// text-xs font-medium uppercase tracking-wide → default <span>
    LabelMd,
    /// text-[11px] font-medium uppercase tracking-wide → default <span>
    LabelSm,

    // Code — monospace
    /// text-sm font-mono → default <code>
    CodeMd,
    /// text-xs font-mono → default <code>
    CodeSm,
}

impl TextVariant {
    fn classes(self) -> &'static str {
        match self {
            Self::HeadingLg => "text-2xl font-semibold tracking-tight",
            Self::HeadingMd => "text-xl font-semibold",
            Self::HeadingSm => "text-lg font-semibold",
            Self::TitleLg => "text-base font-semibold",
            Self::TitleMd => "text-sm font-medium",
            Self::TitleSm => "text-sm font-medium",
            Self::BodyLg => "text-base",
            Self::BodyMd => "text-sm",
            Self::BodySm => "text-xs",
            Self::LabelLg => "text-sm font-medium",
            Self::LabelMd => "text-xs font-medium uppercase tracking-wide",
            Self::LabelSm => "text-[11px] font-medium uppercase tracking-wide",
            Self::CodeMd => "text-sm font-mono",
            Self::CodeSm => "text-xs font-mono",
        }
    }

    fn default_tag(self) -> TextTag {
        match self {
            Self::HeadingLg => TextTag::H1,
            Self::HeadingMd => TextTag::H2,
            Self::HeadingSm => TextTag::H3,
            Self::BodyLg => TextTag::P,
            Self::CodeMd | Self::CodeSm => TextTag::Code,
            _ => TextTag::Span,
        }
    }
}

/// Semantic text color. Always a separate axis from variant.
#[derive(Default, Clone, Copy, PartialEq)]
pub enum TextColor {
    #[default]
    Primary,
    Secondary,
    Tertiary,
    Accent,
    Danger,
    OnAccent,
    Inherit,
}

impl TextColor {
    fn classes(self) -> &'static str {
        match self {
            Self::Primary => "text-text-primary",
            Self::Secondary => "text-text-secondary",
            Self::Tertiary => "text-text-tertiary",
            Self::Accent => "text-accent",
            Self::Danger => "text-danger",
            Self::OnAccent => "text-on-accent",
            Self::Inherit => "",
        }
    }
}

/// HTML element to render. Defaults are determined by variant.
#[derive(Default, Clone, Copy, PartialEq)]
pub enum TextTag {
    H1,
    H2,
    H3,
    H4,
    P,
    #[default]
    Span,
    Label,
    Code,
}

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
) -> impl IntoView {
    let effective_tag = tag.unwrap_or_else(|| variant.default_tag());

    let classes = format!(
        "{} {}{}{}{}",
        variant.classes(),
        color.classes(),
        if truncate { " truncate" } else { "" },
        if line_through { " line-through" } else { "" },
        if class.is_empty() {
            String::new()
        } else {
            format!(" {class}")
        },
    );

    let children = children();
    match effective_tag {
        TextTag::H1 => view! { <h1 class=classes>{children}</h1> }.into_any(),
        TextTag::H2 => view! { <h2 class=classes>{children}</h2> }.into_any(),
        TextTag::H3 => view! { <h3 class=classes>{children}</h3> }.into_any(),
        TextTag::H4 => view! { <h4 class=classes>{children}</h4> }.into_any(),
        TextTag::P => view! { <p class=classes>{children}</p> }.into_any(),
        TextTag::Span => view! { <span class=classes>{children}</span> }.into_any(),
        TextTag::Label => view! { <label class=classes>{children}</label> }.into_any(),
        TextTag::Code => view! { <code class=classes>{children}</code> }.into_any(),
    }
}
