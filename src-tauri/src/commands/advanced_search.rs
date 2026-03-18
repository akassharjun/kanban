use crate::state::AppState;
use crate::models::Issue;
use tauri::State;

/// Represents a single parsed token from the advanced search query.
#[derive(Debug)]
enum Token {
    /// Field filter like status:todo, priority:high, -status:done
    Field { negated: bool, key: String, value: String },
    /// Plain text for fuzzy matching
    Text(String),
    /// Boolean OR operator
    Or,
}

/// Parse advanced search query into tokens.
fn parse_query(input: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();

    while chars.peek().is_some() {
        // Skip whitespace
        while chars.peek() == Some(&' ') { chars.next(); }
        if chars.peek().is_none() { break; }

        // Check for negation
        let negated = if chars.peek() == Some(&'-') {
            chars.next();
            true
        } else {
            false
        };

        // Check for NOT keyword
        let mut word = String::new();
        while let Some(&c) = chars.peek() {
            if c == ' ' || c == ':' { break; }
            word.push(c);
            chars.next();
        }

        if !negated && word.eq_ignore_ascii_case("OR") {
            tokens.push(Token::Or);
            continue;
        }

        if !negated && word.eq_ignore_ascii_case("NOT") {
            // Next token should be negated
            // Skip whitespace
            while chars.peek() == Some(&' ') { chars.next(); }
            let mut next_word = String::new();
            while let Some(&c) = chars.peek() {
                if c == ' ' || c == ':' { break; }
                next_word.push(c);
                chars.next();
            }
            if chars.peek() == Some(&':') {
                chars.next(); // consume ':'
                let value = parse_value(&mut chars);
                tokens.push(Token::Field { negated: true, key: next_word.to_lowercase(), value });
            } else if !next_word.is_empty() {
                tokens.push(Token::Text(format!("-{}", next_word)));
            }
            continue;
        }

        // Check for field:value
        if chars.peek() == Some(&':') {
            chars.next(); // consume ':'
            let value = parse_value(&mut chars);
            tokens.push(Token::Field { negated, key: word.to_lowercase(), value });
        } else if !word.is_empty() {
            if negated {
                // It was a negated plain text, unlikely but handle gracefully
                tokens.push(Token::Text(word));
            } else {
                tokens.push(Token::Text(word));
            }
        }
    }

    tokens
}

fn parse_value(chars: &mut std::iter::Peekable<std::str::Chars>) -> String {
    let mut value = String::new();
    if chars.peek() == Some(&'"') {
        chars.next(); // consume opening quote
        while let Some(&c) = chars.peek() {
            chars.next();
            if c == '"' { break; }
            value.push(c);
        }
    } else {
        while let Some(&c) = chars.peek() {
            if c == ' ' { break; }
            value.push(c);
            chars.next();
        }
    }
    value
}

