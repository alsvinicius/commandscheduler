use std::sync::Arc;

use crate::database::{self as sqlx, FromRow, Row, SqlitePool};
use axum::{
    Json, Router,
    body::Body,
    extract::{FromRequest, Path, Query, State, rejection::JsonRejection},
    http::{HeaderMap, HeaderValue, StatusCode, Uri, header},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, post, put},
};
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::Value;
use uuid::Uuid;

use crate::{
    execution::test_command,
    recurrence::{next_local_occurrence, parse_cron},
    storage::{CreateScheduleOutcome, NewSchedule, create_schedule},
};

const DEFAULT_LIMIT: u64 = 50;
const MAX_LIMIT: u64 = 100;
const MAX_PARAMETERS_BYTES: usize = 64 * 1024;
type RetryMetadata = (String, i64, Option<i64>, Option<i64>, Option<i64>);

#[derive(Clone)]
pub struct ApiState {
    pub pool: SqlitePool,
    pub cancellation: Arc<dyn Fn(String) -> bool + Send + Sync>,
}

pub fn router(state: ApiState) -> Router {
    Router::new()
        .route("/commands/test", post(test_command_handler))
        .route(
            "/schedules",
            get(list_schedules).post(create_schedule_handler),
        )
        .route("/overview", get(get_overview))
        .route("/agenda", get(get_agenda))
        .route("/occurrences", get(list_all_occurrences))
        .route(
            "/schedules/{schedule_id}",
            get(get_schedule).patch(update_schedule),
        )
        .route(
            "/schedules/{schedule_id}/parameters",
            put(update_parameters),
        )
        .route("/schedules/{schedule_id}/pause", post(pause_schedule))
        .route("/schedules/{schedule_id}/resume", post(resume_schedule))
        .route("/schedules/{schedule_id}/cancel", post(cancel_schedule))
        .route("/schedules/{schedule_id}/runs", post(run_now))
        .route(
            "/schedules/{schedule_id}/occurrences",
            get(list_occurrences),
        )
        .route("/occurrences/{occurrence_id}", get(get_occurrence))
        .route(
            "/occurrences/{occurrence_id}/cancel",
            post(cancel_occurrence),
        )
        .route("/occurrences/{occurrence_id}/retry", post(retry_occurrence))
        .with_state(state.clone())
        .layer(middleware::from_fn_with_state(state, validate_origin))
}

async fn validate_origin(
    State(_state): State<ApiState>,
    request: axum::http::Request<Body>,
    next: Next,
) -> Response {
    let host = request
        .headers()
        .get(header::HOST)
        .and_then(|value| value.to_str().ok());
    if request.headers().contains_key(header::HOST) && !host.is_some_and(trusted_local_authority) {
        return ApiError::new(
            StatusCode::FORBIDDEN,
            "origin_forbidden",
            "Host is not a trusted loopback destination",
        )
        .into_response();
    }
    if let Some(origin) = request.headers().get(header::ORIGIN) {
        let origin = origin
            .to_str()
            .ok()
            .and_then(|value| value.parse::<Uri>().ok());
        let origin_authority = origin
            .as_ref()
            .and_then(Uri::authority)
            .map(|value| value.as_str());
        let trusted_local = origin.as_ref().is_some_and(|uri| {
            uri.scheme_str() == Some("http")
                && uri.query().is_none()
                && uri.path() == "/"
                && origin_authority.is_some_and(trusted_local_authority)
        });
        if origin_authority != host || !trusted_local {
            return ApiError::new(
                StatusCode::FORBIDDEN,
                "origin_forbidden",
                "Browser origin is not the local UI origin",
            )
            .into_response();
        }
    }
    next.run(request).await
}

