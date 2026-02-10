use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ParsedText {
    pub cleaned: String,
    pub tags: Vec<String>,
    pub project: Option<String>,
}

pub fn parse_tokens(text: &str) -> ParsedText {
    let mut tags = Vec::new();
    let mut project: Option<String> = None;
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
                        let lower = token.to_lowercase();
                        if !tags.contains(&lower) {
                            tags.push(lower);
                        }
                    }
                    '@' => {
                        project = Some(token);
                    }
                    _ => unreachable!(),
                }
            } else {
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

    ParsedText {
        cleaned,
        tags,
        project,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_tokens() {
        let result = parse_tokens("Buy groceries");
        assert_eq!(result.cleaned, "Buy groceries");
        assert!(result.tags.is_empty());
        assert!(result.project.is_none());
    }

    #[test]
    fn test_single_tag() {
        let result = parse_tokens("Buy groceries #shopping");
        assert_eq!(result.cleaned, "Buy groceries");
        assert_eq!(result.tags, vec!["shopping"]);
        assert!(result.project.is_none());
    }

    #[test]
    fn test_single_project() {
        let result = parse_tokens("Buy groceries @Personal");
        assert_eq!(result.cleaned, "Buy groceries");
        assert!(result.tags.is_empty());
        assert_eq!(result.project, Some("Personal".to_string()));
    }

    #[test]
    fn test_tag_and_project() {
        let result = parse_tokens("Buy groceries #shopping @Personal");
        assert_eq!(result.cleaned, "Buy groceries");
        assert_eq!(result.tags, vec!["shopping"]);
        assert_eq!(result.project, Some("Personal".to_string()));
    }

    #[test]
    fn test_multiple_tags() {
        let result = parse_tokens("#urgent Buy groceries #shopping #food");
        assert_eq!(result.cleaned, "Buy groceries");
        assert_eq!(result.tags, vec!["urgent", "shopping", "food"]);
    }

    #[test]
    fn test_duplicate_tags() {
        let result = parse_tokens("Test #foo #FOO #foo");
        assert_eq!(result.cleaned, "Test");
        assert_eq!(result.tags, vec!["foo"]);
    }

    #[test]
    fn test_last_project_wins() {
        let result = parse_tokens("Task @First @Second");
        assert_eq!(result.cleaned, "Task");
        assert_eq!(result.project, Some("Second".to_string()));
    }

    #[test]
    fn test_mid_word_hash_not_parsed() {
        let result = parse_tokens("C#sharp is great");
        assert_eq!(result.cleaned, "C#sharp is great");
        assert!(result.tags.is_empty());
    }

    #[test]
    fn test_hash_with_hyphen_underscore() {
        let result = parse_tokens("Task #my-tag #my_tag");
        assert_eq!(result.cleaned, "Task");
        assert_eq!(result.tags, vec!["my-tag", "my_tag"]);
    }

    #[test]
    fn test_empty_hash() {
        let result = parse_tokens("Test # nothing");
        assert_eq!(result.cleaned, "Test # nothing");
        assert!(result.tags.is_empty());
    }

    #[test]
    fn test_only_tokens() {
        let result = parse_tokens("#shopping @Personal");
        assert_eq!(result.cleaned, "");
        assert_eq!(result.tags, vec!["shopping"]);
        assert_eq!(result.project, Some("Personal".to_string()));
    }
}
