use std::time::{SystemTime, UNIX_EPOCH};

use rusqlite::params;
use serde::Serialize;
use tauri::{AppHandle, Manager, State};
use uuid::Uuid;

use crate::database::DatabaseState;

use super::{execution::ExecutionTarget, provider::Usage};

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AiUsageItem {
    pub task: String,
    pub provider_id: String,
    pub model: String,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub cached_tokens: i64,
    pub requests: i64,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AiUsageSummary {
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub cached_tokens: i64,
    pub requests: i64,
    pub items: Vec<AiUsageItem>,
}

pub fn record(app: &AppHandle, task: &str, target: &ExecutionTarget, usage: &Usage) {
    if usage.input_tokens == 0 && usage.output_tokens == 0 && usage.cached_tokens.unwrap_or(0) == 0 {
        return;
    }
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis().min(i64::MAX as u128) as i64)
        .unwrap_or_default();
    let state = app.state::<DatabaseState>();
    let _ = state.with_connection(|connection| {
        connection.execute(
            "INSERT INTO ai_usage(id, task, provider_id, model, input_tokens, output_tokens, cached_tokens, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                Uuid::new_v4().to_string(),
                task,
                target.provider_id,
                target.model,
                usage.input_tokens as i64,
                usage.output_tokens as i64,
                usage.cached_tokens.unwrap_or(0) as i64,
                now,
            ],
        )
        .map_err(|error| format!("failed to record AI usage: {error}"))?;
        Ok(())
    });
}

#[tauri::command]
pub fn ai_usage_summary(
    state: State<'_, DatabaseState>,
    days: Option<u32>,
) -> Result<AiUsageSummary, String> {
    let days = days.unwrap_or(30).clamp(1, 365) as i64;
    let cutoff = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis().min(i64::MAX as u128) as i64)
        .unwrap_or_default()
        .saturating_sub(days * 86_400_000);
    state.with_connection(|connection| {
        let (input_tokens, output_tokens, cached_tokens, requests) = connection
            .query_row(
                "SELECT COALESCE(SUM(input_tokens),0), COALESCE(SUM(output_tokens),0), COALESCE(SUM(cached_tokens),0), COUNT(*)
                 FROM ai_usage WHERE created_at >= ?1",
                [cutoff],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .map_err(|error| format!("failed to summarize AI usage: {error}"))?;
        let mut statement = connection
            .prepare(
                "SELECT task, provider_id, model, SUM(input_tokens), SUM(output_tokens), SUM(cached_tokens), COUNT(*)
                 FROM ai_usage WHERE created_at >= ?1
                 GROUP BY task, provider_id, model
                 ORDER BY SUM(input_tokens + output_tokens) DESC, task ASC",
            )
            .map_err(|error| format!("failed to prepare AI usage breakdown: {error}"))?;
        let items = statement
            .query_map([cutoff], |row| {
                Ok(AiUsageItem {
                    task: row.get(0)?,
                    provider_id: row.get(1)?,
                    model: row.get(2)?,
                    input_tokens: row.get(3)?,
                    output_tokens: row.get(4)?,
                    cached_tokens: row.get(5)?,
                    requests: row.get(6)?,
                })
            })
            .map_err(|error| format!("failed to query AI usage breakdown: {error}"))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| format!("failed to read AI usage breakdown: {error}"))?;
        Ok(AiUsageSummary { input_tokens, output_tokens, cached_tokens, requests, items })
    })
}
