use crate::filter::dsl::FilterField;
use crate::filter::field_registry::TaskFieldRegistry;

#[derive(Debug, Clone, PartialEq)]
pub enum DslCompletionContext {
    FieldName {
        partial: String,
        start: usize,
    },
    FieldValue {
        field: FilterField,
        partial: String,
        start: usize,
    },
    ArrayValue {
        field: FilterField,
        partial: String,
        start: usize,
    },
    Keyword {
        partial: String,
        start: usize,
    },
    None,
}

#[derive(Debug, Clone, PartialEq)]
enum ContextToken {
    Ident(String, usize),
    StringLit(String, usize),
    Operator,
    LBracket(usize),
    RBracket,
    LParen,
    RParen,
    Comma,
    Partial(String, usize),
}

fn tokenize_for_context(text: &str, cursor: usize) -> Vec<ContextToken> {
    let before = &text[..cursor];
    let chars: Vec<char> = before.chars().collect();
    let mut tokens = Vec::new();
    let mut pos = 0;

    while pos < chars.len() {
        if chars[pos].is_whitespace() {
            pos += 1;
            continue;
        }

        let start = pos;

        match chars[pos] {
            '(' => {
                tokens.push(ContextToken::LParen);
                pos += 1;
            }
            ')' => {
                tokens.push(ContextToken::RParen);
                pos += 1;
            }
            '[' => {
                tokens.push(ContextToken::LBracket(start));
                pos += 1;
            }
            ']' => {
                tokens.push(ContextToken::RBracket);
                pos += 1;
            }
            ',' => {
                tokens.push(ContextToken::Comma);
                pos += 1;
            }
            '=' | '!' | '>' | '<' => {
                tokens.push(ContextToken::Operator);
                pos += 1;
                if pos < chars.len() && matches!(chars[pos], '=' | '~') {
                    pos += 1;
                }
            }
            '\'' => {
                pos += 1;
                let mut s = String::new();
                while pos < chars.len() && chars[pos] != '\'' {
                    s.push(chars[pos]);
                    pos += 1;
                }
                if pos < chars.len() {
                    pos += 1; // skip closing quote
                    tokens.push(ContextToken::StringLit(s, start));
                } else {
                    // Unclosed string — cursor is inside the string
                    return tokens;
                }
            }
            c if c.is_alphanumeric() || c == '_' => {
                let mut ident = String::new();
                while pos < chars.len() && (chars[pos].is_alphanumeric() || chars[pos] == '_') {
                    ident.push(chars[pos]);
                    pos += 1;
                }
                if pos == chars.len() && cursor == text.len() {
                    // The ident ends at the cursor — it's a partial
                    tokens.push(ContextToken::Partial(ident, start));
                } else if pos == chars.len() {
                    tokens.push(ContextToken::Partial(ident, start));
                } else {
                    tokens.push(ContextToken::Ident(ident, start));
                }
            }
            '-' => {
                // Could be negative number or date
                pos += 1;
                while pos < chars.len()
                    && (chars[pos].is_ascii_digit()
                        || chars[pos] == '-'
                        || chars[pos] == ':'
                        || chars[pos] == '.'
                        || chars[pos] == 'T')
                {
                    pos += 1;
                }
                tokens.push(ContextToken::Ident("_number".into(), start));
            }
            c if c.is_ascii_digit() => {
                while pos < chars.len()
                    && (chars[pos].is_ascii_digit()
                        || chars[pos] == '-'
                        || chars[pos] == ':'
                        || chars[pos] == '.'
                        || chars[pos] == 'T')
                {
                    pos += 1;
                }
                tokens.push(ContextToken::Ident("_number".into(), start));
            }
            _ => {
                pos += 1;
            }
        }
    }

    tokens
}

fn is_operator_keyword(s: &str) -> bool {
    matches!(s.to_uppercase().as_str(), "IS" | "IN" | "NOT")
}

