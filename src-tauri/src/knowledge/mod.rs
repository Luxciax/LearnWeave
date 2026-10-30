use std::time::{SystemTime, UNIX_EPOCH};

use rusqlite::{params, OptionalExtension, Row};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tauri::{AppHandle, State};
use uuid::Uuid;

use crate::database::DatabaseState;

pub(crate) mod assessment;
pub(crate) mod claims;
pub(crate) mod curation;
pub(crate) mod graph;
pub(crate) mod health;
pub(crate) mod inbox;
pub(crate) mod internalization;
pub(crate) mod librarian;
pub(crate) mod librarian_prompt;
pub(crate) mod management;
pub(crate) mod merge;
pub(crate) mod review;
pub(crate) mod research;
pub(crate) mod scheduler;
pub(crate) mod semantic;
pub(crate) mod source;
pub(crate) mod task_center;
pub(crate) mod today;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceCaptureInput {
    pub platform: String,
    #[serde(default)]
    pub source_kind: Option<String>,
    #[serde(default)]
    pub connector_id: Option<String>,
    pub selected_text: String,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub author: Option<String>,
    #[serde(default)]
    pub context_before: Option<String>,
    #[serde(default)]
    pub context_after: Option<String>,
    #[serde(default)]
    pub application: Option<String>,
    #[serde(default)]
    pub window_title: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceRecord {
    pub id: String,
    pub platform: String,
    pub source_kind: String,
    pub connector_id: Option<String>,
    pub url: Option<String>,
    pub title: Option<String>,
    pub author: Option<String>,
    pub selected_text: String,
    pub context_before: Option<String>,
    pub context_after: Option<String>,
    pub application: Option<String>,
    pub window_title: Option<String>,
    pub content_hash: String,
    pub captured_at: i64,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeUnitRecord {
    pub id: String,
    pub source_id: String,
    pub core_claim: String,
    pub concepts: Vec<String>,
    pub prerequisites: Vec<String>,
    pub important_details: Vec<String>,
    pub limitations: Vec<String>,
    pub user_note: Option<String>,
    pub status: String,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CaptureKnowledgeResult {
    pub source: SourceRecord,
    pub knowledge_unit: KnowledgeUnitRecord,
    pub duplicate: bool,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeListItem {
    pub id: String,
    pub source_id: String,
    pub core_claim: String,
    pub selected_text: String,
    pub status: String,
    pub platform: String,
    pub title: Option<String>,
    pub author: Option<String>,
    pub captured_at: i64,
    pub mastery_score: i64,
    pub review_count: i64,
    pub ai_job_status: Option<String>,
    pub ai_job_error: Option<String>,
}

#[tauri::command]
pub fn knowledge_capture(
    app: AppHandle,
    state: State<'_, DatabaseState>,
    input: SourceCaptureInput,
) -> Result<CaptureKnowledgeResult, String> {
    let input = crate::browser_bridge::enrich_capture(&app, input);
    validate_capture(&input)?;
    let content_hash = capture_hash(&input)?;
    let now = now_ms();

    state.with_connection(|connection| {
        if should_upgrade_to_zhihu(&input) {
            if let Some(existing) = load_capture_by_zhihu_url(connection, &input)? {
                return Ok(CaptureKnowledgeResult {
                    duplicate: true,
                    ..existing
                });
            }
        }

        if let Some(existing) = load_capture_by_hash(connection, &content_hash)? {
            return Ok(CaptureKnowledgeResult {
                duplicate: true,
                ..existing
            });
        }

        if should_upgrade_to_zhihu(&input) {
            if let Some(existing) = load_zhihu_upgrade_candidate(connection, &input.selected_text)?
            {
                upgrade_source_to_zhihu(connection, &existing.source.id, &input, &content_hash)?;
                let upgraded =
                    load_capture_by_knowledge_id(connection, &existing.knowledge_unit.id)?
                        .ok_or_else(|| {
                            "source metadata was upgraded but could not be reloaded".to_string()
                        })?;
                return Ok(CaptureKnowledgeResult {
                    duplicate: true,
                    ..upgraded
                });
            }
        }

        let source_id = Uuid::new_v4().to_string();
        let knowledge_unit_id = Uuid::new_v4().to_string();
        let transaction = connection
            .transaction()
            .map_err(|error| format!("failed to start knowledge capture: {error}"))?;

        transaction
            .execute(
                "INSERT INTO sources(
                    id, platform, source_kind, connector_id, url, title, author, selected_text,
                    context_before, context_after, application, window_title, content_hash, captured_at
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
                params![
                    source_id,
                    normalized_platform(&input.platform),
                    normalized_source_kind(&input),
                    normalized_connector_id(&input),
                    normalized_optional(&input.url),
                    normalized_optional(&input.title),
                    normalized_optional(&input.author),
                    input.selected_text,
                    normalized_optional(&input.context_before),
                    normalized_optional(&input.context_after),
                    normalized_optional(&input.application),
                    normalized_optional(&input.window_title),
                    content_hash,
                    now,
                ],
            )
            .map_err(|error| format!("failed to save source: {error}"))?;

        transaction
            .execute(
                "INSERT INTO knowledge_units(
                    id, source_id, core_claim, status, created_at, updated_at
                ) VALUES (?1, ?2, '', 'captured', ?3, ?3)",
                params![knowledge_unit_id, source_id, now],
            )
            .map_err(|error| format!("failed to save knowledge unit: {error}"))?;

        transaction
            .execute(
                "INSERT INTO review_states(knowledge_unit_id, mastery_score) VALUES (?1, 0)",
                params![knowledge_unit_id],
            )
            .map_err(|error| format!("failed to initialize review state: {error}"))?;

        inbox::ensure_for_capture(&transaction, &source_id, &knowledge_unit_id, now)?;

        transaction
            .commit()
            .map_err(|error| format!("failed to commit knowledge capture: {error}"))?;

        load_capture_by_hash(connection, &content_hash)?
            .map(|captured| CaptureKnowledgeResult {
                duplicate: false,
                ..captured
            })
            .ok_or_else(|| "knowledge capture committed but could not be reloaded".to_string())
    })
}

#[tauri::command]
pub fn knowledge_list(
    state: State<'_, DatabaseState>,
    limit: Option<u32>,
) -> Result<Vec<KnowledgeListItem>, String> {
    let limit = limit.unwrap_or(50).clamp(1, 100) as i64;
    state.with_connection(|connection| {
        let mut statement = connection
            .prepare(
                "SELECT
                    k.id, k.source_id, k.core_claim, s.selected_text, k.status, s.platform,
                    s.title, s.author, s.captured_at, COALESCE(r.mastery_score, 0), COALESCE(r.review_count, 0),
                    (SELECT j.status FROM ai_jobs j WHERE j.knowledge_unit_id = k.id ORDER BY j.created_at DESC, j.rowid DESC LIMIT 1),
                    (SELECT j.error FROM ai_jobs j WHERE j.knowledge_unit_id = k.id ORDER BY j.created_at DESC, j.rowid DESC LIMIT 1)
                 FROM knowledge_units k
                 JOIN sources s ON s.id = k.source_id
                 LEFT JOIN review_states r ON r.knowledge_unit_id = k.id
                 WHERE k.deleted_at IS NULL AND k.archived_at IS NULL AND k.status <> 'captured'
                 ORDER BY k.updated_at DESC
                 LIMIT ?1",
            )
            .map_err(|error| format!("failed to prepare knowledge list query: {error}"))?;
        let rows = statement
            .query_map([limit], |row| {
                Ok(KnowledgeListItem {
                    id: row.get(0)?,
                    source_id: row.get(1)?,
                    core_claim: row.get(2)?,
                    selected_text: row.get(3)?,
                    status: row.get(4)?,
                    platform: row.get(5)?,
                    title: row.get(6)?,
                    author: row.get(7)?,
                    captured_at: row.get(8)?,
                    mastery_score: row.get(9)?,
                    review_count: row.get(10)?,
                    ai_job_status: row.get(11)?,
                    ai_job_error: row.get(12)?,
                })
            })
            .map_err(|error| format!("failed to query knowledge list: {error}"))?;

        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|error| format!("failed to read knowledge list: {error}"))
    })
}

#[tauri::command]
pub fn knowledge_get(
    state: State<'_, DatabaseState>,
    knowledge_unit_id: String,
) -> Result<CaptureKnowledgeResult, String> {
    state.with_connection(|connection| {
        load_capture_by_knowledge_id(connection, &knowledge_unit_id)?
            .ok_or_else(|| "knowledge unit not found".to_string())
    })
}

fn validate_capture(input: &SourceCaptureInput) -> Result<(), String> {
    if input.selected_text.trim().is_empty() {
        return Err("selected text is empty".into());
    }
    if input.selected_text.chars().count() > 200_000 {
        return Err("selected text is too large".into());
    }
    Ok(())
}

fn normalized_platform(platform: &str) -> &'static str {
    match platform.trim().to_ascii_lowercase().as_str() {
        "zhihu" => "zhihu",
        "web" => "web",
        "pdf" => "pdf",
        "desktop" => "desktop",
        "manual" => "manual",
        _ => "unknown",
    }
}

fn normalized_source_kind(input: &SourceCaptureInput) -> &'static str {
    match input.source_kind.as_deref().map(str::trim).map(str::to_ascii_lowercase).as_deref() {
        Some("web") => "web",
        Some("desktop") => "desktop",
        Some("document") => "document",
        Some("manual") => "manual",
        Some("unknown") => "unknown",
        _ => match normalized_platform(&input.platform) {
            "zhihu" | "web" => "web",
            "pdf" => "document",
            "desktop" => "desktop",
            "manual" => "manual",
            _ => "unknown",
        },
    }
}

