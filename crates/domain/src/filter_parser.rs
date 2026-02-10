use crate::filter_dsl::*;

#[derive(Debug, Clone, PartialEq)]
pub struct FilterParseError {
    pub message: String,
    pub span: (usize, usize),
}

impl std::fmt::Display for FilterParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} (at position {}..{})",
            self.message, self.span.0, self.span.1
        )
    }
}

pub fn parse_filter(input: &str) -> Result<FilterQuery, Vec<FilterParseError>> {
    let tokens = tokenize(input)?;
    let mut parser = Parser::new(&tokens, input);
    let query = parser.parse_query()?;
    Ok(query)
}

// ── Tokenizer ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
enum Token {
    Ident(String),
    StringLit(String),
    Number(f64),
    Eq,
    Ne,
    GlobMatch,
    GlobNotMatch,
    Gt,
    Lt,
    Gte,
    Lte,
    LParen,
    RParen,
    LBracket,
    RBracket,
    Comma,
}

#[derive(Debug, Clone)]
struct Spanned {
    token: Token,
    start: usize,
    end: usize,
}

fn tokenize(input: &str) -> Result<Vec<Spanned>, Vec<FilterParseError>> {
    let mut tokens = Vec::new();
    let chars: Vec<char> = input.chars().collect();
    let mut pos = 0;

    while pos < chars.len() {
        // Skip whitespace
        if chars[pos].is_whitespace() {
            pos += 1;
            continue;
        }

        let start = pos;

        match chars[pos] {
            '(' => {
                tokens.push(Spanned {
                    token: Token::LParen,
                    start,
                    end: pos + 1,
                });
                pos += 1;
            }
            ')' => {
                tokens.push(Spanned {
                    token: Token::RParen,
                    start,
                    end: pos + 1,
                });
                pos += 1;
            }
            '[' => {
                tokens.push(Spanned {
                    token: Token::LBracket,
                    start,
                    end: pos + 1,
                });
                pos += 1;
            }
            ']' => {
                tokens.push(Spanned {
                    token: Token::RBracket,
                    start,
                    end: pos + 1,
                });
                pos += 1;
            }
            ',' => {
                tokens.push(Spanned {
                    token: Token::Comma,
                    start,
                    end: pos + 1,
                });
                pos += 1;
            }
            '=' => {
                if pos + 1 < chars.len() && chars[pos + 1] == '~' {
                    tokens.push(Spanned {
                        token: Token::GlobMatch,
                        start,
                        end: pos + 2,
                    });
                    pos += 2;
                } else {
                    tokens.push(Spanned {
                        token: Token::Eq,
                        start,
                        end: pos + 1,
                    });
                    pos += 1;
                }
            }
            '!' => {
                if pos + 1 < chars.len() && chars[pos + 1] == '=' {
                    tokens.push(Spanned {
                        token: Token::Ne,
                        start,
                        end: pos + 2,
                    });
                    pos += 2;
                } else if pos + 1 < chars.len() && chars[pos + 1] == '~' {
                    tokens.push(Spanned {
                        token: Token::GlobNotMatch,
                        start,
                        end: pos + 2,
                    });
                    pos += 2;
                } else {
                    return Err(vec![FilterParseError {
                        message: "Expected '=' or '~' after '!'".into(),
                        span: (start, pos + 1),
                    }]);
                }
            }
            '>' => {
                if pos + 1 < chars.len() && chars[pos + 1] == '=' {
                    tokens.push(Spanned {
                        token: Token::Gte,
                        start,
                        end: pos + 2,
                    });
                    pos += 2;
                } else {
                    tokens.push(Spanned {
                        token: Token::Gt,
                        start,
                        end: pos + 1,
                    });
                    pos += 1;
                }
            }
            '<' => {
                if pos + 1 < chars.len() && chars[pos + 1] == '=' {
                    tokens.push(Spanned {
                        token: Token::Lte,
                        start,
                        end: pos + 2,
                    });
                    pos += 2;
                } else {
                    tokens.push(Spanned {
                        token: Token::Lt,
                        start,
                        end: pos + 1,
                    });
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
                if pos >= chars.len() {
                    return Err(vec![FilterParseError {
                        message: "Unterminated string literal".into(),
                        span: (start, pos),
                    }]);
                }
                pos += 1; // skip closing quote
                tokens.push(Spanned {
                    token: Token::StringLit(s),
                    start,
                    end: pos,
                });
            }
            c if c.is_ascii_digit() || c == '-' => {
                let mut num_str = String::new();
                num_str.push(chars[pos]);
                pos += 1;
                while pos < chars.len()
                    && (chars[pos].is_ascii_digit() || chars[pos] == '.')
                {
                    num_str.push(chars[pos]);
                    pos += 1;
                }
                // Check if next char is '-' followed by digit (date: 2024-01-15)
                if !num_str.starts_with('-')
                    && pos < chars.len()
                    && chars[pos] == '-'
                    && pos + 1 < chars.len()
                    && chars[pos + 1].is_ascii_digit()
                {
                    // Read date/datetime: YYYY-MM-DD or YYYY-MM-DDTHH:MM:SS
                    while pos < chars.len()
                        && (chars[pos].is_ascii_alphanumeric()
                            || chars[pos] == '-'
                            || chars[pos] == ':'
                            || chars[pos] == 'T')
                    {
                        num_str.push(chars[pos]);
                        pos += 1;
                    }
                    tokens.push(Spanned {
                        token: Token::StringLit(num_str),
                        start,
                        end: pos,
                    });
                } else if let Ok(n) = num_str.parse::<f64>() {
                    tokens.push(Spanned {
                        token: Token::Number(n),
                        start,
                        end: pos,
                    });
                } else {
                    return Err(vec![FilterParseError {
                        message: format!("Invalid number: {num_str}"),
                        span: (start, pos),
                    }]);
                }
            }
            c if c.is_alphanumeric() || c == '_' => {
                let mut ident = String::new();
                while pos < chars.len()
                    && (chars[pos].is_alphanumeric() || chars[pos] == '_')
                {
                    ident.push(chars[pos]);
                    pos += 1;
                }
                tokens.push(Spanned {
                    token: Token::Ident(ident),
                    start,
                    end: pos,
                });
            }
            c => {
                return Err(vec![FilterParseError {
                    message: format!("Unexpected character: '{c}'"),
                    span: (start, pos + 1),
                }]);
            }
        }
    }

    Ok(tokens)
}

// ── Recursive descent parser ───────────────────────────────────────────

struct Parser<'a> {
    tokens: &'a [Spanned],
    input: &'a str,
    pos: usize,
}

