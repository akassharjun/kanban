use crate::state::AppState;
use crate::db::compat::jsonb_cast;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct AgentRegistryEntry {
    pub id: i64,
    pub agent_id: String,
    pub name: String,
    pub description: Option<String>,
    pub provider: Option<String>,
    pub version: Option<String>,
    pub endpoint: Option<String>,
    pub capabilities: String,
    pub max_concurrent: i64,
    pub max_complexity: String,
    pub hourly_rate: Option<f64>,
    pub rating: Option<f64>,
    pub total_tasks: i64,
    pub registered_at: String,
    pub last_seen_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct AgentCapability {
    pub id: i64,
    pub agent_id: String,
    pub capability: String,
    pub proficiency: f64,
    pub tasks_completed: i64,
    pub tasks_failed: i64,
    pub avg_confidence: Option<f64>,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMatch {
    pub agent_id: String,
    pub name: String,
    pub score: f64,
    pub matched_skills: Vec<String>,
    pub avg_proficiency: f64,
    pub rating: Option<f64>,
    pub status: String,
}

#[derive(Debug, Deserialize)]
pub struct MarketplaceRegisterInput {
    pub agent_id: String,
    pub name: String,
    pub description: Option<String>,
    pub provider: Option<String>,
    pub version: Option<String>,
    pub endpoint: Option<String>,
    pub capabilities: Vec<String>,
    pub max_concurrent: Option<i64>,
    pub max_complexity: Option<String>,
    pub hourly_rate: Option<f64>,
}

#[derive(Debug, Deserialize)]
pub struct MarketplaceUpdateInput {
    pub name: Option<String>,
    pub description: Option<String>,
    pub version: Option<String>,
    pub endpoint: Option<String>,
    pub capabilities: Option<Vec<String>>,
    pub max_concurrent: Option<i64>,
    pub max_complexity: Option<String>,
    pub hourly_rate: Option<f64>,
}

#[tauri::command]
pub fn marketplace_register(app: tauri::AppHandle, state: State<AppState>, input: MarketplaceRegisterInput) -> Result<AgentRegistryEntry, String> {
    use tauri::Emitter;
    state.rt.block_on(async {
        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%SZ").to_string();
        let caps_json = serde_json::to_string(&input.capabilities).unwrap_or_else(|_| "[]".to_string());

        let jb = jsonb_cast(&state.backend);
        sqlx::query(&format!(
            "INSERT INTO agent_registry (agent_id, name, description, provider, version, endpoint, capabilities, max_concurrent, max_complexity, hourly_rate, registered_at, last_seen_at) VALUES ($1, $2, $3, $4, $5, $6, $7{jb}, $8, $9, $10, $11, $12) ON CONFLICT(agent_id) DO UPDATE SET name=$2, description=$3, provider=$4, version=$5, endpoint=$6, capabilities=$7{jb}, max_concurrent=$8, max_complexity=$9, hourly_rate=$10, last_seen_at=$12"
        ))
        .bind(&input.agent_id)
        .bind(&input.name)
        .bind(&input.description)
        .bind(&input.provider)
        .bind(&input.version)
        .bind(&input.endpoint)
        .bind(&caps_json)
        .bind(input.max_concurrent.unwrap_or(1))
        .bind(input.max_complexity.as_deref().unwrap_or("medium"))
        .bind(input.hourly_rate)
        .bind(&now)
        .bind(&now)
        .execute(&state.pool)
        .await
        .map_err(|e| e.to_string())?;

        // Initialize capabilities
        for cap in &input.capabilities {
            let _ = sqlx::query(
                "INSERT INTO agent_capabilities (agent_id, capability) VALUES ($1, $2) ON CONFLICT(agent_id, capability) DO NOTHING"
            )
            .bind(&input.agent_id)
            .bind(cap)
            .execute(&state.pool)
            .await;
        }

        let entry = sqlx::query_as::<_, AgentRegistryEntry>(
            "SELECT * FROM agent_registry WHERE agent_id = $1"
        )
        .bind(&input.agent_id)
        .fetch_one(&state.pool)
        .await
        .map_err(|e| e.to_string())?;

        let _ = app.emit("db-changed", ());
        Ok(entry)
    }).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn marketplace_update(app: tauri::AppHandle, state: State<AppState>, agent_id: String, input: MarketplaceUpdateInput) -> Result<AgentRegistryEntry, String> {
    use tauri::Emitter;
    state.rt.block_on(async {
        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%SZ").to_string();

        let mut qb = sqlx::QueryBuilder::new("UPDATE agent_registry SET last_seen_at = ");
        qb.push_bind(&now);

        if let Some(ref name) = input.name {
            qb.push(", name = "); qb.push_bind(name);
        }
        if let Some(ref desc) = input.description {
            qb.push(", description = "); qb.push_bind(desc);
        }
        if let Some(ref ver) = input.version {
            qb.push(", version = "); qb.push_bind(ver);
        }
        if let Some(ref ep) = input.endpoint {
            qb.push(", endpoint = "); qb.push_bind(ep);
        }
        if let Some(ref caps) = input.capabilities {
            let caps_json = serde_json::to_string(caps).unwrap_or_else(|_| "[]".to_string());
            qb.push(", capabilities = "); qb.push_bind(caps_json);
        }
        if let Some(mc) = input.max_concurrent {
            qb.push(", max_concurrent = "); qb.push_bind(mc);
        }
        if let Some(ref mx) = input.max_complexity {
            qb.push(", max_complexity = "); qb.push_bind(mx);
        }
        if let Some(hr) = input.hourly_rate {
            qb.push(", hourly_rate = "); qb.push_bind(hr);
        }

        qb.push(" WHERE agent_id = "); qb.push_bind(&agent_id);
        qb.build().execute(&state.pool).await.map_err(|e| e.to_string())?;

        let entry = sqlx::query_as::<_, AgentRegistryEntry>(
            "SELECT * FROM agent_registry WHERE agent_id = $1"
        )
        .bind(&agent_id)
        .fetch_one(&state.pool)
        .await
        .map_err(|e| e.to_string())?;

        let _ = app.emit("db-changed", ());
        Ok(entry)
    }).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn marketplace_deregister(app: tauri::AppHandle, state: State<AppState>, agent_id: String) -> Result<(), String> {
    use tauri::Emitter;
    state.rt.block_on(async {
        sqlx::query("DELETE FROM agent_capabilities WHERE agent_id = $1")
            .bind(&agent_id)
            .execute(&state.pool)
            .await
            .map_err(|e| e.to_string())?;
        sqlx::query("DELETE FROM agent_registry WHERE agent_id = $1")
            .bind(&agent_id)
            .execute(&state.pool)
            .await
            .map_err(|e| e.to_string())?;
        let _ = app.emit("db-changed", ());
        Ok(())
    }).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn marketplace_list(state: State<AppState>) -> Result<Vec<AgentRegistryEntry>, String> {
    state.rt.block_on(async {
        sqlx::query_as::<_, AgentRegistryEntry>(
            "SELECT * FROM agent_registry ORDER BY rating DESC NULLS LAST, total_tasks DESC"
        )
        .fetch_all(&state.pool)
        .await
    }).map_err(|e: sqlx::Error| e.to_string())
}

#[tauri::command]
pub fn marketplace_search(state: State<AppState>, skills: Vec<String>, max_complexity: Option<String>) -> Result<Vec<AgentRegistryEntry>, String> {
    state.rt.block_on(async {
        // Get all entries, filter by capabilities match
        let all = sqlx::query_as::<_, AgentRegistryEntry>(
            "SELECT * FROM agent_registry ORDER BY rating DESC NULLS LAST"
        )
        .fetch_all(&state.pool)
        .await
        .map_err(|e| e.to_string())?;

        let complexity_order = |c: &str| match c {
            "small" => 1, "medium" => 2, "large" => 3, _ => 2
        };

        let max_cx = max_complexity.as_deref().unwrap_or("large");
        let max_cx_val = complexity_order(max_cx);

        let results: Vec<AgentRegistryEntry> = all.into_iter().filter(|entry| {
            // Check complexity
            if complexity_order(&entry.max_complexity) < max_cx_val {
                return false;
            }
            // Check if any skill matches capabilities
            if skills.is_empty() { return true; }
            let caps: Vec<String> = serde_json::from_str(&entry.capabilities).unwrap_or_default();
            skills.iter().any(|s| caps.iter().any(|c| c.contains(s) || s.contains(c)))
        }).collect();

        Ok(results)
    }).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn marketplace_get(state: State<AppState>, agent_id: String) -> Result<AgentRegistryEntry, String> {
    state.rt.block_on(async {
        sqlx::query_as::<_, AgentRegistryEntry>(
            "SELECT * FROM agent_registry WHERE agent_id = $1"
        )
        .bind(&agent_id)
        .fetch_one(&state.pool)
        .await
    }).map_err(|e: sqlx::Error| e.to_string())
}

#[tauri::command]
pub fn update_agent_proficiency(state: State<AppState>, agent_id: String, capability: String, success: bool) -> Result<(), String> {
    state.rt.block_on(async {
        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%SZ").to_string();

        if success {
            sqlx::query(
                "INSERT INTO agent_capabilities (agent_id, capability, proficiency, tasks_completed, tasks_failed, updated_at) VALUES ($1, $2, 0.6, 1, 0, $3) ON CONFLICT(agent_id, capability) DO UPDATE SET tasks_completed = tasks_completed + 1, proficiency = MIN(1.0, proficiency + 0.05), updated_at = $3"
            )
            .bind(&agent_id).bind(&capability).bind(&now)
            .execute(&state.pool).await.map_err(|e| e.to_string())?;
        } else {
            sqlx::query(
                "INSERT INTO agent_capabilities (agent_id, capability, proficiency, tasks_completed, tasks_failed, updated_at) VALUES ($1, $2, 0.4, 0, 1, $3) ON CONFLICT(agent_id, capability) DO UPDATE SET tasks_failed = tasks_failed + 1, proficiency = MAX(0.0, proficiency - 0.1), updated_at = $3"
            )
            .bind(&agent_id).bind(&capability).bind(&now)
            .execute(&state.pool).await.map_err(|e| e.to_string())?;
        }

        // Update registry rating
        let avg_prof: f64 = sqlx::query_scalar(
            "SELECT COALESCE(AVG(proficiency), 0.0) FROM agent_capabilities WHERE agent_id = $1"
        ).bind(&agent_id).fetch_one(&state.pool).await.map_err(|e| e.to_string())?;

        sqlx::query("UPDATE agent_registry SET rating = $1 WHERE agent_id = $2")
            .bind(avg_prof).bind(&agent_id).execute(&state.pool).await.map_err(|e| e.to_string())?;

        Ok(())
    }).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_agent_capabilities(state: State<AppState>, agent_id: String) -> Result<Vec<AgentCapability>, String> {
    state.rt.block_on(async {
        sqlx::query_as::<_, AgentCapability>(
            "SELECT * FROM agent_capabilities WHERE agent_id = $1 ORDER BY proficiency DESC"
        )
        .bind(&agent_id)
        .fetch_all(&state.pool)
        .await
    }).map_err(|e: sqlx::Error| e.to_string())
}

#[tauri::command]
pub fn find_best_agent(state: State<AppState>, task_skills: Vec<String>, complexity: String) -> Result<Vec<AgentMatch>, String> {
    state.rt.block_on(async {
        // Get all registered agents
        let entries = sqlx::query_as::<_, AgentRegistryEntry>(
            "SELECT * FROM agent_registry"
        ).fetch_all(&state.pool).await.map_err(|e| e.to_string())?;

        let complexity_order = |c: &str| match c {
            "small" => 1, "medium" => 2, "large" => 3, _ => 2
        };
        let target_cx = complexity_order(&complexity);

        let mut matches: Vec<AgentMatch> = Vec::new();

        for entry in entries {
            // Check complexity
            if complexity_order(&entry.max_complexity) < target_cx {
                continue;
            }

            // Get capabilities
            let caps = sqlx::query_as::<_, AgentCapability>(
                "SELECT * FROM agent_capabilities WHERE agent_id = $1"
            ).bind(&entry.agent_id).fetch_all(&state.pool).await.map_err(|e| e.to_string())?;

            // Match skills
            let mut matched_skills = Vec::new();
            let mut total_prof = 0.0;
            for skill in &task_skills {
                if let Some(cap) = caps.iter().find(|c| c.capability.contains(skill) || skill.contains(&c.capability)) {
                    matched_skills.push(skill.clone());
                    total_prof += cap.proficiency;
                }
            }

            if matched_skills.is_empty() && !task_skills.is_empty() {
                continue;
            }

            let avg_proficiency = if matched_skills.is_empty() { 0.5 } else { total_prof / matched_skills.len() as f64 };
            let skill_match_ratio = if task_skills.is_empty() { 1.0 } else { matched_skills.len() as f64 / task_skills.len() as f64 };

            // Check agent online status
            let status: String = sqlx::query_scalar("SELECT COALESCE(status, 'offline') FROM agents WHERE id = $1")
                .bind(&entry.agent_id)
                .fetch_optional(&state.pool)
                .await
                .map_err(|e| e.to_string())?
                .unwrap_or_else(|| "unknown".to_string());

            let availability_score = match status.as_str() {
                "idle" => 1.0,
                "busy" => 0.3,
                "offline" => 0.0,
                _ => 0.5,
            };

            let rating = entry.rating.unwrap_or(0.5);
            let score = skill_match_ratio * 0.35 + avg_proficiency * 0.25 + availability_score * 0.2 + rating * 0.2;

            matches.push(AgentMatch {
                agent_id: entry.agent_id,
                name: entry.name,
                score,
                matched_skills,
                avg_proficiency,
                rating: entry.rating,
                status,
            });
        }

        matches.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        Ok(matches)
    }).map_err(|e| e.to_string())
}
