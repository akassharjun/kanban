use crate::state::AppState;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct TaskCost {
    pub id: i64,
    pub task_identifier: String,
    pub agent_id: String,
    pub cost_type: String,
    pub amount: f64,
    pub unit: String,
    pub description: Option<String>,
    pub recorded_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskCostSummary {
    pub task_identifier: String,
    pub total_compute_minutes: f64,
    pub total_tokens: i64,
    pub total_cost_dollars: f64,
    pub cost_breakdown: Vec<TaskCost>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentCostEntry {
    pub agent_id: String,
    pub agent_name: String,
    pub total_cost: f64,
    pub task_count: i64,
    pub avg_cost_per_task: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyCostEntry {
    pub date: String,
    pub cost: f64,
    pub task_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetStatus {
    pub budget_id: i64,
    pub budget_type: String,
    pub amount: f64,
    pub unit: String,
    pub spent: f64,
    pub percentage: f64,
    pub alert: bool,
    pub alert_threshold: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectCostSummary {
    pub project_id: i64,
    pub total_cost: f64,
    pub cost_by_agent: Vec<AgentCostEntry>,
    pub daily_costs: Vec<DailyCostEntry>,
    pub budget_status: Vec<BudgetStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct CostBudget {
    pub id: i64,
    pub project_id: i64,
    pub budget_type: String,
    pub amount: f64,
    pub unit: String,
    pub spent: f64,
    pub period_start: Option<String>,
    pub period_end: Option<String>,
    pub alert_threshold: Option<f64>,
    pub created_at: String,
}

#[derive(Deserialize)]
pub struct RecordCostInput {
    pub task_identifier: String,
    pub agent_id: String,
    pub cost_type: String,
    pub amount: f64,
    pub unit: String,
    pub description: Option<String>,
}

#[derive(Deserialize)]
pub struct SetBudgetInput {
    pub project_id: i64,
    pub budget_type: String,
    pub amount: f64,
    pub unit: Option<String>,
    pub alert_threshold: Option<f64>,
}

#[tauri::command]
pub fn record_cost(state: State<AppState>, input: RecordCostInput) -> Result<TaskCost, String> {
    state.rt.block_on(async {
        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%SZ").to_string();
        let id: i64 = sqlx::query_scalar(
            "INSERT INTO task_costs (task_identifier, agent_id, cost_type, amount, unit, description, recorded_at) VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING id"
        )
        .bind(&input.task_identifier)
        .bind(&input.agent_id)
        .bind(&input.cost_type)
        .bind(input.amount)
        .bind(&input.unit)
        .bind(&input.description)
        .bind(&now)
        .fetch_one(&state.pool)
        .await?;

        // Update budget spent amounts for the project that owns this task
        let project_id: Option<i64> = sqlx::query_scalar(
            "SELECT i.project_id FROM issues i WHERE i.identifier = $1"
        )
        .bind(&input.task_identifier)
        .fetch_optional(&state.pool)
        .await?;

        if let Some(pid) = project_id {
            if input.unit == "dollars" {
                sqlx::query("UPDATE cost_budgets SET spent = spent + $1 WHERE project_id = $2")
                    .bind(input.amount)
                    .bind(pid)
                    .execute(&state.pool)
                    .await?;
            }
        }

        sqlx::query_as::<_, TaskCost>("SELECT * FROM task_costs WHERE id = $1")
            .bind(id)
            .fetch_one(&state.pool)
            .await
    }).map_err(|e: sqlx::Error| e.to_string())
}

#[tauri::command]
pub fn get_task_cost_summary(state: State<AppState>, task_identifier: String) -> Result<TaskCostSummary, String> {
    state.rt.block_on(async {
        let costs = sqlx::query_as::<_, TaskCost>(
            "SELECT * FROM task_costs WHERE task_identifier = $1 ORDER BY recorded_at ASC"
        )
        .bind(&task_identifier)
        .fetch_all(&state.pool)
        .await?;

        let mut total_compute_minutes = 0.0f64;
        let mut total_tokens = 0i64;
        let mut total_cost_dollars = 0.0f64;

        for c in &costs {
            match c.cost_type.as_str() {
                "compute_time" => {
                    if c.unit == "minutes" {
                        total_compute_minutes += c.amount;
                    }
                }
                "api_tokens" => {
                    if c.unit == "tokens" {
                        total_tokens += c.amount as i64;
                    }
                }
                _ => {}
            }
            if c.unit == "dollars" {
                total_cost_dollars += c.amount;
            }
        }

        Ok(TaskCostSummary {
            task_identifier,
            total_compute_minutes,
            total_tokens,
            total_cost_dollars,
            cost_breakdown: costs,
        })
    }).map_err(|e: sqlx::Error| e.to_string())
}

#[tauri::command]
pub fn get_project_cost_summary(state: State<AppState>, project_id: i64) -> Result<ProjectCostSummary, String> {
    state.rt.block_on(async {
        // Total cost for the project
        let total_cost: Option<f64> = sqlx::query_scalar(
            "SELECT SUM(tc.amount) FROM task_costs tc JOIN issues i ON tc.task_identifier = i.identifier WHERE i.project_id = $1 AND tc.unit = 'dollars'"
        )
        .bind(project_id)
        .fetch_one(&state.pool)
        .await?;

        // Cost by agent
        let agent_rows = sqlx::query_as::<_, (String, f64, i64)>(
            "SELECT tc.agent_id, SUM(tc.amount) as total, COUNT(DISTINCT tc.task_identifier) as task_count FROM task_costs tc JOIN issues i ON tc.task_identifier = i.identifier WHERE i.project_id = $1 AND tc.unit = 'dollars' GROUP BY tc.agent_id"
        )
        .bind(project_id)
        .fetch_all(&state.pool)
        .await
        .unwrap_or_default();

        let mut cost_by_agent = Vec::new();
        for (agent_id, total, task_count) in agent_rows {
            let agent_name: Option<String> = sqlx::query_scalar(
                "SELECT name FROM agents WHERE id = $1"
            )
            .bind(&agent_id)
            .fetch_optional(&state.pool)
            .await
            .unwrap_or(None);

            let avg = if task_count > 0 { total / task_count as f64 } else { 0.0 };
            cost_by_agent.push(AgentCostEntry {
                agent_id,
                agent_name: agent_name.unwrap_or_else(|| "Unknown".to_string()),
                total_cost: total,
                task_count,
                avg_cost_per_task: avg,
            });
        }

        // Daily costs (last 30 days)
        let daily_rows = sqlx::query_as::<_, (String, f64, i64)>(
            "SELECT DATE(tc.recorded_at) as d, SUM(tc.amount), COUNT(DISTINCT tc.task_identifier) FROM task_costs tc JOIN issues i ON tc.task_identifier = i.identifier WHERE i.project_id = $1 AND tc.unit = 'dollars' AND tc.recorded_at >= datetime('now', '-30 days') GROUP BY d ORDER BY d"
        )
        .bind(project_id)
        .fetch_all(&state.pool)
        .await
        .unwrap_or_default();

        let daily_costs: Vec<DailyCostEntry> = daily_rows.into_iter().map(|(date, cost, task_count)| {
            DailyCostEntry { date, cost, task_count }
        }).collect();

        // Budget status
        let budgets = sqlx::query_as::<_, CostBudget>(
            "SELECT * FROM cost_budgets WHERE project_id = $1"
        )
        .bind(project_id)
        .fetch_all(&state.pool)
        .await
        .unwrap_or_default();

        let budget_status: Vec<BudgetStatus> = budgets.iter().map(|b| {
            let percentage = if b.amount > 0.0 { b.spent / b.amount } else { 0.0 };
            let threshold = b.alert_threshold.unwrap_or(0.8);
            BudgetStatus {
                budget_id: b.id,
                budget_type: b.budget_type.clone(),
                amount: b.amount,
                unit: b.unit.clone(),
                spent: b.spent,
                percentage,
                alert: percentage >= threshold,
                alert_threshold: threshold,
            }
        }).collect();

        Ok(ProjectCostSummary {
            project_id,
            total_cost: total_cost.unwrap_or(0.0),
            cost_by_agent,
            daily_costs,
            budget_status,
        })
    }).map_err(|e: sqlx::Error| e.to_string())
}

#[tauri::command]
pub fn set_budget(state: State<AppState>, input: SetBudgetInput) -> Result<CostBudget, String> {
    state.rt.block_on(async {
        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%SZ").to_string();
        let unit = input.unit.unwrap_or_else(|| "dollars".to_string());
        let threshold = input.alert_threshold.unwrap_or(0.8);

        let id: i64 = sqlx::query_scalar(
            "INSERT INTO cost_budgets (project_id, budget_type, amount, unit, alert_threshold, created_at) VALUES ($1, $2, $3, $4, $5, $6) RETURNING id"
        )
        .bind(input.project_id)
        .bind(&input.budget_type)
        .bind(input.amount)
        .bind(&unit)
        .bind(threshold)
        .bind(&now)
        .fetch_one(&state.pool)
        .await?;

        sqlx::query_as::<_, CostBudget>("SELECT * FROM cost_budgets WHERE id = $1")
            .bind(id)
            .fetch_one(&state.pool)
            .await
    }).map_err(|e: sqlx::Error| e.to_string())
}

#[tauri::command]
pub fn list_budgets(state: State<AppState>, project_id: i64) -> Result<Vec<BudgetStatus>, String> {
    state.rt.block_on(async {
        let budgets = sqlx::query_as::<_, CostBudget>(
            "SELECT * FROM cost_budgets WHERE project_id = $1"
        )
        .bind(project_id)
        .fetch_all(&state.pool)
        .await?;

        Ok(budgets.iter().map(|b| {
            let percentage = if b.amount > 0.0 { b.spent / b.amount } else { 0.0 };
            let threshold = b.alert_threshold.unwrap_or(0.8);
            BudgetStatus {
                budget_id: b.id,
                budget_type: b.budget_type.clone(),
                amount: b.amount,
                unit: b.unit.clone(),
                spent: b.spent,
                percentage,
                alert: percentage >= threshold,
                alert_threshold: threshold,
            }
        }).collect())
    }).map_err(|e: sqlx::Error| e.to_string())
}

#[tauri::command]
pub fn check_budget(state: State<AppState>, project_id: i64) -> Result<Vec<BudgetStatus>, String> {
    state.rt.block_on(async {
        let budgets = sqlx::query_as::<_, CostBudget>(
            "SELECT * FROM cost_budgets WHERE project_id = $1"
        )
        .bind(project_id)
        .fetch_all(&state.pool)
        .await?;

        Ok(budgets.iter().filter_map(|b| {
            let percentage = if b.amount > 0.0 { b.spent / b.amount } else { 0.0 };
            let threshold = b.alert_threshold.unwrap_or(0.8);
            if percentage >= threshold {
                Some(BudgetStatus {
                    budget_id: b.id,
                    budget_type: b.budget_type.clone(),
                    amount: b.amount,
                    unit: b.unit.clone(),
                    spent: b.spent,
                    percentage,
                    alert: true,
                    alert_threshold: threshold,
                })
            } else {
                None
            }
        }).collect())
    }).map_err(|e: sqlx::Error| e.to_string())
}

#[tauri::command]
pub fn delete_budget(state: State<AppState>, id: i64) -> Result<(), String> {
    state.rt.block_on(async {
        let result = sqlx::query("DELETE FROM cost_budgets WHERE id = $1")
            .bind(id)
            .execute(&state.pool)
            .await?;
        if result.rows_affected() == 0 {
            return Err(sqlx::Error::RowNotFound);
        }
        Ok(())
    }).map_err(|e: sqlx::Error| e.to_string())
}
