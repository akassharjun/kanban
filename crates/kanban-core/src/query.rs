use crate::types::Priority;
use chrono::NaiveDate;
use rusqlite::types::Value;
use uuid::Uuid;

#[derive(Debug, Clone, Default)]
pub struct IssueFilter {
    pub project_id: Option<Uuid>,
    pub status_ids: Vec<Uuid>,
    pub priorities: Vec<Priority>,
    pub label_ids: Vec<Uuid>,
    pub due_before: Option<NaiveDate>,
    pub due_after: Option<NaiveDate>,
    pub search_text: Option<String>,
    pub sort: SortBy,
    pub reverse: bool,
    pub limit: Option<i64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SortBy {
    #[default]
    Manual,
    Priority,
    Created,
    Updated,
    Due,
}

impl IssueFilter {
    #[must_use]
    pub fn for_project(project_id: Uuid) -> Self {
        Self {
            project_id: Some(project_id),
            ..Self::default()
        }
    }

    #[must_use]
    pub fn build_sql(&self, base: &str) -> (String, Vec<Value>) {
        use std::fmt::Write as _;

        let mut sql = String::from(base);
        let mut params: Vec<Value> = Vec::new();
        let mut first = true;
        let and = |sql: &mut String, first: &mut bool| {
            if *first {
                sql.push_str(" WHERE ");
                *first = false;
            } else {
                sql.push_str(" AND ");
            }
        };

        if let Some(pid) = self.project_id {
            and(&mut sql, &mut first);
            sql.push_str("project_id = ?");
            params.push(Value::Text(pid.to_string()));
        }
        if !self.status_ids.is_empty() {
            and(&mut sql, &mut first);
            let _ = write!(
                sql,
                "status_id IN ({})",
                placeholders(self.status_ids.len())
            );
            for s in &self.status_ids {
                params.push(Value::Text(s.to_string()));
            }
        }
        if !self.priorities.is_empty() {
            and(&mut sql, &mut first);
            let _ = write!(sql, "priority IN ({})", placeholders(self.priorities.len()));
            for p in &self.priorities {
                params.push(Value::Text(p.as_str().to_string()));
            }
        }
        if let Some(d) = self.due_before {
            and(&mut sql, &mut first);
            sql.push_str("due_date IS NOT NULL AND due_date < ?");
            params.push(Value::Text(d.to_string()));
        }
        if let Some(d) = self.due_after {
            and(&mut sql, &mut first);
            sql.push_str("due_date IS NOT NULL AND due_date > ?");
            params.push(Value::Text(d.to_string()));
        }
        if !self.label_ids.is_empty() {
            and(&mut sql, &mut first);
            let _ = write!(
                sql,
                "id IN (SELECT issue_id FROM issue_labels WHERE label_id IN ({}))",
                placeholders(self.label_ids.len())
            );
            for l in &self.label_ids {
                params.push(Value::Text(l.to_string()));
            }
        }

        let dir = if self.reverse { "DESC" } else { "ASC" };
        sql.push(' ');
        match self.sort {
            SortBy::Manual => {
                let _ = write!(sql, "ORDER BY sort_key {dir}");
            }
            SortBy::Priority => {
                let _ = write!(
                    sql,
                    "ORDER BY CASE priority \
                     WHEN 'urgent' THEN 0 WHEN 'high' THEN 1 WHEN 'medium' THEN 2 \
                     WHEN 'low' THEN 3 ELSE 4 END {dir}, created_at {dir}"
                );
            }
            SortBy::Created => {
                let _ = write!(sql, "ORDER BY created_at {dir}");
            }
            SortBy::Updated => {
                let _ = write!(sql, "ORDER BY updated_at {dir}");
            }
            SortBy::Due => {
                let _ = write!(sql, "ORDER BY due_date IS NULL, due_date {dir}");
            }
        }
        if let Some(n) = self.limit {
            sql.push_str(" LIMIT ?");
            params.push(Value::Integer(n));
        }
        (sql, params)
    }
}

fn placeholders(n: usize) -> String {
    std::iter::repeat_n("?", n).collect::<Vec<_>>().join(",")
}
