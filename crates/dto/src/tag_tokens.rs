/// Client-side tag token extractor.
///
/// Extracts `#tag` tokens from text using the same word-boundary and character
/// rules as the server-side `core::parse_tokens`. Returns the cleaned text
/// (with `#tag` tokens removed) and a deduplicated, lowercased list of tag names.
///
/// `@project` tokens and all other text are preserved in the cleaned output.
pub fn extract_tag_tokens(text: &str) -> (String, Vec<String>) {
    let mut tags = Vec::new();
    let mut cleaned_parts = Vec::new();
    let mut chars = text.chars().peekable();
    let mut current_word = String::new();
    let mut at_word_start = true;

    while let Some(ch) = chars.next() {
        if ch.is_whitespace() {
            if !current_word.is_empty() {
                cleaned_parts.push(current_word.clone());
                current_word.clear();
            }
            at_word_start = true;
            continue;
        }

        if at_word_start && ch == '#' {
            let mut token = String::new();
            while let Some(&next) = chars.peek() {
                if next.is_alphanumeric()
                    || next == '_'
                    || next == '-'
                    || next == ':'
                    || next == '.'
                {
                    token.push(chars.next().unwrap());
                } else {
                    break;
                }
            }

            if !token.is_empty() {
                let lower = token.to_lowercase();
                if !tags.contains(&lower) {
                    tags.push(lower);
                }
            } else {
                // Bare `#` with no valid token chars — keep as text
                current_word.push(ch);
            }
            at_word_start = false;
        } else {
            current_word.push(ch);
            at_word_start = false;
        }
    }

    if !current_word.is_empty() {
        cleaned_parts.push(current_word);
    }

    let cleaned = cleaned_parts.join(" ");
    (cleaned, tags)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_tags() {
        let (cleaned, tags) = extract_tag_tokens("Buy groceries");
        assert_eq!(cleaned, "Buy groceries");
        assert!(tags.is_empty());
    }

    #[test]
    fn single_tag() {
        let (cleaned, tags) = extract_tag_tokens("Buy groceries #shopping");
        assert_eq!(cleaned, "Buy groceries");
        assert_eq!(tags, vec!["shopping"]);
    }

    #[test]
    fn multiple_tags() {
        let (cleaned, tags) = extract_tag_tokens("Buy milk #shopping #urgent");
        assert_eq!(cleaned, "Buy milk");
        assert_eq!(tags, vec!["shopping", "urgent"]);
    }

    #[test]
    fn duplicate_tags_deduplicated() {
        let (cleaned, tags) = extract_tag_tokens("Test #foo #FOO #foo");
        assert_eq!(cleaned, "Test");
        assert_eq!(tags, vec!["foo"]);
    }

    #[test]
    fn tag_at_start() {
        let (cleaned, tags) = extract_tag_tokens("#urgent Buy milk");
        assert_eq!(cleaned, "Buy milk");
        assert_eq!(tags, vec!["urgent"]);
    }

    #[test]
    fn mid_word_hash_not_parsed() {
        let (cleaned, tags) = extract_tag_tokens("C#sharp is great");
        assert_eq!(cleaned, "C#sharp is great");
        assert!(tags.is_empty());
    }

    #[test]
    fn preserves_at_project_tokens() {
        let (cleaned, tags) = extract_tag_tokens("Task #tag @Personal");
        assert_eq!(cleaned, "Task @Personal");
        assert_eq!(tags, vec!["tag"]);
    }

    #[test]
    fn tag_with_special_chars() {
        let (cleaned, tags) = extract_tag_tokens("Task #my-tag:v2.0_final");
        assert_eq!(cleaned, "Task");
        assert_eq!(tags, vec!["my-tag:v2.0_final"]);
    }

    #[test]
    fn bare_hash_preserved() {
        let (cleaned, tags) = extract_tag_tokens("Test # nothing");
        assert_eq!(cleaned, "Test # nothing");
        assert!(tags.is_empty());
    }

    #[test]
    fn only_tags() {
        let (cleaned, tags) = extract_tag_tokens("#shopping #urgent");
        assert_eq!(cleaned, "");
        assert_eq!(tags, vec!["shopping", "urgent"]);
    }

    #[test]
    fn tags_lowercased() {
        let (cleaned, tags) = extract_tag_tokens("Task #Shopping #URGENT");
        assert_eq!(cleaned, "Task");
        assert_eq!(tags, vec!["shopping", "urgent"]);
    }
}
