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
        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%SZ").to_string();
        let color = input.avatar_color.unwrap_or_else(|| {
            let colors = ["#6366f1", "#ec4899", "#f59e0b", "#10b981", "#3b82f6", "#8b5cf6", "#ef4444", "#14b8a6"];
            colors[rand_color_index(&input.name) % colors.len()].to_string()
        });

        let id: i64 = sqlx::query_scalar(
            "INSERT INTO members (name, display_name, email, avatar_color, created_at) VALUES ($1, $2, $3, $4, $5) RETURNING id"
        )
        .bind(&input.name).bind(&input.display_name).bind(&input.email).bind(&color).bind(&now)
        .fetch_one(&state.pool).await?;

        sqlx::query_as::<_, Member>("SELECT * FROM members WHERE id = $1")
            .bind(id).fetch_one(&state.pool).await
    }).map_err(|e: sqlx::Error| e.to_string())
}

fn rand_color_index(name: &str) -> usize {
    name.bytes().fold(0usize, |acc, b| acc.wrapping_add(b as usize))
}

#[tauri::command]
pub fn update_member(state: State<AppState>, id: i64, input: UpdateMemberInput) -> Result<Member, String> {
    state.rt.block_on(async {
        if let Some(ref name) = input.name {
            sqlx::query("UPDATE members SET name = $1 WHERE id = $2")
                .bind(name).bind(id).execute(&state.pool).await?;
        }
        if let Some(ref display_name) = input.display_name {
            sqlx::query("UPDATE members SET display_name = $1 WHERE id = $2")
                .bind(display_name).bind(id).execute(&state.pool).await?;
        }
        if let Some(ref email) = input.email {
            sqlx::query("UPDATE members SET email = $1 WHERE id = $2")
                .bind(email).bind(id).execute(&state.pool).await?;
        }
        if let Some(ref color) = input.avatar_color {
            sqlx::query("UPDATE members SET avatar_color = $1 WHERE id = $2")
                .bind(color).bind(id).execute(&state.pool).await?;
        }
        sqlx::query_as::<_, Member>("SELECT * FROM members WHERE id = $1")
            .bind(id).fetch_one(&state.pool).await
    }).map_err(|e: sqlx::Error| e.to_string())
}

#[tauri::command]
pub fn delete_member(state: State<AppState>, id: i64) -> Result<(), String> {
    state.rt.block_on(async {
        sqlx::query("DELETE FROM members WHERE id = $1").bind(id).execute(&state.pool).await?;
        Ok(())
    }).map_err(|e: sqlx::Error| e.to_string())
}