fn normalized_connector_id(input: &SourceCaptureInput) -> Option<String> {
    normalized_optional(&input.connector_id)
        .map(|value| value.to_ascii_lowercase())
        .or_else(|| (normalized_platform(&input.platform) == "zhihu").then(|| "zhihu".to_string()))
}

fn normalized_optional(value: &Option<String>) -> Option<String> {
    value
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

fn capture_hash(input: &SourceCaptureInput) -> Result<String, String> {
    let canonical = if normalized_platform(&input.platform) == "zhihu" {
        if let Some(url) = canonical_source_url(input) {
            format!(
                "zhihu:url:{url}:text:{}",
                normalized_source_text(&input.selected_text)
            )
            .into_bytes()
        } else {
            format!("zhihu:text:{}", normalized_source_text(&input.selected_text)).into_bytes()
        }
    } else {
        #[derive(Serialize)]
        #[serde(rename_all = "camelCase")]
        struct LegacyCaptureHashInput<'a> {
            platform: &'a str,
            selected_text: &'a str,
            url: &'a Option<String>,
            title: &'a Option<String>,
            author: &'a Option<String>,
            context_before: &'a Option<String>,
            context_after: &'a Option<String>,
            application: &'a Option<String>,
            window_title: &'a Option<String>,
        }
        serde_json::to_vec(&LegacyCaptureHashInput {
            platform: &input.platform,
            selected_text: &input.selected_text,
            url: &input.url,
            title: &input.title,
            author: &input.author,
            context_before: &input.context_before,
            context_after: &input.context_after,
            application: &input.application,
            window_title: &input.window_title,
        })
        .map_err(|error| format!("failed to serialize source for deduplication: {error}"))?
    };
    let digest = Sha256::digest(canonical);
    Ok(format!("{digest:x}"))
}