impl<'a> Parser<'a> {
    fn new(tokens: &'a [Spanned], input: &'a str) -> Self {
        Self {
            tokens,
            input,
            pos: 0,
        }
    }

    fn peek(&self) -> Option<&Spanned> {
        self.tokens.get(self.pos)
    }

    fn advance(&mut self) -> Option<&Spanned> {
        let t = self.tokens.get(self.pos);
        if t.is_some() {
            self.pos += 1;
        }
        t
    }

    fn current_span(&self) -> (usize, usize) {
        self.peek()
            .map(|t| (t.start, t.end))
            .unwrap_or((self.input.len(), self.input.len()))
    }

    fn err(&self, msg: impl Into<String>) -> Vec<FilterParseError> {
        vec![FilterParseError {
            message: msg.into(),
            span: self.current_span(),
        }]
    }

    fn is_ident_ci(&self, name: &str) -> bool {
        matches!(
            self.peek(),
            Some(Spanned { token: Token::Ident(s), .. })
                if s.eq_ignore_ascii_case(name)
        )
    }

    fn parse_query(&mut self) -> Result<FilterQuery, Vec<FilterParseError>> {
        if self.pos >= self.tokens.len() {
            return Ok(FilterQuery {
                expression: None,
                order_by: None,
            });
        }

        // Check if starts with ORDER BY (no expression)
        if self.is_ident_ci("order") {
            let order_by = self.parse_order_by()?;
            return Ok(FilterQuery {
                expression: None,
                order_by: Some(order_by),
            });
        }

        let expr = self.parse_or_expr()?;

        let order_by = if self.is_ident_ci("order") {
            Some(self.parse_order_by()?)
        } else {
            None
        };

        if self.pos < self.tokens.len() {
            return Err(self.err(format!(
                "Unexpected token at position {}",
                self.current_span().0
            )));
        }

        Ok(FilterQuery {
            expression: Some(expr),
            order_by,
        })
    }