pub fn detect_completion_context(text: &str, cursor: usize) -> DslCompletionContext {
    let cursor = cursor.min(text.len());

    // If cursor is inside an unclosed string, don't autocomplete
    let before = &text[..cursor];
    let single_quotes = before.chars().filter(|c| *c == '\'').count();
    if single_quotes % 2 != 0 {
        return DslCompletionContext::None;
    }

    let tokens = tokenize_for_context(text, cursor);

    if tokens.is_empty() {
        return DslCompletionContext::FieldName {
            partial: String::new(),
            start: cursor,
        };
    }

    // Check if cursor is right after whitespace or structural char
    let at_word_boundary = cursor == 0
        || before.ends_with(|c: char| c.is_whitespace())
        || before.ends_with('(')
        || before.ends_with('[')
        || before.ends_with(',');

    // Find last meaningful tokens for context
    let last = tokens.last().unwrap();

    match last {
        // Currently typing something
        ContextToken::Partial(partial, start) => {
            let partial_upper = partial.to_uppercase();

            // Look at what comes before this partial
            let preceding: Vec<_> = tokens[..tokens.len() - 1]
                .iter()
                .filter(|t| !matches!(t, ContextToken::LParen | ContextToken::RParen))
                .collect();

            // After ORDER → could be typing BY
            if let Some(ContextToken::Ident(s, _)) = preceding.last() {
                if s.eq_ignore_ascii_case("ORDER") {
                    return DslCompletionContext::FieldName {
                        partial: partial.clone(),
                        start: *start,
                    };
                }
            }

            // After ORDER BY → field name
            if preceding.len() >= 2 {
                if let (Some(ContextToken::Ident(s1, _)), Some(ContextToken::Ident(s2, _))) =
                    (preceding.get(preceding.len() - 2), preceding.last())
                {
                    if s1.eq_ignore_ascii_case("ORDER") && s2.eq_ignore_ascii_case("BY") {
                        return DslCompletionContext::FieldName {
                            partial: partial.clone(),
                            start: *start,
                        };
                    }
                }
            }

            // Check if inside array: look for [ ... field in pattern
            if is_inside_array(&tokens[..tokens.len() - 1]) {
                if let Some(field) = find_array_field(&tokens[..tokens.len() - 1]) {
                    return DslCompletionContext::ArrayValue {
                        field,
                        partial: partial.clone(),
                        start: *start,
                    };
                }
            }

            // After <field> <operator> → FieldValue
            if let Some(field) = find_field_before_operator(&preceding) {
                // The partial is a value
                if !is_operator_keyword(&partial_upper) {
                    return DslCompletionContext::FieldValue {
                        field,
                        partial: partial.clone(),
                        start: *start,
                    };
                }
            }

            // After a complete condition or after AND/OR/NOT/( → FieldName
            if is_field_position(&preceding) {
                if partial_upper == "AND"
                    || partial_upper == "OR"
                    || partial_upper == "NOT"
                    || partial_upper.starts_with("ORD")
                {
                    return DslCompletionContext::Keyword {
                        partial: partial.clone(),
                        start: *start,
                    };
                }
                return DslCompletionContext::FieldName {
                    partial: partial.clone(),
                    start: *start,
                };
            }

            // Could be typing a keyword after a condition
            if is_after_condition(&tokens[..tokens.len() - 1]) {
                return DslCompletionContext::Keyword {
                    partial: partial.clone(),
                    start: *start,
                };
            }

            DslCompletionContext::FieldName {
                partial: partial.clone(),
                start: *start,
            }
        }

        // Just placed cursor after whitespace
        _ if at_word_boundary => {
            let meaningful: Vec<_> = tokens
                .iter()
                .filter(|t| !matches!(t, ContextToken::LParen | ContextToken::RParen))
                .collect();

            // Inside array
            if is_inside_array(&tokens) {
                if let Some(field) = find_array_field(&tokens) {
                    return DslCompletionContext::ArrayValue {
                        field,
                        partial: String::new(),
                        start: cursor,
                    };
                }
            }

            // After operator → FieldValue
            if let Some(field) = find_field_before_operator(&meaningful) {
                return DslCompletionContext::FieldValue {
                    field,
                    partial: String::new(),
                    start: cursor,
                };
            }

            // After a complete condition → Keyword
            if is_after_condition(&tokens) {
                return DslCompletionContext::Keyword {
                    partial: String::new(),
                    start: cursor,
                };
            }

            // After keyword (AND/OR/NOT) or at start → FieldName
            if is_field_position(&meaningful) {
                return DslCompletionContext::FieldName {
                    partial: String::new(),
                    start: cursor,
                };
            }

            // After ORDER BY field → could suggest ASC/DESC
            DslCompletionContext::FieldName {
                partial: String::new(),
                start: cursor,
            }
        }

        // After a complete value token
        ContextToken::StringLit(_, _) | ContextToken::Ident(_, _) => {
            if is_after_condition(&tokens) {
                DslCompletionContext::Keyword {
                    partial: String::new(),
                    start: cursor,
                }
            } else {
                DslCompletionContext::None
            }
        }

        ContextToken::Operator => {
            let meaningful: Vec<_> = tokens
                .iter()
                .filter(|t| !matches!(t, ContextToken::LParen | ContextToken::RParen))
                .collect();
            if let Some(field) = find_field_before_operator(&meaningful) {
                DslCompletionContext::FieldValue {
                    field,
                    partial: String::new(),
                    start: cursor,
                }
            } else {
                DslCompletionContext::None
            }
        }

        ContextToken::LBracket(_) => {
            if let Some(field) = find_array_field(&tokens) {
                DslCompletionContext::ArrayValue {
                    field,
                    partial: String::new(),
                    start: cursor,
                }
            } else {
                DslCompletionContext::None
            }
        }

        ContextToken::Comma => {
            if is_inside_array(&tokens) {
                if let Some(field) = find_array_field(&tokens) {
                    return DslCompletionContext::ArrayValue {
                        field,
                        partial: String::new(),
                        start: cursor,
                    };
                }
            }
            DslCompletionContext::None
        }

        _ => DslCompletionContext::None,
    }
}