fn canonical_source_url(input: &SourceCaptureInput) -> Option<String> {
    normalized_optional(&input.url).map(|url| {
        let without_fragment = url.split('#').next().unwrap_or(url.as_str());
        let without_query = without_fragment.split('?').next().unwrap_or(without_fragment);
        without_query.trim_end_matches('/').to_string()
    })
}

fn normalized_source_text(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn should_upgrade_to_zhihu(input: &SourceCaptureInput) -> bool {
    normalized_platform(&input.platform) == "zhihu" && canonical_source_url(input).is_some()
}

fn load_capture_by_zhihu_url(
    connection: &rusqlite::Connection,
    input: &SourceCaptureInput,
) -> Result<Option<CaptureKnowledgeResult>, String> {
    let Some(canonical_url) = canonical_source_url(input) else {
        return Ok(None);
    };
    let query = capture_query(
        "s.platform = 'zhihu' AND (
            rtrim(s.url, '/') = ?1 OR
            s.url LIKE ?2 OR
            s.url LIKE ?3
        ) AND s.selected_text = ?4",
    );
    connection
        .query_row(
            &query,
            params![
                canonical_url,
                format!("{canonical_url}?%"),
                format!("{canonical_url}#%"),
                input.selected_text,
            ],
            map_capture_row,
        )
        .optional()
        .map_err(|error| format!("failed to query existing Zhihu source URL: {error}"))
}