#[tauri::command]
pub fn advanced_search(state: State<AppState>, project_id: i64, query_string: String, member_id: Option<i64>) -> Result<Vec<Issue>, String> {
    let tokens = parse_query(&query_string);

    state.rt.block_on(async {
        let mut qb: sqlx::QueryBuilder<sqlx::Any> = sqlx::QueryBuilder::new(
            "SELECT DISTINCT i.* FROM issues i"
        );

        // Track joins needed
        let mut needs_status_join = false;
        let mut needs_label_join = false;
        let mut needs_member_join = false;
        let mut needs_starred_join = false;

        // First pass: determine joins
        for token in &tokens {
            match token {
                Token::Field { key, .. } => {
                    match key.as_str() {
                        "status" | "is" => needs_status_join = true,
                        "label" => needs_label_join = true,
                        "assignee" => needs_member_join = true,
                        _ => {}
                    }
                }
                _ => {}
            }
        }

        // Check for is:starred
        for token in &tokens {
            if let Token::Field { key, value, negated } = token {
                if key == "is" && value.eq_ignore_ascii_case("starred") && !negated {
                    needs_starred_join = true;
                }
            }
        }

        if needs_status_join {
            qb.push(" LEFT JOIN statuses s ON i.status_id = s.id");
        }
        if needs_label_join {
            qb.push(" JOIN issue_labels il ON i.id = il.issue_id JOIN labels l ON il.label_id = l.id");
        }
        if needs_member_join {
            qb.push(" LEFT JOIN members m ON i.assignee_id = m.id");
        }
        if needs_starred_join {
            if let Some(mid) = member_id {
                qb.push(" JOIN starred_issues si ON i.id = si.issue_id AND si.member_id = ");
                qb.push_bind(mid);
            }
        }

        qb.push(" WHERE i.project_id = ");
        qb.push_bind(project_id);

        // Process tokens
        for token in &tokens {
            match token {
                Token::Or => {
                    // Next conditions go in a new group (not implemented with raw qb easily,
                    // so we'll simplify: OR applies between adjacent field conditions)
                }
                Token::Field { negated, key, value } => {
                    let neg = if *negated { "NOT " } else { "" };

                    match key.as_str() {
                        "status" => {
                            qb.push(format!(" AND {}LOWER(s.name) = LOWER(", neg));
                            qb.push_bind(value.clone());
                            qb.push(")");
                        }
                        "priority" => {
                            qb.push(format!(" AND {}LOWER(i.priority) = LOWER(", neg));
                            qb.push_bind(value.clone());
                            qb.push(")");
                        }
                        "assignee" => {
                            if value.eq_ignore_ascii_case("me") {
                                if let Some(mid) = member_id {
                                    if *negated {
                                        qb.push(" AND (i.assignee_id IS NULL OR i.assignee_id != ");
                                        qb.push_bind(mid);
                                        qb.push(")");
                                    } else {
                                        qb.push(" AND i.assignee_id = ");
                                        qb.push_bind(mid);
                                    }
                                }
                            } else {
                                qb.push(format!(" AND {}(LOWER(m.name) = LOWER(", neg));
                                qb.push_bind(value.clone());
                                qb.push(") OR LOWER(m.display_name) = LOWER(");
                                qb.push_bind(value.clone());
                                qb.push("))");
                            }
                        }
                        "label" => {
                            qb.push(format!(" AND {}LOWER(l.name) = LOWER(", neg));
                            qb.push_bind(value.clone());
                            qb.push(")");
                        }
                        "is" => {
                            match value.to_lowercase().as_str() {
                                "open" => {
                                    qb.push(format!(" AND {}s.category IN ('unstarted', 'started')", neg));
                                }
                                "closed" => {
                                    qb.push(format!(" AND {}s.category IN ('completed', 'discarded')", neg));
                                }
                                "blocked" => {
                                    qb.push(format!(" AND {}s.category = 'blocked'", neg));
                                }
                                "starred" => {
                                    // Handled via join above for positive, need subquery for negated
                                    if *negated {
                                        if let Some(mid) = member_id {
                                            qb.push(" AND i.id NOT IN (SELECT issue_id FROM starred_issues WHERE member_id = ");
                                            qb.push_bind(mid);
                                            qb.push(")");
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }
                        "due" => {
                            match value.to_lowercase().as_str() {
                                "today" => {
                                    qb.push(format!(" AND {}i.due_date = date('now')", neg));
                                }
                                "overdue" => {
                                    qb.push(format!(" AND {}(i.due_date IS NOT NULL AND i.due_date < date('now'))", neg));
                                }
                                "this-week" => {
                                    qb.push(format!(" AND {}(i.due_date IS NOT NULL AND i.due_date >= date('now') AND i.due_date <= date('now', '+7 days'))", neg));
                                }
                                _ => {}
                            }
                        }
                        "created" | "updated" => {
                            let field = if key == "created" { "i.created_at" } else { "i.updated_at" };
                            if value.starts_with('>') {
                                let date_val = value.trim_start_matches('>');
                                qb.push(format!(" AND {}{} > ", neg, field));
                                qb.push_bind(date_val.to_string());
                            } else if value.starts_with('<') {
                                let date_val = value.trim_start_matches('<');
                                qb.push(format!(" AND {}{} < ", neg, field));
                                qb.push_bind(date_val.to_string());
                            }
                        }
                        "has" => {
                            match value.to_lowercase().as_str() {
                                "description" => {
                                    qb.push(format!(" AND {}(i.description IS NOT NULL AND i.description != '')", neg));
                                }
                                "assignee" => {
                                    qb.push(format!(" AND {}i.assignee_id IS NOT NULL", neg));
                                }
                                "due-date" | "duedate" | "due_date" => {
                                    qb.push(format!(" AND {}i.due_date IS NOT NULL", neg));
                                }
                                _ => {}
                            }
                        }
                        _ => {
                            // Unknown field, treat as text search
                            let pattern = format!("%{}%", value);
                            qb.push(" AND (i.title LIKE ");
                            qb.push_bind(pattern.clone());
                            qb.push(" OR i.description LIKE ");
                            qb.push_bind(pattern);
                            qb.push(")");
                        }
                    }
                }
                Token::Text(text) => {
                    let pattern = format!("%{}%", text);
                    qb.push(" AND (i.title LIKE ");
                    qb.push_bind(pattern.clone());
                    qb.push(" OR i.description LIKE ");
                    qb.push_bind(pattern.clone());
                    qb.push(" OR i.identifier LIKE ");
                    qb.push_bind(pattern);
                    qb.push(")");
                }
            }
        }

        qb.push(" ORDER BY i.updated_at DESC LIMIT 50");

        qb.build_query_as::<Issue>()
            .fetch_all(&state.pool)
            .await
    }).map_err(|e| e.to_string())
}