fn is_inside_array(tokens: &[ContextToken]) -> bool {
    let mut depth = 0i32;
    for t in tokens {
        match t {
            ContextToken::LBracket(_) => depth += 1,
            ContextToken::RBracket => depth -= 1,
            _ => {}
        }
    }
    depth > 0
}

fn find_array_field(tokens: &[ContextToken]) -> Option<FilterField> {
    // Walk backwards to find the pattern: <field> in [
    let mut bracket_depth = 0i32;
    for t in tokens.iter().rev() {
        match t {
            ContextToken::RBracket => bracket_depth += 1,
            ContextToken::LBracket(_) => {
                bracket_depth -= 1;
                if bracket_depth < 0 {
                    break;
                }
            }
            _ => {}
        }
    }

    // Look for <field> <in-keyword> before the [
    let non_bracket: Vec<_> = tokens.iter().collect();
    for (i, t) in non_bracket.iter().enumerate() {
        if let ContextToken::LBracket(_) = t {
            // Look for 'in' keyword before bracket
            if i >= 2 {
                if let ContextToken::Ident(kw, _) = &non_bracket[i - 1] {
                    if kw.eq_ignore_ascii_case("in") || kw.eq_ignore_ascii_case("not") {
                        // find field before in/not
                        let field_idx = if kw.eq_ignore_ascii_case("not") && i >= 3 {
                            if let ContextToken::Ident(kw2, _) = &non_bracket[i - 2] {
                                if kw2.eq_ignore_ascii_case("in") {
                                    i - 3
                                } else {
                                    i - 2
                                }
                            } else {
                                i - 2
                            }
                        } else {
                            i - 2
                        };
                        if let ContextToken::Ident(field_name, _)
                        | ContextToken::Partial(field_name, _) = &non_bracket[field_idx]
                        {
                            return TaskFieldRegistry::from_str_ci(field_name);
                        }
                    }
                }
            }
            if i >= 1 {
                if let ContextToken::Ident(kw, _) = &non_bracket[i - 1] {
                    if kw.eq_ignore_ascii_case("in") && i >= 2 {
                        if let ContextToken::Ident(field_name, _)
                        | ContextToken::Partial(field_name, _) = &non_bracket[i - 2]
                        {
                            return TaskFieldRegistry::from_str_ci(field_name);
                        }
                    }
                }
            }
        }
    }
    None
}