fn load_zhihu_upgrade_candidate(
    connection: &rusqlite::Connection,
    selected_text: &str,
) -> Result<Option<CaptureKnowledgeResult>, String> {
    connection
        .query_row(
            &capture_query(
                "s.id = (
                    SELECT id FROM sources
                    WHERE selected_text = ?1 AND platform IN ('desktop', 'unknown')
                    ORDER BY captured_at DESC
                    LIMIT 1
                )",
            ),
            [selected_text],
            map_capture_row,
        )
        .optional()
        .map_err(|error| format!("failed to query source metadata upgrade candidate: {error}"))
}

fn upgrade_source_to_zhihu(
    connection: &rusqlite::Connection,
    source_id: &str,
    input: &SourceCaptureInput,
    content_hash: &str,
) -> Result<(), String> {
    connection
        .execute(
            "UPDATE sources SET
                platform = 'zhihu',
                source_kind = 'web',
                connector_id = 'zhihu',
                url = COALESCE(?2, url),
                title = COALESCE(?3, title),
                author = COALESCE(?4, author),
                context_before = COALESCE(?5, context_before),
                context_after = COALESCE(?6, context_after),
                window_title = COALESCE(?7, window_title),
                content_hash = ?8
             WHERE id = ?1",
            params![
                source_id,
                normalized_optional(&input.url),
                normalized_optional(&input.title),
                normalized_optional(&input.author),
                normalized_optional(&input.context_before),
                normalized_optional(&input.context_after),
                normalized_optional(&input.window_title),
                content_hash,
            ],
        )
        .map_err(|error| format!("failed to upgrade source with Zhihu metadata: {error}"))?;
    Ok(())
}

pub(crate) fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis().min(i64::MAX as u128) as i64)
        .unwrap_or_default()
}

fn load_capture_by_hash(
    connection: &rusqlite::Connection,
    content_hash: &str,
) -> Result<Option<CaptureKnowledgeResult>, String> {
    connection
        .query_row(
            &capture_query("s.content_hash = ?1"),
            [content_hash],
            map_capture_row,
        )
        .optional()
        .map_err(|error| format!("failed to query existing source: {error}"))
}

fn load_capture_by_knowledge_id(
    connection: &rusqlite::Connection,
    knowledge_unit_id: &str,
) -> Result<Option<CaptureKnowledgeResult>, String> {
    connection
        .query_row(
            &capture_query("k.id = ?1"),
            [knowledge_unit_id],
            map_capture_row,
        )
        .optional()
        .map_err(|error| format!("failed to query knowledge unit: {error}"))
}

fn capture_query(filter: &str) -> String {
    format!(
        "SELECT
            s.id, s.platform, s.source_kind, s.connector_id, s.url, s.title, s.author, s.selected_text,
            s.context_before, s.context_after, s.application, s.window_title,
            s.content_hash, s.captured_at,
            k.id, k.source_id, k.core_claim, k.concepts_json, k.prerequisites_json,
            k.important_details_json, k.limitations_json, k.user_note, k.status,
            k.created_at, k.updated_at
         FROM sources s
         JOIN knowledge_units k ON k.source_id = s.id
         WHERE {filter}
         ORDER BY k.created_at ASC, k.rowid ASC
         LIMIT 1"
    )
}

