use leptos::prelude::*;
use north_stores::AppStore;
use north_ui::MarkdownView;

/// Renders markdown body text with enriched `#tag` and `@project` tokens.
/// - `#tag` → `[#tag](/filters/new?q=tags%3D%22tagname%22)` (link to filter page)
/// - `@project` → `[@project](/project/{id})` (link to project page)
/// Tokens inside code blocks and existing markdown links are left unchanged.
#[component]
pub fn EnrichedMarkdownView(content: String) -> impl IntoView {
    let app_store = use_context::<AppStore>();
    let enriched = enrich_body(&content, app_store);

    view! {
        <MarkdownView content=enriched/>
    }
}

fn enrich_body(text: &str, app_store: Option<AppStore>) -> String {
    let mut result = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    let mut in_code_block = false;
    let mut in_inline_code = false;
    let mut in_link_text = false;
    let mut in_link_target = false;
    let mut at_word_start = true;

    while let Some(ch) = chars.next() {
        // Track code fences
        if ch == '`' {
            if chars.peek() == Some(&'`') {
                // Potential code fence
                let next1 = chars.next(); // second `
                if chars.peek() == Some(&'`') {
                    let _ = chars.next(); // third `
                    in_code_block = !in_code_block;
                    result.push_str("```");
                } else {
                    // Two backticks - push them
                    result.push(ch);
                    if let Some(c) = next1 {
                        result.push(c);
                    }
                }
            } else {
                in_inline_code = !in_inline_code;
                result.push(ch);
            }
            at_word_start = false;
            continue;
        }

        if in_code_block || in_inline_code {
            result.push(ch);
            at_word_start = ch.is_whitespace() || ch == '\n';
            continue;
        }

        // Track markdown links
        if ch == '[' {
            in_link_text = true;
            result.push(ch);
            at_word_start = false;
            continue;
        }
        if ch == ']' && in_link_text {
            in_link_text = false;
            result.push(ch);
            if chars.peek() == Some(&'(') {
                in_link_target = true;
            }
            at_word_start = false;
            continue;
        }
        if ch == '(' && in_link_target {
            result.push(ch);
            at_word_start = false;
            continue;
        }
        if ch == ')' && in_link_target {
            in_link_target = false;
            result.push(ch);
            at_word_start = false;
            continue;
        }
        if in_link_text || in_link_target {
            result.push(ch);
            at_word_start = false;
            continue;
        }

        // Handle tokens at word boundaries
        if at_word_start && (ch == '#' || ch == '@') {
            let trigger = ch;
            let mut token = String::new();
            while let Some(&next) = chars.peek() {
                if next.is_alphanumeric() || next == '_' || next == '-' {
                    token.push(chars.next().unwrap());
                } else {
                    break;
                }
            }

            if !token.is_empty() {
                match trigger {
                    '#' => {
                        let query = format!("tags=\"{}\"", token.to_lowercase());
                        let encoded = urlencoding::encode(&query);
                        result.push_str(&format!("[#{token}](/filters/new?q={encoded})"));
                    }
                    '@' => {
                        let project_id = app_store.and_then(|s| {
                            s.projects
                                .get()
                                .iter()
                                .find(|p| p.title.eq_ignore_ascii_case(&token))
                                .map(|p| p.id)
                        });
                        if let Some(pid) = project_id {
                            result.push_str(&format!("[@{token}](/project/{pid})"));
                        } else {
                            result.push('@');
                            result.push_str(&token);
                        }
                    }
                    _ => unreachable!(),
                }
                at_word_start = false;
                continue;
            } else {
                result.push(ch);
                at_word_start = false;
                continue;
            }
        }

        result.push(ch);
        at_word_start = ch.is_whitespace() || ch == '\n';
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plain_text_unchanged() {
        let result = enrich_body("Hello world", None);
        assert_eq!(result, "Hello world");
    }

    #[test]
    fn test_tag_converted() {
        let result = enrich_body("Check #urgent task", None);
        assert!(result.contains("[#urgent]"));
        assert!(result.contains("/filters/new?q="));
    }

    #[test]
    fn test_code_block_preserved() {
        let result = enrich_body("```\n#tag\n```", None);
        assert!(!result.contains("[#tag]"));
    }

    #[test]
    fn test_inline_code_preserved() {
        let result = enrich_body("Use `#tag` syntax", None);
        assert!(!result.contains("[#tag]"));
    }

    #[test]
    fn test_existing_link_preserved() {
        let result = enrich_body("[link](https://example.com)", None);
        assert_eq!(result, "[link](https://example.com)");
    }
}
