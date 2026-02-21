use leptos::prelude::*;

use crate::atoms::{TextColor, TextVariant};

/// Renders a task title with `[text](url)` patterns as clickable links.
/// All other text is rendered as plain spans. Links get `stop_propagation`
/// to prevent opening the task detail modal when clicking a link.
#[component]
pub fn RichTitle(
    title: String,
    #[prop(optional)] variant: TextVariant,
    #[prop(optional)] color: TextColor,
    #[prop(optional)] line_through: bool,
    #[prop(optional)] class: &'static str,
) -> impl IntoView {
    let parts = parse_title(&title);

    let base_classes = format!(
        "{} {}{}{}",
        variant.classes(),
        color.classes(),
        if line_through { " line-through" } else { "" },
        if class.is_empty() {
            String::new()
        } else {
            format!(" {class}")
        },
    );

    let has_links = parts.iter().any(|p| matches!(p, TitlePart::Link { .. }));

    if !has_links {
        return view! {
            <span class=base_classes>{title}</span>
        }
        .into_any();
    }

    view! {
        <span class=base_classes>
            {parts
                .into_iter()
                .map(|part| match part {
                    TitlePart::Text(t) => view! { <span>{t}</span> }.into_any(),
                    TitlePart::Link { text, url } => {
                        view! {
                            <a
                                href=url
                                target="_blank"
                                rel="noopener noreferrer"
                                class="text-accent hover:text-accent-hover \
                                       underline decoration-accent/40 \
                                       hover:decoration-accent"
                                on:click=move |ev| ev.stop_propagation()
                            >
                                {text}
                            </a>
                        }
                            .into_any()
                    }
                })
                .collect::<Vec<_>>()}
        </span>
    }
    .into_any()
}

enum TitlePart {
    Text(String),
    Link { text: String, url: String },
}

/// Manual parser for `[text](url)` patterns. No regex needed.
fn parse_title(input: &str) -> Vec<TitlePart> {
    let mut parts = Vec::new();
    let mut rest = input;

    while !rest.is_empty() {
        if let Some(bracket_start) = rest.find('[') {
            // Try to parse a markdown link starting at this position
            if let Some((link_text, url, consumed)) = try_parse_link(&rest[bracket_start..]) {
                // Push text before the link
                if bracket_start > 0 {
                    parts.push(TitlePart::Text(rest[..bracket_start].to_string()));
                }
                parts.push(TitlePart::Link {
                    text: link_text,
                    url,
                });
                rest = &rest[bracket_start + consumed..];
                continue;
            }
            // Not a valid link — push up to and including the bracket as text
            parts.push(TitlePart::Text(rest[..bracket_start + 1].to_string()));
            rest = &rest[bracket_start + 1..];
        } else {
            parts.push(TitlePart::Text(rest.to_string()));
            break;
        }
    }

    parts
}

/// Try to parse `[text](url)` at the start of the input.
/// Returns (link_text, url, total_chars_consumed) or None.
fn try_parse_link(input: &str) -> Option<(String, String, usize)> {
    let bytes = input.as_bytes();
    if bytes.first() != Some(&b'[') {
        return None;
    }

    // Find closing bracket
    let close_bracket = input[1..].find(']')? + 1;
    let link_text = &input[1..close_bracket];

    // Must be followed by (
    let after_bracket = &input[close_bracket + 1..];
    if !after_bracket.starts_with('(') {
        return None;
    }

    // Find closing paren
    let close_paren = after_bracket[1..].find(')')? + 1;
    let url = &after_bracket[1..close_paren];

    // Basic URL validation
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return None;
    }

    let total = close_bracket + 1 + close_paren + 1;
    Some((link_text.to_string(), url.to_string(), total))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plain_text() {
        let parts = parse_title("Buy groceries");
        assert_eq!(parts.len(), 1);
        assert!(matches!(&parts[0], TitlePart::Text(t) if t == "Buy groceries"));
    }

    #[test]
    fn test_single_link() {
        let parts = parse_title("[Google](https://google.com)");
        assert_eq!(parts.len(), 1);
        assert!(
            matches!(&parts[0], TitlePart::Link { text, url } if text == "Google" && url == "https://google.com")
        );
    }

    #[test]
    fn test_text_with_link() {
        let parts = parse_title("Check [Google](https://google.com) for info");
        assert_eq!(parts.len(), 3);
        assert!(matches!(&parts[0], TitlePart::Text(t) if t == "Check "));
        assert!(matches!(&parts[1], TitlePart::Link { text, .. } if text == "Google"));
        assert!(matches!(&parts[2], TitlePart::Text(t) if t == " for info"));
    }

    #[test]
    fn test_incomplete_bracket() {
        let parts = parse_title("array[0] is great");
        assert_eq!(parts.len(), 2);
        // The [ triggers a parse attempt, but ](url) doesn't follow
        // so it falls back to text
    }
}