fn map_capture_row(row: &Row<'_>) -> rusqlite::Result<CaptureKnowledgeResult> {
    let concepts_json: String = row.get(17)?;
    let prerequisites_json: String = row.get(18)?;
    let important_details_json: String = row.get(19)?;
    let limitations_json: String = row.get(20)?;

    Ok(CaptureKnowledgeResult {
        source: SourceRecord {
            id: row.get(0)?,
            platform: row.get(1)?,
            source_kind: row.get(2)?,
            connector_id: row.get(3)?,
            url: row.get(4)?,
            title: row.get(5)?,
            author: row.get(6)?,
            selected_text: row.get(7)?,
            context_before: row.get(8)?,
            context_after: row.get(9)?,
            application: row.get(10)?,
            window_title: row.get(11)?,
            content_hash: row.get(12)?,
            captured_at: row.get(13)?,
        },
        knowledge_unit: KnowledgeUnitRecord {
            id: row.get(14)?,
            source_id: row.get(15)?,
            core_claim: row.get(16)?,
            concepts: parse_string_array(&concepts_json),
            prerequisites: parse_string_array(&prerequisites_json),
            important_details: parse_string_array(&important_details_json),
            limitations: parse_string_array(&limitations_json),
            user_note: row.get(21)?,
            status: row.get(22)?,
            created_at: row.get(23)?,
            updated_at: row.get(24)?,
        },
        duplicate: false,
    })
}

fn parse_string_array(value: &str) -> Vec<String> {
    serde_json::from_str(value).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn platform_is_bounded_to_supported_values() {
        assert_eq!(normalized_platform("zhihu"), "zhihu");
        assert_eq!(normalized_platform("DESKTOP"), "desktop");
        assert_eq!(normalized_platform("something-else"), "unknown");
    }

    #[test]
    fn zhihu_identity_ignores_metadata_but_preserves_selection() {
        let first = SourceCaptureInput {
            platform: "zhihu".into(),
            source_kind: None,
            connector_id: None,
            selected_text: "一段正文".into(),
            url: Some("https://zhuanlan.zhihu.com/p/269199110?utm_source=test".into()),
            title: Some("标题".into()),
            author: None,
            context_before: Some("上下文 A".into()),
            context_after: None,
            application: None,
            window_title: None,
        };
        let mut second = first.clone();
        second.author = Some("JavaGuide".into());
        second.context_before = Some("上下文 B".into());
        second.url = Some("https://zhuanlan.zhihu.com/p/269199110#section".into());
        assert_eq!(capture_hash(&first).unwrap(), capture_hash(&second).unwrap());
        assert_eq!(canonical_source_url(&first).as_deref(), Some("https://zhuanlan.zhihu.com/p/269199110"));

        second.selected_text = "另一段正文".into();
        assert_ne!(capture_hash(&first).unwrap(), capture_hash(&second).unwrap());
    }

    #[test]
    fn zhihu_upgrade_requires_canonical_source_url() {
        let mut input = SourceCaptureInput {
            platform: "zhihu".into(),
            source_kind: None,
            connector_id: None,
            selected_text: "same quote".into(),
            url: None,
            title: Some("Question".into()),
            author: None,
            context_before: None,
            context_after: None,
            application: None,
            window_title: None,
        };
        assert!(!should_upgrade_to_zhihu(&input));
        input.url = Some("https://www.zhihu.com/question/1/answer/2".into());
        assert!(should_upgrade_to_zhihu(&input));
    }

    #[test]
    fn empty_selection_is_rejected() {
        let input = SourceCaptureInput {
            platform: "desktop".into(),
            source_kind: None,
            connector_id: None,
            selected_text: "   ".into(),
            url: None,
            title: None,
            author: None,
            context_before: None,
            context_after: None,
            application: None,
            window_title: None,
        };
        assert!(validate_capture(&input).is_err());
    }
}