fn find_field_before_operator(tokens: &[&ContextToken]) -> Option<FilterField> {
    // Look for the pattern: <field> <operator> at the end
    let len = tokens.len();
    if len == 0 {
        return None;
    }

    // Last token is operator
    if matches!(tokens[len - 1], ContextToken::Operator) && len >= 2 {
        if let ContextToken::Ident(name, _) | ContextToken::Partial(name, _) = tokens[len - 2] {
            return TaskFieldRegistry::from_str_ci(name);
        }
    }

    // Last token is 'is' or 'in' (keyword operators)
    if let ContextToken::Ident(kw, _) = tokens[len - 1] {
        if (kw.eq_ignore_ascii_case("is") || kw.eq_ignore_ascii_case("in")) && len >= 2 {
            if let ContextToken::Ident(name, _) | ContextToken::Partial(name, _) = tokens[len - 2] {
                return TaskFieldRegistry::from_str_ci(name);
            }
        }
        // field not [in] — "not" after "is"
        if kw.eq_ignore_ascii_case("not") && len >= 3 {
            if let ContextToken::Ident(kw2, _) = tokens[len - 2] {
                if kw2.eq_ignore_ascii_case("is") && len >= 3 {
                    if let ContextToken::Ident(name, _) | ContextToken::Partial(name, _) =
                        tokens[len - 3]
                    {
                        return TaskFieldRegistry::from_str_ci(name);
                    }
                }
            }
        }
    }

    None
}

fn is_field_position(tokens: &[&ContextToken]) -> bool {
    if tokens.is_empty() {
        return true;
    }

    match tokens.last() {
        Some(ContextToken::Ident(s, _)) => {
            s.eq_ignore_ascii_case("AND")
                || s.eq_ignore_ascii_case("OR")
                || s.eq_ignore_ascii_case("NOT")
                || s.eq_ignore_ascii_case("BY")
        }
        Some(ContextToken::LParen) => true,
        _ => false,
    }
}

