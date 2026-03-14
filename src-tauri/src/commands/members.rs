use crate::state::AppState;
use crate::models::Member;
use tauri::State;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct CreateMemberInput {
    pub name: String,
    pub display_name: Option<String>,
    pub email: Option<String>,
    pub avatar_color: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdateMemberInput {
    pub name: Option<String>,
    pub display_name: Option<String>,
    pub email: Option<String>,
    pub avatar_color: Option<String>,
}

#[tauri::command]
pub fn list_members(state: State<AppState>) -> Result<Vec<Member>, String> {
    state.rt.block_on(async {
        sqlx::query_as::<_, Member>("SELECT * FROM members ORDER BY name")
            .fetch_all(&state.pool).await
    }).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_member(state: State<AppState>, input: CreateMemberInput) -> Result<Member, String> {
    state.rt.block_on(async {
        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let color = input.avatar_color.unwrap_or_else(|| {
            let colors = ["#6366f1", "#ec4899", "#f59e0b", "#10b981", "#3b82f6", "#8b5cf6", "#ef4444", "#14b8a6"];
            colors[rand_color_index(&input.name) % colors.len()].to_string()
        });

        let result = sqlx::query(
            "INSERT INTO members (name, display_name, email, avatar_color, created_at) VALUES (?, ?, ?, ?, ?)"
        )
        .bind(&input.name).bind(&input.display_name).bind(&input.email).bind(&color).bind(&now)
        .execute(&state.pool).await?;

        sqlx::query_as::<_, Member>("SELECT * FROM members WHERE id = ?")
            .bind(result.last_insert_rowid()).fetch_one(&state.pool).await
    }).map_err(|e: sqlx::Error| e.to_string())
}

fn rand_color_index(name: &str) -> usize {
    name.bytes().fold(0usize, |acc, b| acc.wrapping_add(b as usize))
}

#[tauri::command]
pub fn update_member(state: State<AppState>, id: i64, input: UpdateMemberInput) -> Result<Member, String> {
    state.rt.block_on(async {
        if let Some(ref name) = input.name {
            sqlx::query("UPDATE members SET name = ? WHERE id = ?")
                .bind(name).bind(id).execute(&state.pool).await?;
        }
        if let Some(ref display_name) = input.display_name {
            sqlx::query("UPDATE members SET display_name = ? WHERE id = ?")
                .bind(display_name).bind(id).execute(&state.pool).await?;
        }
        if let Some(ref email) = input.email {
            sqlx::query("UPDATE members SET email = ? WHERE id = ?")
                .bind(email).bind(id).execute(&state.pool).await?;
        }
        if let Some(ref color) = input.avatar_color {
            sqlx::query("UPDATE members SET avatar_color = ? WHERE id = ?")
                .bind(color).bind(id).execute(&state.pool).await?;
        }
        sqlx::query_as::<_, Member>("SELECT * FROM members WHERE id = ?")
            .bind(id).fetch_one(&state.pool).await
    }).map_err(|e: sqlx::Error| e.to_string())
}

#[tauri::command]
pub fn delete_member(state: State<AppState>, id: i64) -> Result<(), String> {
    state.rt.block_on(async {
        sqlx::query("DELETE FROM members WHERE id = ?").bind(id).execute(&state.pool).await?;
        Ok(())
    }).map_err(|e: sqlx::Error| e.to_string())
}
