use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// Splits a tag name like `"priority:high"` into `("priority", "high")`.
/// Returns `None` for simple tags, empty key, or empty value.
pub fn parse_kv(name: &str) -> Option<(&str, &str)> {
    let (key, value) = name.split_once(':')?;
    if key.is_empty() || value.is_empty() {
        return None;
    }
    Some((key, value))
}

/// Groups tag names into simple tags and k:v groups.
/// K:V groups store full tag names (e.g. `"priority:high"`), keyed by prefix.
pub fn group_tag_names(names: &[String]) -> (Vec<String>, BTreeMap<String, Vec<String>>) {
    let mut simple = Vec::new();
    let mut kv: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for name in names {
        if let Some((key, _)) = parse_kv(name) {
            kv.entry(key.to_string()).or_default().push(name.clone());
        } else {
            simple.push(name.clone());
        }
    }
    (simple, kv)
}

/// Faceted filter: simple tags use AND, k:v groups use OR within same key, AND between keys.
/// Empty filter matches all tasks.
pub fn matches_faceted_filter(task_tags: &[&str], active: &[String]) -> bool {
    if active.is_empty() {
        return true;
    }
    let (simple, kv_groups) = group_tag_names(active);
    for name in &simple {
        if !task_tags.contains(&name.as_str()) {
            return false;
        }
    }
    for values in kv_groups.values() {
        if !values.iter().any(|v| task_tags.contains(&v.as_str())) {
            return false;
        }
    }
    true
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TagInfo {
    pub name: String,
    pub color: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Tag {
    pub id: i64,
    pub user_id: i64,
    pub name: String,
    pub color: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateTag {
    pub name: String,
    pub color: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateTag {
    pub name: Option<String>,
    pub color: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    // ── parse_kv() ──────────────────────────────────────────────

    #[test]
    fn parse_kv_splits_key_value() {
        assert_eq!(parse_kv("priority:high"), Some(("priority", "high")));
    }

    #[test]
    fn parse_kv_returns_none_for_simple_tag() {
        assert_eq!(parse_kv("urgent"), None);
    }

    #[test]
    fn parse_kv_returns_none_for_empty_key() {
        assert_eq!(parse_kv(":value"), None);
    }

    #[test]
    fn parse_kv_returns_none_for_empty_value() {
        assert_eq!(parse_kv("key:"), None);
    }

    #[test]
    fn parse_kv_splits_on_first_colon_only() {
        assert_eq!(parse_kv("a:b:c"), Some(("a", "b:c")));
    }

    // ── group_tag_names() ───────────────────────────────────────

    #[test]
    fn group_tag_names_mixed_input() {
        let names = vec![
            "priority:high".to_string(),
            "priority:low".to_string(),
            "urgent".to_string(),
            "type:bug".to_string(),
        ];
        let (simple, kv) = group_tag_names(&names);
        assert_eq!(simple, vec!["urgent".to_string()]);
        let mut expected_kv = BTreeMap::new();
        expected_kv.insert(
            "priority".to_string(),
            vec!["priority:high".to_string(), "priority:low".to_string()],
        );
        expected_kv.insert("type".to_string(), vec!["type:bug".to_string()]);
        assert_eq!(kv, expected_kv);
    }

    #[test]
    fn group_tag_names_empty_input() {
        let names: Vec<String> = vec![];
        let (simple, kv) = group_tag_names(&names);
        assert!(simple.is_empty());
        assert!(kv.is_empty());
    }

    #[test]
    fn group_tag_names_only_simple_tags() {
        let names = vec!["urgent".to_string(), "important".to_string()];
        let (simple, kv) = group_tag_names(&names);
        assert_eq!(simple, vec!["urgent".to_string(), "important".to_string()]);
        assert!(kv.is_empty());
    }

    #[test]
    fn group_tag_names_only_kv_tags() {
        let names = vec!["priority:high".to_string(), "type:bug".to_string()];
        let (simple, kv) = group_tag_names(&names);
        assert!(simple.is_empty());
        assert_eq!(kv.len(), 2);
        assert_eq!(kv["priority"], vec!["priority:high".to_string()]);
        assert_eq!(kv["type"], vec!["type:bug".to_string()]);
    }

    // ── matches_faceted_filter() ────────────────────────────────

    #[test]
    fn matches_faceted_filter_or_within_same_key() {
        let task_tags = vec!["priority:high", "type:bug"];
        let filter = vec!["priority:high".to_string(), "priority:low".to_string()];
        assert!(matches_faceted_filter(&task_tags, &filter));
    }

    #[test]
    fn matches_faceted_filter_and_between_keys() {
        let task_tags = vec!["priority:high"];
        let filter = vec!["priority:high".to_string(), "type:bug".to_string()];
        assert!(!matches_faceted_filter(&task_tags, &filter));
    }

    #[test]
    fn matches_faceted_filter_all_keys_present() {
        let task_tags = vec!["priority:high", "type:bug"];
        let filter = vec!["priority:high".to_string(), "type:bug".to_string()];
        assert!(matches_faceted_filter(&task_tags, &filter));
    }

    #[test]
    fn matches_faceted_filter_simple_tag() {
        let task_tags = vec!["urgent"];
        let filter = vec!["urgent".to_string()];
        assert!(matches_faceted_filter(&task_tags, &filter));
    }

    #[test]
    fn matches_faceted_filter_empty_filter_matches_all() {
        let task_tags = vec!["priority:high", "urgent"];
        let filter: Vec<String> = vec![];
        assert!(matches_faceted_filter(&task_tags, &filter));
    }

    #[test]
    fn matches_faceted_filter_mixed_simple_and_kv() {
        let task_tags = vec!["priority:high", "urgent"];
        let filter = vec!["priority:high".to_string(), "urgent".to_string()];
        assert!(matches_faceted_filter(&task_tags, &filter));
    }
}