fn is_after_condition(tokens: &[ContextToken]) -> bool {
    // A condition is complete when we see: <field> <op> <value>
    // Walk backwards to determine if the last thing is a complete value
    let meaningful: Vec<_> = tokens
        .iter()
        .filter(|t| !matches!(t, ContextToken::LParen | ContextToken::RParen))
        .collect();

    let len = meaningful.len();
    if len < 3 {
        return false;
    }

    // Pattern: Ident Operator (StringLit|Ident)
    let last = meaningful[len - 1];
    let is_value = matches!(
        last,
        ContextToken::StringLit(_, _) | ContextToken::Ident(_, _)
    );
    if !is_value {
        return false;
    }

    // Ensure the value isn't a keyword that's part of incomplete syntax
    if let ContextToken::Ident(s, _) = last {
        if s.eq_ignore_ascii_case("AND")
            || s.eq_ignore_ascii_case("OR")
            || s.eq_ignore_ascii_case("NOT")
            || s.eq_ignore_ascii_case("ORDER")
            || s.eq_ignore_ascii_case("BY")
            || s.eq_ignore_ascii_case("IS")
            || s.eq_ignore_ascii_case("IN")
        {
            return false;
        }
    }

    // Check second-to-last is an operator
    let has_op = matches!(meaningful[len - 2], ContextToken::Operator)
        || matches!(meaningful[len - 2], ContextToken::Ident(ref s, _)
            if s.eq_ignore_ascii_case("is") || s.eq_ignore_ascii_case("in")
                || s.eq_ignore_ascii_case("not"));

    if !has_op {
        return false;
    }

    // Check for a field before the operator
    if let ContextToken::Ident(ref name, _) = meaningful[len - 3] {
        if TaskFieldRegistry::from_str_ci(name).is_some() {
            return true;
        }
        // "is not" pattern: field is not <value> — field at len-4
        if name.eq_ignore_ascii_case("is") && len >= 4 {
            if let ContextToken::Ident(ref field_name, _) = meaningful[len - 4] {
                return TaskFieldRegistry::from_str_ci(field_name).is_some();
            }
        }
    }

    // Check ] (end of array) as value
    if matches!(last, ContextToken::Ident(_, _)) {
        // Might be after an array expression
        return false;
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_input() {
        let ctx = detect_completion_context("", 0);
        assert!(
            matches!(ctx, DslCompletionContext::FieldName { ref partial, start }
                if partial.is_empty() && start == 0
            )
        );
    }

    #[test]
    fn test_partial_field_name() {
        let ctx = detect_completion_context("sta", 3);
        assert!(
            matches!(ctx, DslCompletionContext::FieldName { ref partial, start }
                if partial == "sta" && start == 0
            )
        );
    }

    #[test]
    fn test_field_after_and() {
        let ctx = detect_completion_context("status = 'ACTIVE' AND ", 22);
        assert!(
            matches!(ctx, DslCompletionContext::FieldName { ref partial, .. }
                if partial.is_empty()
            )
        );
    }

    #[test]
    fn test_field_after_and_partial() {
        let ctx = detect_completion_context("status = 'ACTIVE' AND ti", 24);
        assert!(
            matches!(ctx, DslCompletionContext::FieldName { ref partial, .. }
                if partial == "ti"
            )
        );
    }

    #[test]
    fn test_value_after_operator() {
        let ctx = detect_completion_context("tags = ", 7);
        assert!(matches!(ctx, DslCompletionContext::FieldValue {
            field: FilterField::Tags,
            ref partial,
            ..
        } if partial.is_empty()
        ));
    }

    #[test]
    fn test_value_with_partial() {
        let ctx = detect_completion_context("status = AC", 11);
        assert!(matches!(ctx, DslCompletionContext::FieldValue {
            field: FilterField::Status,
            ref partial,
            ..
        } if partial == "AC"
        ));
    }

    #[test]
    fn test_keyword_after_condition() {
        let ctx = detect_completion_context("status = 'ACTIVE' ", 18);
        assert!(
            matches!(ctx, DslCompletionContext::Keyword { ref partial, .. }
                if partial.is_empty()
            )
        );
    }

    #[test]
    fn test_keyword_partial() {
        let ctx = detect_completion_context("status = 'ACTIVE' AN", 20);
        assert!(
            matches!(ctx, DslCompletionContext::Keyword { ref partial, .. }
                if partial == "AN"
            )
        );
    }

    #[test]
    fn test_inside_string_no_completion() {
        let ctx = detect_completion_context("title = 'hel", 12);
        assert!(matches!(ctx, DslCompletionContext::None));
    }

    #[test]
    fn test_array_value() {
        let ctx = detect_completion_context("tags in [", 9);
        assert!(matches!(ctx, DslCompletionContext::ArrayValue {
            field: FilterField::Tags,
            ref partial,
            ..
        } if partial.is_empty()
        ));
    }

    #[test]
    fn test_array_value_after_comma() {
        let ctx = detect_completion_context("tags in ['work', ", 17);
        assert!(matches!(ctx, DslCompletionContext::ArrayValue {
            field: FilterField::Tags,
            ref partial,
            ..
        } if partial.is_empty()
        ));
    }

    #[test]
    fn test_array_value_partial() {
        let ctx = detect_completion_context("tags in [wo", 11);
        assert!(matches!(ctx, DslCompletionContext::ArrayValue {
            field: FilterField::Tags,
            ref partial,
            ..
        } if partial == "wo"
        ));
    }

    #[test]
    fn test_field_after_paren() {
        let ctx = detect_completion_context("(", 1);
        assert!(
            matches!(ctx, DslCompletionContext::FieldName { ref partial, .. }
                if partial.is_empty()
            )
        );
    }

    #[test]
    fn test_field_after_not() {
        let ctx = detect_completion_context("NOT ", 4);
        assert!(
            matches!(ctx, DslCompletionContext::FieldName { ref partial, .. }
                if partial.is_empty()
            )
        );
    }

    #[test]
    fn test_project_value() {
        let ctx = detect_completion_context("project = ", 10);
        assert!(matches!(ctx, DslCompletionContext::FieldValue {
            field: FilterField::Project,
            ref partial,
            ..
        } if partial.is_empty()
        ));
    }

    #[test]
    fn test_order_by_field() {
        let ctx = detect_completion_context("status = 'ACTIVE' ORDER BY ", 27);
        assert!(
            matches!(ctx, DslCompletionContext::FieldName { ref partial, .. }
                if partial.is_empty()
            )
        );
    }

    #[test]
    fn test_order_by_partial() {
        let ctx = detect_completion_context("status = 'ACTIVE' ORDER BY du", 29);
        assert!(
            matches!(ctx, DslCompletionContext::FieldName { ref partial, .. }
                if partial == "du"
            )
        );
    }
}
