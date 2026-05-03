use crate::error::Result;
use crate::query::IssueFilter;
use crate::store::read::issues::{ISSUE_LIST_BASE_QUALIFIED, row_to_issue};
use crate::types::Issue;
use rusqlite::Connection;

pub(crate) fn search(
    conn: &Connection,
    query: &str,
    mut filter: IssueFilter,
) -> Result<Vec<Issue>> {
    filter.search_text = Some(query.to_string());
    let (sql, params) = filter.build_sql_with_search(ISSUE_LIST_BASE_QUALIFIED);
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(rusqlite::params_from_iter(params.iter()), row_to_issue)?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}