fn trusted_local_authority(value: &str) -> bool {
    if value.contains('@') {
        return false;
    }
    value
        .parse::<axum::http::uri::Authority>()
        .ok()
        .is_some_and(|authority| {
            let host = authority
                .host()
                .trim_start_matches('[')
                .trim_end_matches(']');
            host.eq_ignore_ascii_case("localhost")
                || host
                    .parse::<std::net::IpAddr>()
                    .is_ok_and(|ip| {
                        ip.is_loopback()
                            || matches!(ip, std::net::IpAddr::V4(address)
                                if u32::from(address) >= u32::from(std::net::Ipv4Addr::new(100, 64, 0, 0))
                                    && u32::from(address) <= u32::from(std::net::Ipv4Addr::new(100, 127, 255, 255)))
                    })
        })
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ScheduleDto {
    id: String,
    name: Option<String>,
    program: String,
    args: Vec<String>,
    timing: Value,
    parameters: Value,
    missed_policy: String,
    parameter_update_trigger: Value,
    retry_policy: Value,
    status: String,
    created_at: String,
    updated_at: String,
}

#[derive(Debug, Serialize)]
struct SchedulePage {
    items: Vec<ScheduleDto>,
    #[serde(rename = "nextCursor")]
    next_cursor: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct OccurrenceDto {
    id: String,
    schedule_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    schedule_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    schedule_program: Option<String>,
    status: String,
    source: String,
    attempt: i64,
    created_at: String,
    scheduled_for: Option<String>,
    started_at: Option<String>,
    finished_at: Option<String>,
    exit_code: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stdout: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stderr: Option<String>,
    stdout_truncated: bool,
    stderr_truncated: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    missed_occurrences: Option<MissedOccurrences>,
    retry_of: Option<String>,
}

#[derive(Debug, Serialize)]
struct MissedOccurrences {
    count: i64,
    from: String,
    through: String,
}

#[derive(Debug, Serialize)]
struct OccurrencePage {
    items: Vec<OccurrenceDto>,
    #[serde(rename = "nextCursor")]
    next_cursor: Option<String>,
}

#[derive(Debug, Serialize)]
struct Problem {
    r#type: String,
    title: String,
    status: u16,
    detail: String,
    code: String,
}

#[derive(Debug)]
struct ApiError {
    status: StatusCode,
    code: &'static str,
    detail: String,
}

struct ApiJson<T>(T);

impl<S, T> FromRequest<S> for ApiJson<T>
where
    S: Send + Sync,
    T: DeserializeOwned,
{
    type Rejection = ApiError;

    async fn from_request(
        request: axum::http::Request<Body>,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        Json::<T>::from_request(request, state)
            .await
            .map(|Json(value)| Self(value))
            .map_err(json_rejection)
    }
}

fn json_rejection(rejection: JsonRejection) -> ApiError {
    let status = match rejection.status() {
        StatusCode::UNSUPPORTED_MEDIA_TYPE => StatusCode::UNSUPPORTED_MEDIA_TYPE,
        StatusCode::PAYLOAD_TOO_LARGE => StatusCode::PAYLOAD_TOO_LARGE,
        _ => StatusCode::BAD_REQUEST,
    };
    let code = if status == StatusCode::UNSUPPORTED_MEDIA_TYPE {
        "unsupported_media_type"
    } else if status == StatusCode::PAYLOAD_TOO_LARGE {
        "payload_too_large"
    } else {
        "validation_error"
    };
    ApiError::new(status, code, rejection.body_text())
}

impl ApiError {
    fn new(status: StatusCode, code: &'static str, detail: impl Into<String>) -> Self {
        Self {
            status,
            code,
            detail: detail.into(),
        }
    }
    fn not_found(kind: &str) -> Self {
        Self::new(
            StatusCode::NOT_FOUND,
            "not_found",
            format!("{kind} was not found"),
        )
    }
    fn bad(detail: impl Into<String>) -> Self {
        Self::new(StatusCode::BAD_REQUEST, "validation_error", detail)
    }
    fn db(error: sqlx::Error) -> Self {
        Self::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "internal_error",
            error.to_string(),
        )
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let title = self.status.canonical_reason().unwrap_or("Error").to_owned();
        let body = Json(Problem {
            r#type: "about:blank".to_owned(),
            title,
            status: self.status.as_u16(),
            detail: self.detail,
            code: self.code.to_owned(),
        });
        let mut response = (self.status, body).into_response();
        response.headers_mut().insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/problem+json"),
        );
        response
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct CreateRequest {
    name: Option<String>,
    program: String,
    args: Vec<String>,
    timing: Value,
    #[serde(default)]
    parameters: Option<Value>,
    missed_policy: String,
    #[serde(default)]
    parameter_update_trigger: Option<Value>,
    #[serde(default)]
    retry_policy: Option<Value>,
}

#[derive(Deserialize)]
struct PageQuery {
    limit: Option<u64>,
    cursor: Option<String>,
    status: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RangeQuery {
    from: Option<String>,
    to: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct GlobalOccurrenceQuery {
    limit: Option<u64>,
    cursor: Option<String>,
    status: Option<String>,
    schedule_id: Option<String>,
    from: Option<String>,
    to: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct OverviewDto {
    schedules: ScheduleCounts,
    completed: i64,
    failed: i64,
    next: Option<NextScheduleDto>,
}

#[derive(Serialize)]
struct ScheduleCounts {
    active: i64,
    paused: i64,
    cancelled: i64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct NextScheduleDto {
    schedule_id: String,
    name: Option<String>,
    program: String,
    at: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AgendaItemDto {
    schedule_id: String,
    name: Option<String>,
    program: String,
    at: String,
    occurrence_id: Option<String>,
    status: Option<String>,
}

#[derive(Serialize)]
struct AgendaDto {
    items: Vec<AgendaItemDto>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct UpdateRequest {
    name: Option<String>,
    program: Option<String>,
    args: Option<Vec<String>>,
    timing: Option<Value>,
    missed_policy: Option<String>,
    parameter_update_trigger: Option<Value>,
    retry_policy: Option<Value>,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct RetryRequest {
    #[serde(default)]
    confirm_interrupted: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct TestCommandRequest {
    program: String,
    #[serde(default)]
    args: Vec<String>,
    #[serde(default)]
    parameters: Value,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct TestCommandResponse {
    exit_code: Option<i32>,
    stdout: String,
    stderr: String,
    stdout_truncated: bool,
    stderr_truncated: bool,
    timed_out: bool,
}

async fn test_command_handler(
    ApiJson(input): ApiJson<TestCommandRequest>,
) -> Result<Json<TestCommandResponse>, ApiError> {
    if input.program.trim().is_empty() {
        return Err(ApiError::bad("program must not be empty"));
    }
    if serde_json::to_vec(&input.parameters)
        .map_err(|error| ApiError::bad(error.to_string()))?
        .len()
        > MAX_PARAMETERS_BYTES
    {
        return Err(ApiError::new(
            StatusCode::PAYLOAD_TOO_LARGE,
            "parameters_too_large",
            "parameters must be at most 64 KiB when serialized",
        ));
    }
    let result = test_command(&input.program, &input.args, &input.parameters)
        .await
        .map_err(|error| {
            ApiError::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                "command_test_failed",
                error.to_string(),
            )
        })?;
    Ok(Json(TestCommandResponse {
        exit_code: result.exit_code,
        stdout: result.stdout,
        stderr: result.stderr,
        stdout_truncated: result.stdout_truncated,
        stderr_truncated: result.stderr_truncated,
        timed_out: result.timed_out,
    }))
}

async fn create_schedule_handler(
    State(state): State<ApiState>,
    headers: HeaderMap,
    ApiJson(input): ApiJson<CreateRequest>,
) -> Result<Response, ApiError> {
    validate_timing(&input.timing)?;
    validate_policy(&input.missed_policy)?;
    if let Some(trigger) = &input.parameter_update_trigger {
        validate_trigger(trigger)?;
    }
    if let Some(retry) = &input.retry_policy {
        validate_retry_policy(retry)?;
    }
    let idempotency_key = headers
        .get("idempotency-key")
        .map(|value| {
            value
                .to_str()
                .map(str::to_owned)
                .map_err(|_| ApiError::bad("Idempotency-Key must be valid text"))
        })
        .transpose()?;
    if idempotency_key.as_ref().is_some_and(String::is_empty) {
        return Err(ApiError::bad("Idempotency-Key must not be empty"));
    }
    if let Some(at) = once_timestamp(&input.timing) {
        if at <= now_micros()? {
            let existing: bool = if let Some(key) = &idempotency_key {
                sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM schedules WHERE idempotency_key=?)")
                    .bind(key)
                    .fetch_one(&state.pool)
                    .await
                    .map_err(ApiError::db)?
            } else {
                false
            };
            if !existing {
                return Err(ApiError::bad(
                    "new one-time schedules must be in the future",
                ));
            }
        }
    }
    let request = NewSchedule {
        name: input.name,
        program: input.program,
        args: input.args,
        timing: input.timing,
        parameters: input.parameters,
        missed_policy: input.missed_policy,
        parameter_update_trigger: input.parameter_update_trigger,
        retry_policy: input.retry_policy,
        idempotency_key,
    };
    let outcome = create_schedule(&state.pool, &request)
        .await
        .map_err(|error| {
            if error.to_string().contains("different request content") {
                ApiError::new(
                    StatusCode::CONFLICT,
                    "idempotency_conflict",
                    error.to_string(),
                )
            } else if error.to_string().contains("64 KiB") {
                ApiError::new(
                    StatusCode::PAYLOAD_TOO_LARGE,
                    "parameters_too_large",
                    error.to_string(),
                )
            } else {
                ApiError::bad(error.to_string())
            }
        })?;
    let (id, is_replay) = match outcome {
        CreateScheduleOutcome::Created { id } => (id, false),
        CreateScheduleOutcome::Replayed { id } => (id, true),
    };
    let schedule = fetch_schedule(&state.pool, &id)
        .await?
        .ok_or_else(|| ApiError::not_found("schedule"))?;
    Ok((
        if is_replay {
            StatusCode::OK
        } else {
            StatusCode::CREATED
        },
        Json(schedule),
    )
        .into_response())
}

async fn list_schedules(
    State(state): State<ApiState>,
    Query(query): Query<PageQuery>,
) -> Result<Json<SchedulePage>, ApiError> {
    let limit = page_limit(query.limit)?;
    let cursor = decode_key_cursor(query.cursor.as_deref())?;
    if query
        .status
        .as_deref()
        .is_some_and(|s| !valid_schedule_status(s))
    {
        return Err(ApiError::bad("invalid schedule status"));
    }
    let mut qb = sqlx::QueryBuilder::<sqlx::Sqlite>::new(
        "SELECT id,name,program,args_json,timing_json,parameters_json,missed_policy,parameter_update_trigger_json,retry_policy_json,status,created_at,updated_at FROM schedules WHERE 1=1",
    );
    if let Some(status) = query.status {
        qb.push(" AND status=").push_bind(status);
    }
    if let Some((created, id)) = cursor {
        qb.push(" AND (created_at,id)<(")
            .push_bind(created)
            .push(",")
            .push_bind(id)
            .push(")");
    }
    qb.push(" ORDER BY created_at DESC,id DESC LIMIT ")
        .push_bind(limit as i64 + 1);
    let rows = qb
        .build_query_as::<ScheduleRow>()
        .fetch_all(&state.pool)
        .await
        .map_err(ApiError::db)?;
    let has_more = rows.len() > limit as usize;
    let next_cursor = has_more.then(|| {
        encode_key_cursor(
            rows[limit as usize - 1].created_at,
            &rows[limit as usize - 1].id,
        )
    });
    let items = rows
        .into_iter()
        .take(limit as usize)
        .map(schedule_dto)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Json(SchedulePage { items, next_cursor }))
}

async fn get_overview(
    State(state): State<ApiState>,
    Query(query): Query<RangeQuery>,
) -> Result<Json<OverviewDto>, ApiError> {
    let now = now_micros()?;
    let from = parse_bound(query.from.as_deref())?.unwrap_or(now - 24 * 60 * 60 * 1_000_000);
    let to = parse_bound(query.to.as_deref())?.unwrap_or(now);
    validate_range(from, to)?;
    let active: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM schedules WHERE status='active'")
        .fetch_one(&state.pool)
        .await
        .map_err(ApiError::db)?;
    let paused: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM schedules WHERE status='paused'")
        .fetch_one(&state.pool)
        .await
        .map_err(ApiError::db)?;
    let cancelled: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM schedules WHERE status='cancelled'")
            .fetch_one(&state.pool)
            .await
            .map_err(ApiError::db)?;
    let completed: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM occurrences WHERE status='succeeded' AND finished_at>=? AND finished_at<?").bind(from).bind(to).fetch_one(&state.pool).await.map_err(ApiError::db)?;
    let failed: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM occurrences WHERE status='failed' AND finished_at>=? AND finished_at<?").bind(from).bind(to).fetch_one(&state.pool).await.map_err(ApiError::db)?;
    let rows = sqlx::query_as::<_, (String, Option<String>, String, String)>(
        "SELECT id,name,program,timing_json FROM schedules WHERE status='active'",
    )
    .fetch_all(&state.pool)
    .await
    .map_err(ApiError::db)?;
    let mut next: Option<NextScheduleDto> = None;
    for (id, name, program, raw) in rows {
        let timing: Value = serde_json::from_str(&raw).map_err(|e| ApiError::bad(e.to_string()))?;
        let at = match timing.get("type").and_then(Value::as_str) {
            Some("once") => timing
                .get("at")
                .and_then(Value::as_str)
                .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                .map(|d| d.timestamp_micros())
                .filter(|d| *d > now),
            Some("cron") => timing
                .get("expression")
                .and_then(Value::as_str)
                .and_then(|s| next_local_occurrence(s, now).ok()),
            _ => None,
        };
        if let Some(at) = at {
            if next.as_ref().is_none_or(|old| {
                at < DateTime::parse_from_rfc3339(&old.at)
                    .unwrap()
                    .timestamp_micros()
            }) {
                next = Some(NextScheduleDto {
                    schedule_id: id,
                    name,
                    program,
                    at: iso_from_micros(at)?,
                });
            }
        }
    }
    Ok(Json(OverviewDto {
        schedules: ScheduleCounts {
            active,
            paused,
            cancelled,
        },
        completed,
        failed,
        next,
    }))
}

async fn get_agenda(
    State(state): State<ApiState>,
    Query(query): Query<RangeQuery>,
) -> Result<Json<AgendaDto>, ApiError> {
    let from =
        parse_bound(query.from.as_deref())?.ok_or_else(|| ApiError::bad("from is required"))?;
    let to = parse_bound(query.to.as_deref())?.ok_or_else(|| ApiError::bad("to is required"))?;
    validate_range(from, to)?;
    let now = now_micros()?;
    let future_from = from.max(now);
    let rows = sqlx::query_as::<_, (String, Option<String>, String, String)>(
        "SELECT id,name,program,timing_json FROM schedules WHERE status='active'",
    )
    .fetch_all(&state.pool)
    .await
    .map_err(ApiError::db)?;
    let mut items = Vec::new();
    for (id, name, program, raw) in rows {
        let timing: Value = serde_json::from_str(&raw).map_err(|e| ApiError::bad(e.to_string()))?;
        match timing.get("type").and_then(Value::as_str) {
            Some("once") => {
                if let Some(at) = timing
                    .get("at")
                    .and_then(Value::as_str)
                    .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                    .map(|d| d.timestamp_micros())
                    .filter(|t| *t >= future_from && *t < to)
                {
                    items.push(AgendaItemDto {
                        schedule_id: id,
                        name,
                        program,
                        at: iso_from_micros(at)?,
                        occurrence_id: None,
                        status: None,
                    });
                }
            }
            Some("cron") => {
                if let Some(expr) = timing.get("expression").and_then(Value::as_str) {
                    let mut cursor = future_from.saturating_sub(1);
                    let mut generated = 0usize;
                    while generated < 500 {
                        let Ok(at) = next_local_occurrence(expr, cursor) else {
                            break;
                        };
                        if at >= to {
                            break;
                        }
                        items.push(AgendaItemDto {
                            schedule_id: id.clone(),
                            name: name.clone(),
                            program: program.clone(),
                            at: iso_from_micros(at)?,
                            occurrence_id: None,
                            status: None,
                        });
                        cursor = at;
                        generated += 1;
                    }
                }
            }
            _ => {}
        }
    }
    let queued = sqlx::query_as::<_, (String, Option<String>, String, String, String, i64)>(
        "SELECT o.schedule_id,s.name,s.program,o.id,o.status,COALESCE(o.scheduled_for,o.started_at,o.created_at) FROM occurrences o JOIN schedules s ON s.id=o.schedule_id WHERE o.status='running' OR (o.status='queued' AND COALESCE(o.scheduled_for,o.created_at)>=? AND COALESCE(o.scheduled_for,o.created_at)<?)",
    )
    .bind(future_from)
    .bind(to)
    .fetch_all(&state.pool)
    .await
    .map_err(ApiError::db)?;
    for (schedule_id, name, program, occurrence_id, status, at) in queued {
        let at = iso_from_micros(at)?;
        items.retain(|item| !(item.schedule_id == schedule_id && item.at == at));
        items.push(AgendaItemDto {
            schedule_id,
            name,
            program,
            at,
            occurrence_id: Some(occurrence_id),
            status: Some(status),
        });
    }
    items.sort_by(|a, b| a.at.cmp(&b.at).then(a.schedule_id.cmp(&b.schedule_id)));
    items.truncate(500);
    let history = sqlx::query_as::<_, (String, Option<String>, String, String, String, i64)>(
        "SELECT o.schedule_id,s.name,s.program,o.id,o.status,o.finished_at FROM occurrences o JOIN schedules s ON s.id=o.schedule_id WHERE o.finished_at>=? AND o.finished_at<? ORDER BY o.finished_at DESC,o.id DESC LIMIT 500",
    ).bind(from).bind(to).fetch_all(&state.pool).await.map_err(ApiError::db)?;
    for (schedule_id, name, program, occurrence_id, status, finished_at) in history {
        items.push(AgendaItemDto {
            schedule_id,
            name,
            program,
            at: iso_from_micros(finished_at)?,
            occurrence_id: Some(occurrence_id),
            status: Some(status),
        });
    }
    items.sort_by(|a, b| a.at.cmp(&b.at).then(a.schedule_id.cmp(&b.schedule_id)));
    Ok(Json(AgendaDto { items }))
}

async fn list_all_occurrences(
    State(state): State<ApiState>,
    Query(query): Query<GlobalOccurrenceQuery>,
) -> Result<Json<OccurrencePage>, ApiError> {
    if query
        .status
        .as_deref()
        .is_some_and(|s| !valid_occurrence_status(s))
    {
        return Err(ApiError::bad("invalid occurrence status"));
    }
    let limit = page_limit(query.limit)?;
    let cursor = decode_key_cursor(query.cursor.as_deref())?;
    let from = parse_bound(query.from.as_deref())?;
    let to = parse_bound(query.to.as_deref())?;
    if let (Some(a), Some(b)) = (from, to) {
        validate_range(a, b)?;
    }
    let mut qb = sqlx::QueryBuilder::<sqlx::Sqlite>::new(
        "SELECT o.id,o.schedule_id,s.name AS schedule_name,s.program AS schedule_program,o.status,o.source,o.attempt,o.created_at,o.scheduled_for,o.started_at,o.finished_at,o.exit_code,NULL AS stdout,NULL AS stderr,o.stdout_truncated,o.stderr_truncated,o.missed_count,o.missed_from,o.missed_through,o.retry_of_id FROM occurrences o LEFT JOIN schedules s ON s.id=o.schedule_id WHERE 1=1",
    );
    if let Some(status) = query.status {
        qb.push(" AND o.status=").push_bind(status);
    }
    if let Some(id) = query.schedule_id {
        qb.push(" AND o.schedule_id=").push_bind(id);
    }
    if let Some(from) = from {
        qb.push(" AND COALESCE(o.finished_at,o.created_at)>=")
            .push_bind(from);
    }
    if let Some(to) = to {
        qb.push(" AND COALESCE(o.finished_at,o.created_at)<")
            .push_bind(to);
    }
    if let Some((created, id)) = cursor {
        qb.push(" AND (o.created_at,o.id)<(")
            .push_bind(created)
            .push(",")
            .push_bind(id)
            .push(")");
    }
    qb.push(" ORDER BY o.created_at DESC,o.id DESC LIMIT ")
        .push_bind(limit as i64 + 1);
    let rows = qb
        .build_query_as::<OccurrenceRow>()
        .fetch_all(&state.pool)
        .await
        .map_err(ApiError::db)?;
    let has_more = rows.len() > limit as usize;
    let next_cursor = has_more.then(|| {
        encode_key_cursor(
            rows[limit as usize - 1].created_at,
            &rows[limit as usize - 1].id,
        )
    });
    let items = rows
        .into_iter()
        .take(limit as usize)
        .map(occurrence_dto)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Json(OccurrencePage { items, next_cursor }))
}

async fn get_schedule(
    State(state): State<ApiState>,
    Path(id): Path<String>,
) -> Result<Json<ScheduleDto>, ApiError> {
    fetch_schedule(&state.pool, &id)
        .await?
        .map(Json)
        .ok_or_else(|| ApiError::not_found("schedule"))
}

async fn update_schedule(
    State(state): State<ApiState>,
    Path(id): Path<String>,
    ApiJson(input): ApiJson<UpdateRequest>,
) -> Result<Json<ScheduleDto>, ApiError> {
    if input
        .program
        .as_deref()
        .is_some_and(|value| value.trim().is_empty())
    {
        return Err(ApiError::bad("program must not be empty"));
    }
    if let Some(timing) = &input.timing {
        validate_timing(timing)?;
    }
    if let Some(policy) = &input.missed_policy {
        validate_policy(policy)?;
    }
    if let Some(trigger) = &input.parameter_update_trigger {
        validate_trigger(trigger)?;
    }
    if let Some(retry) = &input.retry_policy {
        validate_retry_policy(retry)?;
    }
    if input.name.is_none()
        && input.program.is_none()
        && input.args.is_none()
        && input.timing.is_none()
        && input.missed_policy.is_none()
        && input.parameter_update_trigger.is_none()
        && input.retry_policy.is_none()
    {
        return Err(ApiError::bad("at least one field is required"));
    }
    let args = input
        .args
        .map(|value| serde_json::to_string(&value))
        .transpose()
        .map_err(|error| ApiError::bad(error.to_string()))?;
    let timing = input
        .timing
        .map(|value| serde_json::to_string(&value))
        .transpose()
        .map_err(|error| ApiError::bad(error.to_string()))?;
    let trigger = input
        .parameter_update_trigger
        .map(|value| serde_json::to_string(&value))
        .transpose()
        .map_err(|error| ApiError::bad(error.to_string()))?;
    let retry = input
        .retry_policy
        .map(|value| serde_json::to_string(&value))
        .transpose()
        .map_err(|error| ApiError::bad(error.to_string()))?;
    let mut tx = state
        .pool
        .begin_with("BEGIN IMMEDIATE")
        .await
        .map_err(ApiError::db)?;
    let previous: Option<String> =
        sqlx::query_scalar("SELECT timing_json FROM schedules WHERE id=?")
            .bind(&id)
            .fetch_optional(&mut *tx)
            .await
            .map_err(ApiError::db)?;
    let new_timing = timing
        .as_deref()
        .map(serde_json::from_str::<Value>)
        .transpose()
        .map_err(|error| ApiError::bad(error.to_string()))?;
    let old_timing = previous
        .as_deref()
        .map(serde_json::from_str::<Value>)
        .transpose()
        .map_err(|error| ApiError::bad(error.to_string()))?;
    let timing_changed = new_timing
        .as_ref()
        .zip(old_timing.as_ref())
        .is_some_and(|(new, old)| !same_timing(new, old));
    let now = now_micros()?;
    if timing_changed
        && new_timing
            .as_ref()
            .and_then(once_timestamp)
            .is_some_and(|at| at <= now)
    {
        return Err(ApiError::bad("new one-time timing must be in the future"));
    }
    let result = sqlx::query("UPDATE schedules SET name=COALESCE(?,name),program=COALESCE(?,program),args_json=COALESCE(?,args_json),timing_json=COALESCE(?,timing_json),last_evaluated_at=CASE WHEN ? THEN ? ELSE last_evaluated_at END,missed_policy=COALESCE(?,missed_policy),parameter_update_trigger_json=COALESCE(?,parameter_update_trigger_json),retry_policy_json=COALESCE(?,retry_policy_json),updated_at=? WHERE id=? AND status!='cancelled'")
        .bind(input.name).bind(input.program).bind(args).bind(timing).bind(timing_changed).bind(now).bind(input.missed_policy).bind(trigger).bind(retry).bind(now).bind(&id).execute(&mut *tx).await.map_err(ApiError::db)?;
    if result.rows_affected() == 0 {
        tx.rollback().await.map_err(ApiError::db)?;
        return if fetch_schedule(&state.pool, &id).await?.is_some() {
            Err(ApiError::new(
                StatusCode::CONFLICT,
                "invalid_state",
                "Cancelled schedule cannot be updated",
            ))
        } else {
            Err(ApiError::not_found("schedule"))
        };
    }
    tx.commit().await.map_err(ApiError::db)?;
    Ok(Json(
        fetch_schedule(&state.pool, &id)
            .await?
            .ok_or_else(|| ApiError::not_found("schedule"))?,
    ))
}

fn once_timestamp(timing: &Value) -> Option<i64> {
    if timing.get("type").and_then(Value::as_str) != Some("once") {
        return None;
    }
    timing
        .get("at")
        .and_then(Value::as_str)
        .and_then(|at| DateTime::parse_from_rfc3339(at).ok())
        .map(|at| at.timestamp_micros())
}

fn same_timing(a: &Value, b: &Value) -> bool {
    if a.get("type") != b.get("type") {
        return false;
    }
    match a.get("type").and_then(Value::as_str) {
        Some("once") => once_timestamp(a) == once_timestamp(b),
        Some("cron") => {
            a.get("expression")
                .and_then(Value::as_str)
                .map(|expr| expr.split_whitespace().collect::<Vec<_>>())
                == b.get("expression")
                    .and_then(Value::as_str)
                    .map(|expr| expr.split_whitespace().collect::<Vec<_>>())
        }
        _ => a == b,
    }
}

async fn update_parameters(
    State(state): State<ApiState>,
    Path(id): Path<String>,
    ApiJson(parameters): ApiJson<Value>,
) -> Result<Json<Value>, ApiError> {
    let encoded =
        serde_json::to_string(&parameters).map_err(|error| ApiError::bad(error.to_string()))?;
    if encoded.len() > MAX_PARAMETERS_BYTES {
        return Err(ApiError::new(
            StatusCode::PAYLOAD_TOO_LARGE,
            "parameters_too_large",
            "Parameters exceed 64 KiB",
        ));
    }
    let mut tx = state
        .pool
        .begin_with("BEGIN IMMEDIATE")
        .await
        .map_err(ApiError::db)?;
    let trigger: Option<String> = sqlx::query_scalar(
        "SELECT parameter_update_trigger_json FROM schedules WHERE id=? AND status!='cancelled'",
    )
    .bind(&id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(ApiError::db)?;
    let Some(trigger) = trigger else {
        // Release the write reservation and connection before a second pool query.
        tx.rollback().await.map_err(ApiError::db)?;
        return if fetch_schedule(&state.pool, &id).await?.is_some() {
            Err(ApiError::new(
                StatusCode::CONFLICT,
                "invalid_state",
                "Cancelled schedule cannot be updated",
            ))
        } else {
            Err(ApiError::not_found("schedule"))
        };
    };
    let trigger: Value =
        serde_json::from_str(&trigger).map_err(|error| ApiError::bad(error.to_string()))?;
    let updated_at = now_micros()?;
    sqlx::query("UPDATE schedules SET parameters_json=?,updated_at=? WHERE id=?")
        .bind(encoded)
        .bind(updated_at)
        .bind(&id)
        .execute(&mut *tx)
        .await
        .map_err(ApiError::db)?;
    let occurrence_id = if trigger
        .get("enabled")
        .and_then(Value::as_bool)
        .unwrap_or(false)
    {
        if trigger.get("scheduleBehavior").and_then(Value::as_str) == Some("replace") {
            sqlx::query("UPDATE occurrences SET status='cancelled',finished_at=? WHERE schedule_id=? AND status='queued' AND source='schedule'").bind(updated_at).bind(&id).execute(&mut *tx).await.map_err(ApiError::db)?;
        }
        let occurrence_id = Uuid::new_v4().to_string();
        sqlx::query("INSERT INTO occurrences(id,schedule_id,status,source,scheduled_for,created_at) VALUES(?,?,'queued','parameter_update',?,?)").bind(&occurrence_id).bind(&id).bind(updated_at).bind(updated_at).execute(&mut *tx).await.map_err(ApiError::db)?;
        Some(occurrence_id)
    } else {
        None
    };
    tx.commit().await.map_err(ApiError::db)?;
    let schedule = fetch_schedule(&state.pool, &id)
        .await?
        .ok_or_else(|| ApiError::not_found("schedule"))?;
    let mut result = serde_json::json!({"schedule":schedule});
    if let Some(occurrence_id) = occurrence_id {
        result["occurrenceId"] = Value::String(occurrence_id);
    }
    Ok(Json(result))
}

async fn pause_schedule(
    State(state): State<ApiState>,
    Path(id): Path<String>,
) -> Result<Json<ScheduleDto>, ApiError> {
    transition_schedule(&state, &id, "active", "paused").await
}
async fn resume_schedule(
    State(state): State<ApiState>,
    Path(id): Path<String>,
) -> Result<Json<ScheduleDto>, ApiError> {
    transition_schedule(&state, &id, "paused", "active").await
}
async fn cancel_schedule(
    State(state): State<ApiState>,
    Path(id): Path<String>,
) -> Result<Json<ScheduleDto>, ApiError> {
    let mut tx = state
        .pool
        .begin_with("BEGIN IMMEDIATE")
        .await
        .map_err(ApiError::db)?;
    let now = now_micros()?;
    let result=sqlx::query("UPDATE schedules SET status='cancelled',updated_at=? WHERE id=? AND status IN ('active','paused')")
        .bind(now).bind(&id).execute(&mut *tx).await.map_err(ApiError::db)?;
    if result.rows_affected() == 0 {
        tx.rollback().await.map_err(ApiError::db)?;
        return if fetch_schedule(&state.pool, &id).await?.is_some() {
            Err(ApiError::new(
                StatusCode::CONFLICT,
                "invalid_state",
                "Schedule is already cancelled",
            ))
        } else {
            Err(ApiError::not_found("schedule"))
        };
    }
    sqlx::query("UPDATE occurrences SET status='cancelled',finished_at=? WHERE schedule_id=? AND status='queued' AND source='retry'")
        .bind(now).bind(&id).execute(&mut *tx).await.map_err(ApiError::db)?;
    tx.commit().await.map_err(ApiError::db)?;
    Ok(Json(
        fetch_schedule(&state.pool, &id)
            .await?
            .ok_or_else(|| ApiError::not_found("schedule"))?,
    ))
}

async fn transition_schedule(
    state: &ApiState,
    id: &str,
    from: &str,
    to: &str,
) -> Result<Json<ScheduleDto>, ApiError> {
    let result = sqlx::query("UPDATE schedules SET status=?,updated_at=? WHERE id=? AND status=?")
        .bind(to)
        .bind(now_micros()?)
        .bind(id)
        .bind(from)
        .execute(&state.pool)
        .await
        .map_err(ApiError::db)?;
    if result.rows_affected() == 0 {
        return if fetch_schedule(&state.pool, id).await?.is_some() {
            Err(ApiError::new(
                StatusCode::CONFLICT,
                "invalid_state",
                format!("Schedule cannot transition from {from} to {to}"),
            ))
        } else {
            Err(ApiError::not_found("schedule"))
        };
    }
    Ok(Json(
        fetch_schedule(&state.pool, id)
            .await?
            .ok_or_else(|| ApiError::not_found("schedule"))?,
    ))
}

async fn run_now(
    State(state): State<ApiState>,
    Path(id): Path<String>,
) -> Result<Response, ApiError> {
    let exists: Option<String> =
        sqlx::query_scalar("SELECT id FROM schedules WHERE id=? AND status!='cancelled'")
            .bind(&id)
            .fetch_optional(&state.pool)
            .await
            .map_err(ApiError::db)?;
    if exists.is_none() {
        return Err(ApiError::not_found("schedule"));
    }
    let occurrence = insert_manual_occurrence(&state.pool, &id, "manual", None).await?;
    let dto = fetch_occurrence(&state.pool, &occurrence)
        .await?
        .ok_or_else(|| ApiError::not_found("occurrence"))?;
    Ok((StatusCode::ACCEPTED, Json(dto)).into_response())
}

async fn list_occurrences(
    State(state): State<ApiState>,
    Path(schedule_id): Path<String>,
    Query(query): Query<PageQuery>,
) -> Result<Json<OccurrencePage>, ApiError> {
    if !schedule_exists(&state.pool, &schedule_id).await? {
        return Err(ApiError::not_found("schedule"));
    }
    let limit = page_limit(query.limit)?;
    let cursor = decode_key_cursor(query.cursor.as_deref())?;
    let rows = if let Some(status) = query.status.as_deref() {
        if !valid_occurrence_status(status) {
            return Err(ApiError::bad("invalid occurrence status"));
        }
        sqlx::query_as::<_,OccurrenceRow>("SELECT o.id,o.schedule_id,s.name AS schedule_name,s.program AS schedule_program,o.status,o.source,o.attempt,o.created_at,o.scheduled_for,o.started_at,o.finished_at,o.exit_code,o.stdout,o.stderr,o.stdout_truncated,o.stderr_truncated,o.missed_count,o.missed_from,o.missed_through,o.retry_of_id FROM occurrences o LEFT JOIN schedules s ON s.id=o.schedule_id WHERE o.schedule_id=? AND o.status=? AND (? IS NULL OR (o.created_at,o.id)<(?,?)) ORDER BY o.created_at DESC,o.id DESC LIMIT ?").bind(&schedule_id).bind(status).bind(cursor.as_ref().map(|c|c.0)).bind(cursor.as_ref().map(|c|c.0)).bind(cursor.as_ref().map(|c|c.1.as_str())).bind(limit as i64+1).fetch_all(&state.pool).await.map_err(ApiError::db)?
    } else {
        sqlx::query_as::<_,OccurrenceRow>("SELECT o.id,o.schedule_id,s.name AS schedule_name,s.program AS schedule_program,o.status,o.source,o.attempt,o.created_at,o.scheduled_for,o.started_at,o.finished_at,o.exit_code,o.stdout,o.stderr,o.stdout_truncated,o.stderr_truncated,o.missed_count,o.missed_from,o.missed_through,o.retry_of_id FROM occurrences o LEFT JOIN schedules s ON s.id=o.schedule_id WHERE o.schedule_id=? AND (? IS NULL OR (o.created_at,o.id)<(?,?)) ORDER BY o.created_at DESC,o.id DESC LIMIT ?").bind(&schedule_id).bind(cursor.as_ref().map(|c|c.0)).bind(cursor.as_ref().map(|c|c.0)).bind(cursor.as_ref().map(|c|c.1.as_str())).bind(limit as i64+1).fetch_all(&state.pool).await.map_err(ApiError::db)?
    };
    let has_more = rows.len() > limit as usize;
    let next_cursor = has_more.then(|| {
        encode_key_cursor(
            rows[limit as usize - 1].created_at,
            &rows[limit as usize - 1].id,
        )
    });
    let items = rows
        .into_iter()
        .take(limit as usize)
        .map(occurrence_dto)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Json(OccurrencePage { items, next_cursor }))
}

async fn get_occurrence(
    State(state): State<ApiState>,
    Path(id): Path<String>,
) -> Result<Json<OccurrenceDto>, ApiError> {
    fetch_occurrence(&state.pool, &id)
        .await?
        .map(Json)
        .ok_or_else(|| ApiError::not_found("occurrence"))
}

async fn cancel_occurrence(
    State(state): State<ApiState>,
    Path(id): Path<String>,
) -> Result<Response, ApiError> {
    let status: Option<String> = sqlx::query_scalar("SELECT status FROM occurrences WHERE id=?")
        .bind(&id)
        .fetch_optional(&state.pool)
        .await
        .map_err(ApiError::db)?;
    let Some(status) = status else {
        return Err(ApiError::not_found("occurrence"));
    };
    if status == "queued" {
        let changed = sqlx::query("UPDATE occurrences SET status='cancelled',finished_at=? WHERE id=? AND status='queued'").bind(now_micros()?).bind(&id).execute(&state.pool).await.map_err(ApiError::db)?;
        if changed.rows_affected() == 0 {
            let current: String = sqlx::query_scalar("SELECT status FROM occurrences WHERE id=?")
                .bind(&id)
                .fetch_one(&state.pool)
                .await
                .map_err(ApiError::db)?;
            if current == "running" && (state.cancellation)(id.clone()) {
                let dto = fetch_occurrence(&state.pool, &id)
                    .await?
                    .ok_or_else(|| ApiError::not_found("occurrence"))?;
                return Ok((StatusCode::ACCEPTED, Json(dto)).into_response());
            }
            return Err(ApiError::new(
                StatusCode::CONFLICT,
                "invalid_state",
                format!("Occurrence is {current}; cancellation was not applied"),
            ));
        }
        let dto = fetch_occurrence(&state.pool, &id)
            .await?
            .ok_or_else(|| ApiError::not_found("occurrence"))?;
        return Ok((StatusCode::ACCEPTED, Json(dto)).into_response());
    }
    if status != "running" {
        return Err(ApiError::new(
            StatusCode::CONFLICT,
            "invalid_state",
            format!("Occurrence is {status}, not running"),
        ));
    }
    if !(state.cancellation)(id.clone()) {
        return Err(ApiError::new(
            StatusCode::CONFLICT,
            "invalid_state",
            "Cancellation was requested before the child process became cancellable",
        ));
    }
    let dto = fetch_occurrence(&state.pool, &id)
        .await?
        .ok_or_else(|| ApiError::not_found("occurrence"))?;
    Ok((StatusCode::ACCEPTED, Json(dto)).into_response())
}

async fn retry_occurrence(
    State(state): State<ApiState>,
    Path(id): Path<String>,
    body: Option<Json<RetryRequest>>,
) -> Result<Response, ApiError> {
    let row=sqlx::query_as::<_,(String,String,i64,Option<i64>,Option<i64>,Option<i64>)>("SELECT schedule_id,status,attempt,missed_count,missed_from,missed_through FROM occurrences WHERE id=?").bind(&id).fetch_optional(&state.pool).await.map_err(ApiError::db)?.ok_or_else(||ApiError::not_found("occurrence"))?;
    let (schedule_id, status, attempt, count, from, through) = row;
    if status != "failed" && status != "interrupted" {
        return Err(ApiError::new(
            StatusCode::CONFLICT,
            "invalid_state",
            format!("Cannot retry occurrence in state {status}"),
        ));
    }
    if status == "interrupted"
        && !body
            .as_ref()
            .is_some_and(|Json(value)| value.confirm_interrupted)
    {
        return Err(ApiError::new(
            StatusCode::CONFLICT,
            "interrupted_confirmation_required",
            "Retrying an interrupted process requires confirmInterrupted=true",
        ));
    }
    let retry_id = insert_manual_occurrence(
        &state.pool,
        &schedule_id,
        "retry",
        Some((id, attempt + 1, count, from, through)),
    )
    .await?;
    let dto = fetch_occurrence(&state.pool, &retry_id)
        .await?
        .ok_or_else(|| ApiError::not_found("occurrence"))?;
    Ok((StatusCode::ACCEPTED, Json(dto)).into_response())
}

async fn insert_manual_occurrence(
    pool: &SqlitePool,
    schedule_id: &str,
    source: &str,
    retry: Option<RetryMetadata>,
) -> Result<String, ApiError> {
    let id = Uuid::new_v4().to_string();
    let now = now_micros()?;
    let (retry_of, attempt, count, from, through) = retry
        .map(|(id, attempt, count, from, through)| (Some(id), attempt, count, from, through))
        .unwrap_or((None, 1, None, None, None));
    let inserted = sqlx::query("INSERT INTO occurrences(id,schedule_id,status,source,attempt,retry_of_id,scheduled_for,created_at,missed_count,missed_from,missed_through) SELECT ?,id,'queued',?,?,?,?,?,?,?,? FROM schedules WHERE id=? AND status!='cancelled'")
        .bind(&id).bind(source).bind(attempt).bind(retry_of).bind(now).bind(now).bind(count).bind(from).bind(through).bind(schedule_id).execute(pool).await.map_err(ApiError::db)?;
    if inserted.rows_affected() == 0 {
        return Err(ApiError::new(
            StatusCode::CONFLICT,
            "invalid_state",
            "Cancelled schedules cannot enqueue new executions",
        ));
    }
    Ok(id)
}

struct ScheduleRow {
    id: String,
    name: Option<String>,
    program: String,
    args_json: String,
    timing_json: String,
    parameters_json: String,
    missed_policy: String,
    parameter_update_trigger_json: String,
    retry_policy_json: String,
    status: String,
    created_at: i64,
    updated_at: i64,
}

impl FromRow<'_, sqlx::sqlite::SqliteRow> for ScheduleRow {
    fn from_row(row: &sqlx::sqlite::SqliteRow) -> Result<Self, sqlx::Error> {
        Ok(Self {
            id: row.try_get("id")?,
            name: row.try_get("name")?,
            program: row.try_get("program")?,
            args_json: row.try_get("args_json")?,
            timing_json: row.try_get("timing_json")?,
            parameters_json: row.try_get("parameters_json")?,
            missed_policy: row.try_get("missed_policy")?,
            parameter_update_trigger_json: row.try_get("parameter_update_trigger_json")?,
            retry_policy_json: row.try_get("retry_policy_json")?,
            status: row.try_get("status")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        })
    }
}

async fn fetch_schedule(pool: &SqlitePool, id: &str) -> Result<Option<ScheduleDto>, ApiError> {
    let row=sqlx::query_as::<_,ScheduleRow>("SELECT id,name,program,args_json,timing_json,parameters_json,missed_policy,parameter_update_trigger_json,retry_policy_json,status,created_at,updated_at FROM schedules WHERE id=?").bind(id).fetch_optional(pool).await.map_err(ApiError::db)?;
    row.map(schedule_dto).transpose()
}

fn schedule_dto(row: ScheduleRow) -> Result<ScheduleDto, ApiError> {
    Ok(ScheduleDto {
        id: row.id,
        name: row.name,
        program: row.program,
        args: serde_json::from_str(&row.args_json).map_err(|e| ApiError::bad(e.to_string()))?,
        timing: serde_json::from_str(&row.timing_json).map_err(|e| ApiError::bad(e.to_string()))?,
        parameters: serde_json::from_str(&row.parameters_json)
            .map_err(|e| ApiError::bad(e.to_string()))?,
        missed_policy: row.missed_policy,
        parameter_update_trigger: serde_json::from_str(&row.parameter_update_trigger_json)
            .map_err(|e| ApiError::bad(e.to_string()))?,
        retry_policy: serde_json::from_str(&row.retry_policy_json)
            .map_err(|e| ApiError::bad(e.to_string()))?,
        status: row.status,
        created_at: format_timestamp(row.created_at)?,
        updated_at: format_timestamp(row.updated_at)?,
    })
}

async fn schedule_exists(pool: &SqlitePool, id: &str) -> Result<bool, ApiError> {
    sqlx::query_scalar::<_, i64>("SELECT 1 FROM schedules WHERE id=?")
        .bind(id)
        .fetch_optional(pool)
        .await
        .map(|row| row.is_some())
        .map_err(ApiError::db)
}

struct OccurrenceRow {
    id: String,
    schedule_id: String,
    schedule_name: Option<String>,
    schedule_program: Option<String>,
    status: String,
    source: String,
    attempt: i64,
    created_at: i64,
    scheduled_for: Option<i64>,
    started_at: Option<i64>,
    finished_at: Option<i64>,
    exit_code: Option<i32>,
    stdout: Option<Vec<u8>>,
    stderr: Option<Vec<u8>>,
    stdout_truncated: i64,
    stderr_truncated: i64,
    missed_count: Option<i64>,
    missed_from: Option<i64>,
    missed_through: Option<i64>,
    retry_of_id: Option<String>,
}

impl FromRow<'_, sqlx::sqlite::SqliteRow> for OccurrenceRow {
    fn from_row(row: &sqlx::sqlite::SqliteRow) -> Result<Self, sqlx::Error> {
        Ok(Self {
            id: row.try_get("id")?,
            schedule_id: row.try_get("schedule_id")?,
            schedule_name: row.try_get("schedule_name")?,
            schedule_program: row.try_get("schedule_program")?,
            status: row.try_get("status")?,
            source: row.try_get("source")?,
            attempt: row.try_get("attempt")?,
            created_at: row.try_get("created_at")?,
            scheduled_for: row.try_get("scheduled_for")?,
            started_at: row.try_get("started_at")?,
            finished_at: row.try_get("finished_at")?,
            exit_code: row.try_get("exit_code")?,
            stdout: row.try_get("stdout")?,
            stderr: row.try_get("stderr")?,
            stdout_truncated: row.try_get("stdout_truncated")?,
            stderr_truncated: row.try_get("stderr_truncated")?,
            missed_count: row.try_get("missed_count")?,
            missed_from: row.try_get("missed_from")?,
            missed_through: row.try_get("missed_through")?,
            retry_of_id: row.try_get("retry_of_id")?,
        })
    }
}

async fn fetch_occurrence(pool: &SqlitePool, id: &str) -> Result<Option<OccurrenceDto>, ApiError> {
    let row=sqlx::query_as::<_,OccurrenceRow>("SELECT o.id,o.schedule_id,s.name AS schedule_name,s.program AS schedule_program,o.status,o.source,o.attempt,o.created_at,o.scheduled_for,o.started_at,o.finished_at,o.exit_code,o.stdout,o.stderr,o.stdout_truncated,o.stderr_truncated,o.missed_count,o.missed_from,o.missed_through,o.retry_of_id FROM occurrences o LEFT JOIN schedules s ON s.id=o.schedule_id WHERE o.id=?").bind(id).fetch_optional(pool).await.map_err(ApiError::db)?;
    row.map(occurrence_dto).transpose()
}

fn occurrence_dto(row: OccurrenceRow) -> Result<OccurrenceDto, ApiError> {
    let missed_occurrences = match (row.missed_count, row.missed_from, row.missed_through) {
        (Some(count), Some(from), Some(through)) => Some(MissedOccurrences {
            count,
            from: format_timestamp(from)?,
            through: format_timestamp(through)?,
        }),
        _ => None,
    };
    Ok(OccurrenceDto {
        id: row.id,
        schedule_id: row.schedule_id,
        schedule_name: row.schedule_name,
        schedule_program: row.schedule_program,
        status: row.status,
        source: row.source,
        attempt: row.attempt,
        created_at: format_timestamp(row.created_at)?,
        scheduled_for: row.scheduled_for.map(format_timestamp).transpose()?,
        started_at: row.started_at.map(format_timestamp).transpose()?,
        finished_at: row.finished_at.map(format_timestamp).transpose()?,
        exit_code: row.exit_code,
        stdout: row
            .stdout
            .map(|bytes| String::from_utf8_lossy(&bytes).into_owned()),
        stderr: row
            .stderr
            .map(|bytes| String::from_utf8_lossy(&bytes).into_owned()),
        stdout_truncated: row.stdout_truncated != 0,
        stderr_truncated: row.stderr_truncated != 0,
        missed_occurrences,
        retry_of: row.retry_of_id,
    })
}

fn validate_timing(timing: &Value) -> Result<(), ApiError> {
    let object = timing
        .as_object()
        .ok_or_else(|| ApiError::bad("timing must be a JSON object"))?;
    match timing.get("type").and_then(Value::as_str) {
        Some("cron") => {
            if object.len() != 2
                || object
                    .keys()
                    .any(|key| key != "type" && key != "expression")
            {
                return Err(ApiError::bad(
                    "cron timing accepts only type and expression",
                ));
            }
            let expression = timing
                .get("expression")
                .and_then(Value::as_str)
                .ok_or_else(|| ApiError::bad("cron timing requires expression"))?;
            parse_cron(expression).map_err(|e| ApiError::bad(e.to_string()))?;
        }
        Some("once") => {
            if object.len() != 2 || object.keys().any(|key| key != "type" && key != "at") {
                return Err(ApiError::bad("once timing accepts only type and at"));
            }
            let at = timing
                .get("at")
                .and_then(Value::as_str)
                .ok_or_else(|| ApiError::bad("once timing requires at"))?;
            DateTime::parse_from_rfc3339(at).map_err(|e| ApiError::bad(e.to_string()))?;
        }
        _ => return Err(ApiError::bad("timing type must be once or cron")),
    }
    Ok(())
}
fn validate_trigger(value: &Value) -> Result<(), ApiError> {
    let object = value
        .as_object()
        .ok_or_else(|| ApiError::bad("parameterUpdateTrigger must be an object"))?;
    if object
        .keys()
        .any(|key| key != "enabled" && key != "scheduleBehavior")
        || object.len() != 2
    {
        return Err(ApiError::bad(
            "parameterUpdateTrigger requires enabled and scheduleBehavior only",
        ));
    }
    if !object.get("enabled").is_some_and(Value::is_boolean) {
        return Err(ApiError::bad(
            "parameterUpdateTrigger.enabled must be boolean",
        ));
    }
    if !matches!(
        object.get("scheduleBehavior").and_then(Value::as_str),
        Some("replace" | "coexist")
    ) {
        return Err(ApiError::bad("scheduleBehavior must be replace or coexist"));
    }
    Ok(())
}
fn validate_retry_policy(value: &Value) -> Result<(), ApiError> {
    let object = value
        .as_object()
        .ok_or_else(|| ApiError::bad("retryPolicy must be an object"))?;
    if object
        .keys()
        .any(|key| key != "maxRetries" && key != "delaysSeconds")
    {
        return Err(ApiError::bad("retryPolicy contains unsupported fields"));
    }
    if object
        .get("maxRetries")
        .is_some_and(|value| value.as_u64().is_none())
    {
        return Err(ApiError::bad("maxRetries must be a non-negative integer"));
    }
    if let Some(delays) = object.get("delaysSeconds") {
        let delays = delays
            .as_array()
            .ok_or_else(|| ApiError::bad("delaysSeconds must be an array"))?;
        if delays
            .iter()
            .any(|delay| delay.as_u64().is_none_or(|value| value == 0))
        {
            return Err(ApiError::bad("retry delays must be positive integers"));
        }
    }
    Ok(())
}
fn validate_policy(value: &str) -> Result<(), ApiError> {
    if matches!(value, "skip" | "consolidate" | "sequence") {
        Ok(())
    } else {
        Err(ApiError::bad(
            "missedPolicy must be skip, consolidate, or sequence",
        ))
    }
}
fn valid_occurrence_status(value: &str) -> bool {
    matches!(
        value,
        "queued" | "running" | "succeeded" | "failed" | "cancelled" | "interrupted"
    )
}
fn valid_schedule_status(value: &str) -> bool {
    matches!(value, "active" | "paused" | "cancelled")
}
fn parse_bound(value: Option<&str>) -> Result<Option<i64>, ApiError> {
    value
        .map(|s| {
            DateTime::parse_from_rfc3339(s)
                .map(|d| d.timestamp_micros())
                .map_err(|_| ApiError::bad("date bounds must be RFC 3339 timestamps"))
        })
        .transpose()
}
fn validate_range(from: i64, to: i64) -> Result<(), ApiError> {
    if to <= from {
        Err(ApiError::bad("to must be later than from"))
    } else {
        Ok(())
    }
}
fn iso_from_micros(value: i64) -> Result<String, ApiError> {
    format_timestamp(value)
}
fn page_limit(value: Option<u64>) -> Result<u64, ApiError> {
    let value = value.unwrap_or(DEFAULT_LIMIT);
    if value == 0 || value > MAX_LIMIT {
        Err(ApiError::bad("limit must be between 1 and 100"))
    } else {
        Ok(value)
    }
}
fn decode_key_cursor(cursor: Option<&str>) -> Result<Option<(i64, String)>, ApiError> {
    let Some(value) = cursor else { return Ok(None) };
    let bytes = URL_SAFE_NO_PAD
        .decode(value)
        .map_err(|_| ApiError::bad("cursor is invalid"))?;
    let decoded: (i64, String) =
        serde_json::from_slice(&bytes).map_err(|_| ApiError::bad("cursor is invalid"))?;
    Ok(Some(decoded))
}
fn encode_key_cursor(created_at: i64, id: &str) -> String {
    URL_SAFE_NO_PAD.encode(serde_json::to_vec(&(created_at, id)).expect("cursor tuple serializes"))
}
fn format_timestamp(value: i64) -> Result<String, ApiError> {
    DateTime::<Utc>::from_timestamp_micros(value)
        .map(|time| time.to_rfc3339())
        .ok_or_else(|| ApiError::bad("timestamp is out of range"))
}
fn now_micros() -> std::result::Result<i64, ApiError> {
    let duration = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| ApiError::bad(e.to_string()))?;
    i64::try_from(duration.as_micros()).map_err(|e| ApiError::bad(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::to_bytes,
        http::{Method, Request},
    };
    use tower::ServiceExt;

    async fn test_router() -> (tempfile::TempDir, Router, SqlitePool) {
        let directory = tempfile::tempdir().unwrap();
        let pool = crate::storage::connect_at(&directory.path().join("api.sqlite3"))
            .await
            .unwrap();
        let state = ApiState {
            pool: pool.clone(),
            cancellation: Arc::new(|_| false),
        };
        (directory, router(state), pool)
    }

    #[tokio::test]
    async fn api_creates_and_replays_idempotent_schedule() {
        let (_directory, app, _pool) = test_router().await;
        let body = r#"{"program":"/bin/true","args":[],"timing":{"type":"cron","expression":"0 8 * * 1-5"},"missedPolicy":"sequence"}"#;
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/schedules")
                    .header(header::CONTENT_TYPE, "application/json")
                    .header("Idempotency-Key", "create-1")
                    .body(Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);
        let payload: Value =
            serde_json::from_slice(&to_bytes(response.into_body(), usize::MAX).await.unwrap())
                .unwrap();
        assert_eq!(payload["timing"]["expression"], "0 8 * * 1-5");
        let response = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/schedules")
                    .header(header::CONTENT_TYPE, "application/json")
                    .header("Idempotency-Key", "create-1")
                    .body(Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn api_rejects_invalid_cron_and_external_browser_origin() {
        let (_directory, app, _pool) = test_router().await;
        let response = app.clone().oneshot(Request::builder().method(Method::POST).uri("/schedules").header(header::CONTENT_TYPE,"application/json").body(Body::from(r#"{"program":"/bin/true","args":[],"timing":{"type":"cron","expression":"0 8 * * MON"},"missedPolicy":"skip"}"#)).unwrap()).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/schedules")
                    .header(header::HOST, "127.0.0.1:30002")
                    .header(header::ORIGIN, "https://example.com")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);

        let (_directory, app, _pool) = test_router().await;
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/schedules")
                    .body(Body::from("{}"))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::UNSUPPORTED_MEDIA_TYPE);
        let response = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/schedules")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from("{"))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn api_accepts_matching_tailscale_origin_for_workspace_proxy() {
        let (_directory, app, _pool) = test_router().await;
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/schedules")
                    .header(header::HOST, "100.94.53.63:30014")
                    .header(header::ORIGIN, "http://100.94.53.63:30014")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/schedules")
                    .header(header::HOST, "192.168.1.20:30014")
                    .header(header::ORIGIN, "http://192.168.1.20:30014")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn api_requires_confirmation_before_retrying_interrupted_occurrence() {
        let (_directory, app, pool) = test_router().await;
        let response=app.clone().oneshot(Request::builder().method(Method::POST).uri("/schedules").header(header::CONTENT_TYPE,"application/json").body(Body::from(r#"{"program":"/bin/true","args":[],"timing":{"type":"cron","expression":"0 8 * * *"},"missedPolicy":"skip"}"#)).unwrap()).await.unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);
        let schedule_id: String = sqlx::query_scalar("SELECT id FROM schedules")
            .fetch_one(&pool)
            .await
            .unwrap();
        let occurrence_id = Uuid::new_v4().to_string();
        sqlx::query("INSERT INTO occurrences(id,schedule_id,status,source,created_at) VALUES(?,?,'interrupted','schedule',?)").bind(&occurrence_id).bind(schedule_id).bind(now_micros().unwrap()).execute(&pool).await.unwrap();
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri(format!("/occurrences/{occurrence_id}/retry"))
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from("{}"))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::CONFLICT);
        let response = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri(format!("/occurrences/{occurrence_id}/retry"))
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(r#"{"confirmInterrupted":true}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::ACCEPTED);
    }

    #[tokio::test]
    async fn global_operational_queries_filter_and_return_active_agenda() {
        let (_directory, app, pool) = test_router().await;
        let response = app.clone().oneshot(Request::builder().method(Method::POST).uri("/schedules").header(header::CONTENT_TYPE,"application/json").body(Body::from(r#"{"name":"daily","program":"/bin/true","args":[],"timing":{"type":"cron","expression":"0 9 * * *"},"missedPolicy":"skip"}"#)).unwrap()).await.unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);
        let schedule_id: String = sqlx::query_scalar("SELECT id FROM schedules")
            .fetch_one(&pool)
            .await
            .unwrap();
        let now = now_micros().unwrap();
        let occurrence_id = Uuid::new_v4().to_string();
        sqlx::query("INSERT INTO occurrences(id,schedule_id,status,source,scheduled_for,created_at,finished_at,stdout) VALUES(?,?,'succeeded','manual',?,?,?,?)")
            .bind(&occurrence_id).bind(&schedule_id).bind(now).bind(now).bind(now).bind(b"done".as_slice()).execute(&pool).await.unwrap();
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!(
                        "/overview?from={}&to={}",
                        iso_from_micros(now - 60_000_000)
                            .unwrap()
                            .replace("+00:00", "Z"),
                        iso_from_micros(now + 60_000_000)
                            .unwrap()
                            .replace("+00:00", "Z")
                    ))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let overview: Value =
            serde_json::from_slice(&to_bytes(response.into_body(), usize::MAX).await.unwrap())
                .unwrap();
        assert_eq!(overview["schedules"]["active"], 1);
        assert_eq!(overview["next"]["scheduleId"], schedule_id);
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/schedules?status=paused")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let page: Value =
            serde_json::from_slice(&to_bytes(response.into_body(), usize::MAX).await.unwrap())
                .unwrap();
        assert_eq!(page["items"].as_array().unwrap().len(), 0);
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!(
                        "/agenda?from={}&to={}",
                        iso_from_micros(now).unwrap().replace("+00:00", "Z"),
                        iso_from_micros(now + 3 * 24 * 60 * 60 * 1_000_000)
                            .unwrap()
                            .replace("+00:00", "Z")
                    ))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let agenda: Value =
            serde_json::from_slice(&to_bytes(response.into_body(), usize::MAX).await.unwrap())
                .unwrap();
        assert!(!agenda["items"].as_array().unwrap().is_empty());
        assert!(
            agenda["items"]
                .as_array()
                .unwrap()
                .iter()
                .any(|item| item["occurrenceId"] == occurrence_id)
        );
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!(
                        "/occurrences?status=succeeded&scheduleId={schedule_id}&from={}&to={}",
                        iso_from_micros(now - 60_000_000)
                            .unwrap()
                            .replace("+00:00", "Z"),
                        iso_from_micros(now + 60_000_000)
                            .unwrap()
                            .replace("+00:00", "Z")
                    ))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let global: Value =
            serde_json::from_slice(&to_bytes(response.into_body(), usize::MAX).await.unwrap())
                .unwrap();
        assert_eq!(global["items"].as_array().unwrap().len(), 1);
        assert!(global["items"][0].get("stdout").is_none());
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/occurrences?status=bogus")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }
}
