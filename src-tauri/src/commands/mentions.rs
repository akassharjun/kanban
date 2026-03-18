use crate::state::AppState;
use crate::models::{Mention, MentionWithContext};
use tauri::State;
use serde::Deserialize;

/// Parse @username mentions from text, returning matched member IDs.
pub async fn parse_mentions(pool: &sqlx::AnyPool, text: &str) -> Result<Vec<(i64, String)>, sqlx::Error> {
    let mut found = Vec::new();
    let re_pattern: Vec<&str> = text.split('@').skip(1).collect();

    for chunk in re_pattern {
        // Extract the potential username (alphanumeric, hyphens, underscores)
        let username: String = chunk.chars()
            .take_while(|c| c.is_alphanumeric() || *c == '-' || *c == '_')
            .collect();
        if username.is_empty() {
            continue;
        }

        // Look up member by name (case-insensitive)
        let member: Option<(i64, String)> = sqlx::query_as(
            "SELECT id, name FROM members WHERE LOWER(name) = LOWER($1)"
        )
        .bind(&username)
        .fetch_optional(pool)
        .await?;

        if let Some(m) = member {
            if !found.iter().any(|(id, _)| *id == m.0) {
                found.push(m);
            }
        }
    }

    Ok(found)
}

/// Insert mentions and create notifications for each mentioned member.
pub async fn process_mentions(
    pool: &sqlx::AnyPool,
    issue_id: i64,
    comment_id: Option<i64>,
    source: &str,
    text: &str,
) -> Result<(), sqlx::Error> {
    let mentioned = parse_mentions(pool, text).await?;
    let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%SZ").to_string();

    // Get issue identifier for notification message
    let identifier: String = sqlx::query_scalar("SELECT identifier FROM issues WHERE id = $1")
        .bind(issue_id)
        .fetch_one(pool)
        .await?;

    for (member_id, _member_name) in &mentioned {
        // Insert mention
        sqlx::query(
            "INSERT INTO mentions (issue_id, comment_id, member_id, source, created_at) VALUES ($1, $2, $3, $4, $5)"
        )
        .bind(issue_id)
        .bind(comment_id)
        .bind(member_id)
        .bind(source)
        .bind(&now)
        .execute(pool)
        .await?;

        // Create notification
        let message = if source == "comment" {
            format!("You were mentioned in a comment on {}", identifier)
        } else {
            format!("You were mentioned in {}", identifier)
        };

        sqlx::query(
            "INSERT INTO notifications (type, issue_id, message, read, created_at) VALUES ($1, $2, $3, FALSE, $4)"
        )
        .bind("mention")
        .bind(issue_id)
        .bind(&message)
        .bind(&now)
        .execute(pool)
        .await?;
    }

    Ok(())
}

/// Remove old mentions for a given source (before re-processing on update).
pub async fn clear_mentions(
    pool: &sqlx::AnyPool,
    issue_id: i64,
    comment_id: Option<i64>,
    source: &str,
) -> Result<(), sqlx::Error> {
    if let Some(cid) = comment_id {
        sqlx::query("DELETE FROM mentions WHERE issue_id = $1 AND comment_id = $2 AND source = $3")
            .bind(issue_id).bind(cid).bind(source)
            .execute(pool).await?;
    } else {
        sqlx::query("DELETE FROM mentions WHERE issue_id = $1 AND comment_id IS NULL AND source = $2")
            .bind(issue_id).bind(source)
            .execute(pool).await?;
    }
    Ok(())
}

#[tauri::command]
pub fn list_mentions(state: State<AppState>, member_id: i64) -> Result<Vec<MentionWithContext>, String> {
    state.rt.block_on(async {
        let rows = sqlx::query_as::<_, MentionRow>(
            "SELECT mn.id, mn.issue_id, i.identifier as issue_identifier, i.title as issue_title, \
             mn.comment_id, mn.member_id, mn.source, mn.created_at \
             FROM mentions mn \
             JOIN issues i ON mn.issue_id = i.id \
             WHERE mn.member_id = $1 \
             ORDER BY mn.created_at DESC \
             LIMIT 100"
        )
        .bind(member_id)
        .fetch_all(&state.pool)
        .await?;

        Ok(rows.into_iter().map(|r| MentionWithContext {
            id: r.id,
            issue_id: r.issue_id,
            issue_identifier: r.issue_identifier,
            issue_title: r.issue_title,
            comment_id: r.comment_id,
            member_id: r.member_id,
            source: r.source,
            created_at: r.created_at,
        }).collect())
    }).map_err(|e: sqlx::Error| e.to_string())
}

#[tauri::command]
pub fn search_members_for_mention(state: State<AppState>, query: String) -> Result<Vec<crate::models::Member>, String> {
    state.rt.block_on(async {
        let pattern = format!("%{}%", query);
        sqlx::query_as::<_, crate::models::Member>(
            "SELECT * FROM members WHERE LOWER(name) LIKE LOWER($1) OR LOWER(display_name) LIKE LOWER($2) ORDER BY name LIMIT 10"
        )
        .bind(&pattern)
        .bind(&pattern)
        .fetch_all(&state.pool)
        .await
    }).map_err(|e| e.to_string())
}

#[derive(sqlx::FromRow)]
struct MentionRow {
    id: i64,
    issue_id: i64,
    issue_identifier: String,
    issue_title: String,
    comment_id: Option<i64>,
    member_id: i64,
    source: String,
    created_at: String,
}