    fn parse_order_by(&mut self) -> Result<OrderBy, Vec<FilterParseError>> {
        // Consume ORDER
        if !self.is_ident_ci("order") {
            return Err(self.err("Expected 'ORDER'"));
        }
        self.advance();

        // Consume BY
        if !self.is_ident_ci("by") {
            return Err(self.err("Expected 'BY' after 'ORDER'"));
        }
        self.advance();

        // Parse field
        let field = self.parse_field()?;

        // Optional ASC/DESC
        let direction = if self.is_ident_ci("asc") {
            self.advance();
            SortDirection::Asc
        } else if self.is_ident_ci("desc") {
            self.advance();
            SortDirection::Desc
        } else {
            SortDirection::Asc
        };

        Ok(OrderBy { field, direction })
    }

    fn parse_or_expr(&mut self) -> Result<FilterExpr, Vec<FilterParseError>> {
        let mut left = self.parse_and_expr()?;

        while self.is_ident_ci("or") {
            self.advance();
            let right = self.parse_and_expr()?;
            left = FilterExpr::Or(Box::new(left), Box::new(right));
        }

        Ok(left)
    }

    fn parse_and_expr(&mut self) -> Result<FilterExpr, Vec<FilterParseError>> {
        let mut left = self.parse_unary()?;

        while self.is_ident_ci("and") {
            self.advance();
            let right = self.parse_unary()?;
            left = FilterExpr::And(Box::new(left), Box::new(right));
        }

        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<FilterExpr, Vec<FilterParseError>> {
        if self.is_ident_ci("not") {
            self.advance();
            let expr = self.parse_unary()?;
            return Ok(FilterExpr::Not(Box::new(expr)));
        }

        if matches!(self.peek(), Some(Spanned { token: Token::LParen, .. })) {
            self.advance();
            let expr = self.parse_or_expr()?;
            if !matches!(self.peek(), Some(Spanned { token: Token::RParen, .. })) {
                return Err(self.err("Expected closing ')'"));
            }
            self.advance();
            return Ok(expr);
        }

        self.parse_condition()
    }

    fn parse_condition(&mut self) -> Result<FilterExpr, Vec<FilterParseError>> {
        let field = self.parse_field()?;
        let (op, value) = self.parse_op_and_value()?;
        Ok(FilterExpr::Condition(Condition { field, op, value }))
    }

    fn parse_field(&mut self) -> Result<FilterField, Vec<FilterParseError>> {
        match self.peek() {
            Some(Spanned {
                token: Token::Ident(s),
                start,
                end,
            }) => {
                let span = (*start, *end);
                let s = s.clone();
                self.advance();
                FilterField::from_str_ci(&s).ok_or_else(|| {
                    vec![FilterParseError {
                        message: format!("Unknown field: '{s}'"),
                        span,
                    }]
                })
            }
            _ => Err(self.err("Expected field name")),
        }
    }

    fn parse_op_and_value(
        &mut self,
    ) -> Result<(FilterOp, FilterValue), Vec<FilterParseError>> {
        match self.peek() {
            Some(Spanned { token: Token::Eq, .. }) => {
                self.advance();
                let val = self.parse_value()?;
                Ok((FilterOp::Eq, val))
            }
            Some(Spanned { token: Token::Ne, .. }) => {
                self.advance();
                let val = self.parse_value()?;
                Ok((FilterOp::Ne, val))
            }
            Some(Spanned { token: Token::GlobMatch, .. }) => {
                self.advance();
                let val = self.parse_value()?;
                Ok((FilterOp::GlobMatch, val))
            }
            Some(Spanned { token: Token::GlobNotMatch, .. }) => {
                self.advance();
                let val = self.parse_value()?;
                Ok((FilterOp::GlobNotMatch, val))
            }
            Some(Spanned { token: Token::Gt, .. }) => {
                self.advance();
                let val = self.parse_value()?;
                Ok((FilterOp::Gt, val))
            }
            Some(Spanned { token: Token::Lt, .. }) => {
                self.advance();
                let val = self.parse_value()?;
                Ok((FilterOp::Lt, val))
            }
            Some(Spanned { token: Token::Gte, .. }) => {
                self.advance();
                let val = self.parse_value()?;
                Ok((FilterOp::Gte, val))
            }
            Some(Spanned { token: Token::Lte, .. }) => {
                self.advance();
                let val = self.parse_value()?;
                Ok((FilterOp::Lte, val))
            }
            Some(Spanned { token: Token::Ident(s), .. })
                if s.eq_ignore_ascii_case("is") =>
            {
                self.advance();
                if self.is_ident_ci("not") {
                    self.advance();
                    let val = self.parse_value()?;
                    Ok((FilterOp::IsNot, val))
                } else {
                    let val = self.parse_value()?;
                    Ok((FilterOp::Is, val))
                }
            }
            Some(Spanned { token: Token::Ident(s), .. })
                if s.eq_ignore_ascii_case("not") =>
            {
                self.advance();
                if self.is_ident_ci("in") {
                    self.advance();
                    let val = self.parse_value()?;
                    Ok((FilterOp::NotIn, val))
                } else {
                    Err(self.err("Expected 'in' after 'not'"))
                }
            }
            Some(Spanned { token: Token::Ident(s), .. })
                if s.eq_ignore_ascii_case("in") =>
            {
                self.advance();
                let val = self.parse_value()?;
                Ok((FilterOp::In, val))
            }
            _ => Err(self.err("Expected operator")),
        }
    }

    fn parse_value(&mut self) -> Result<FilterValue, Vec<FilterParseError>> {
        match self.peek() {
            Some(Spanned { token: Token::StringLit(s), .. }) => {
                let s = s.clone();
                self.advance();
                Ok(FilterValue::String(s))
            }
            Some(Spanned { token: Token::Number(n), .. }) => {
                let n = *n;
                self.advance();
                Ok(FilterValue::Number(n))
            }
            Some(Spanned { token: Token::Ident(s), .. })
                if s.eq_ignore_ascii_case("null") =>
            {
                self.advance();
                Ok(FilterValue::Null)
            }
            Some(Spanned { token: Token::Ident(s), .. })
                if s.eq_ignore_ascii_case("true") =>
            {
                self.advance();
                Ok(FilterValue::Bool(true))
            }
            Some(Spanned { token: Token::Ident(s), .. })
                if s.eq_ignore_ascii_case("false") =>
            {
                self.advance();
                Ok(FilterValue::Bool(false))
            }
            // Bare identifiers as string values (e.g., COMPLETED, ACTIVE)
            Some(Spanned { token: Token::Ident(s), .. }) => {
                let s = s.clone();
                self.advance();
                Ok(FilterValue::String(s))
            }
            Some(Spanned { token: Token::LBracket, .. }) => {
                self.parse_array()
            }
            _ => Err(self.err("Expected value")),
        }
    }

    fn parse_array(&mut self) -> Result<FilterValue, Vec<FilterParseError>> {
        // Consume '['
        self.advance();

        let mut values = Vec::new();

        if matches!(self.peek(), Some(Spanned { token: Token::RBracket, .. })) {
            self.advance();
            return Ok(FilterValue::Array(values));
        }

        values.push(self.parse_value()?);

        while matches!(self.peek(), Some(Spanned { token: Token::Comma, .. })) {
            self.advance();
            values.push(self.parse_value()?);
        }

        if !matches!(self.peek(), Some(Spanned { token: Token::RBracket, .. })) {
            return Err(self.err("Expected ']'"));
        }
        self.advance();

        Ok(FilterValue::Array(values))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_query() {
        let result = parse_filter("").unwrap();
        assert_eq!(
            result,
            FilterQuery {
                expression: None,
                order_by: None
            }
        );
    }

    #[test]
    fn test_simple_eq() {
        let result = parse_filter("status = 'COMPLETED'").unwrap();
        assert_eq!(
            result.expression,
            Some(FilterExpr::Condition(Condition {
                field: FilterField::Status,
                op: FilterOp::Eq,
                value: FilterValue::String("COMPLETED".into()),
            }))
        );
    }

    #[test]
    fn test_glob_match() {
        let result = parse_filter("tags =~ 'work:*'").unwrap();
        assert_eq!(
            result.expression,
            Some(FilterExpr::Condition(Condition {
                field: FilterField::Tags,
                op: FilterOp::GlobMatch,
                value: FilterValue::String("work:*".into()),
            }))
        );
    }

    #[test]
    fn test_and_expression() {
        let result =
            parse_filter("status = 'ACTIVE' AND project = 'My Project'").unwrap();
        match result.expression {
            Some(FilterExpr::And(left, right)) => {
                assert!(matches!(*left, FilterExpr::Condition(_)));
                assert!(matches!(*right, FilterExpr::Condition(_)));
            }
            _ => panic!("Expected And expression"),
        }
    }

    #[test]
    fn test_or_expression() {
        let result = parse_filter("status = 'ACTIVE' OR status = 'COMPLETED'").unwrap();
        assert!(matches!(result.expression, Some(FilterExpr::Or(_, _))));
    }

    #[test]
    fn test_not_expression() {
        let result = parse_filter("NOT status = 'COMPLETED'").unwrap();
        assert!(matches!(result.expression, Some(FilterExpr::Not(_))));
    }

    #[test]
    fn test_parentheses() {
        let result = parse_filter(
            "(status = 'ACTIVE' OR status = 'COMPLETED') AND project = 'Work'",
        )
        .unwrap();
        assert!(matches!(result.expression, Some(FilterExpr::And(_, _))));
    }

    #[test]
    fn test_is_null() {
        let result = parse_filter("due_date is null").unwrap();
        assert_eq!(
            result.expression,
            Some(FilterExpr::Condition(Condition {
                field: FilterField::DueDate,
                op: FilterOp::Is,
                value: FilterValue::Null,
            }))
        );
    }

    #[test]
    fn test_is_not_null() {
        let result = parse_filter("due_date is not null").unwrap();
        assert_eq!(
            result.expression,
            Some(FilterExpr::Condition(Condition {
                field: FilterField::DueDate,
                op: FilterOp::IsNot,
                value: FilterValue::Null,
            }))
        );
    }

    #[test]
    fn test_in_array() {
        let result =
            parse_filter("tags in ['work', 'personal']").unwrap();
        assert_eq!(
            result.expression,
            Some(FilterExpr::Condition(Condition {
                field: FilterField::Tags,
                op: FilterOp::In,
                value: FilterValue::Array(vec![
                    FilterValue::String("work".into()),
                    FilterValue::String("personal".into()),
                ]),
            }))
        );
    }

    #[test]
    fn test_not_in_array() {
        let result = parse_filter("tags not in ['archive']").unwrap();
        assert_eq!(
            result.expression,
            Some(FilterExpr::Condition(Condition {
                field: FilterField::Tags,
                op: FilterOp::NotIn,
                value: FilterValue::Array(vec![FilterValue::String(
                    "archive".into()
                )]),
            }))
        );
    }

    #[test]
    fn test_order_by() {
        let result =
            parse_filter("status = 'ACTIVE' ORDER BY due_date DESC").unwrap();
        assert!(result.expression.is_some());
        assert_eq!(
            result.order_by,
            Some(OrderBy {
                field: FilterField::DueDate,
                direction: SortDirection::Desc,
            })
        );
    }

    #[test]
    fn test_order_by_only() {
        let result = parse_filter("ORDER BY created ASC").unwrap();
        assert!(result.expression.is_none());
        assert_eq!(
            result.order_by,
            Some(OrderBy {
                field: FilterField::Created,
                direction: SortDirection::Asc,
            })
        );
    }

    #[test]
    fn test_comparison_operators() {
        let result = parse_filter("due_date > '2024-01-01'").unwrap();
        assert_eq!(
            result.expression,
            Some(FilterExpr::Condition(Condition {
                field: FilterField::DueDate,
                op: FilterOp::Gt,
                value: FilterValue::String("2024-01-01".into()),
            }))
        );
    }

    #[test]
    fn test_date_value() {
        let result = parse_filter("due_date >= 2024-01-15").unwrap();
        assert_eq!(
            result.expression,
            Some(FilterExpr::Condition(Condition {
                field: FilterField::DueDate,
                op: FilterOp::Gte,
                value: FilterValue::String("2024-01-15".into()),
            }))
        );
    }

    #[test]
    fn test_bare_ident_value() {
        let result = parse_filter("status = COMPLETED").unwrap();
        assert_eq!(
            result.expression,
            Some(FilterExpr::Condition(Condition {
                field: FilterField::Status,
                op: FilterOp::Eq,
                value: FilterValue::String("COMPLETED".into()),
            }))
        );
    }

    #[test]
    fn test_case_insensitive_keywords() {
        let result = parse_filter("status = 'active' and title =~ 'hello*'").unwrap();
        assert!(matches!(result.expression, Some(FilterExpr::And(_, _))));
    }

    #[test]
    fn test_case_insensitive_fields() {
        let result = parse_filter("Title = 'hello'").unwrap();
        assert_eq!(
            result.expression,
            Some(FilterExpr::Condition(Condition {
                field: FilterField::Title,
                op: FilterOp::Eq,
                value: FilterValue::String("hello".into()),
            }))
        );
    }

    #[test]
    fn test_complex_expression() {
        let result = parse_filter(
            "tags =~ 'work:*' AND status != 'COMPLETED' \
             AND (project = 'Alpha' OR project = 'Beta') \
             ORDER BY due_date ASC",
        )
        .unwrap();
        assert!(result.expression.is_some());
        assert!(result.order_by.is_some());
    }

    #[test]
    fn test_unknown_field_error() {
        let result = parse_filter("foobar = 'test'");
        assert!(result.is_err());
        let errs = result.unwrap_err();
        assert!(errs[0].message.contains("Unknown field"));
    }

    #[test]
    fn test_unterminated_string_error() {
        let result = parse_filter("title = 'hello");
        assert!(result.is_err());
        let errs = result.unwrap_err();
        assert!(errs[0].message.contains("Unterminated string"));
    }

    #[test]
    fn test_field_aliases() {
        assert!(parse_filter("due = '2024-01-01'").is_ok());
        assert!(parse_filter("start = '2024-01-01'").is_ok());
        assert!(parse_filter("col = 'Done'").is_ok());
        assert!(parse_filter("tag = 'work'").is_ok());
        assert!(parse_filter("created_at > '2024-01-01'").is_ok());
        assert!(parse_filter("updated_at > '2024-01-01'").is_ok());
    }
}
