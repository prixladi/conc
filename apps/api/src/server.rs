use std::{fs, io::Read, io::Seek, io::SeekFrom, path::Path, sync::Arc};

use app_config::RestartType;
use axum::{
    body::Bytes,
    extract::{Path as UrlPath, Request, State},
    http::{header, HeaderMap, HeaderValue, Method},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use daemon_client::{ErrorResponse, Requester};
use project_settings::ProjectSettings;
use serde::Deserialize;
use serde_json::{json, Value};
use tower_http::{cors::CorsLayer, trace::TraceLayer};

use crate::{
    dto::{Project, Service},
    error::{ApiError, ApiQuery},
    keys::{KeyStore, Principal},
};

const MAX_LOG_LINES: usize = 10_000;
const MAX_LOG_BYTES: u64 = 4 * 1024 * 1024;

#[derive(Clone)]
pub struct AppState {
    pub requester: Requester,
    pub keys: Arc<KeyStore>,
    pub default_restart: RestartType,
}

type ApiResult<T> = Result<T, ApiError>;

pub fn router(state: AppState, cors_origins: &[String]) -> Router {
    let api = Router::new()
        .route("/projects", get(list_projects).put(upsert_project))
        .route(
            "/projects/{project}",
            get(get_project).delete(remove_project),
        )
        .route("/projects/{project}/settings", get(get_project_settings))
        .route("/projects/{project}/start", post(start_project))
        .route("/projects/{project}/stop", post(stop_project))
        .route("/projects/{project}/restart", post(restart_project))
        .route("/projects/{project}/clear-logs", post(clear_project_logs))
        .route("/projects/{project}/services/{service}", get(get_service))
        .route(
            "/projects/{project}/services/{service}/start",
            post(start_service),
        )
        .route(
            "/projects/{project}/services/{service}/stop",
            post(stop_service),
        )
        .route(
            "/projects/{project}/services/{service}/restart",
            post(restart_service),
        )
        .route(
            "/projects/{project}/services/{service}/clear-logs",
            post(clear_service_logs),
        )
        .route(
            "/projects/{project}/services/{service}/logs",
            get(service_logs),
        )
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            require_api_key,
        ));

    let mut app = Router::new()
        .route("/health", get(health))
        .nest("/api/v1", api)
        .with_state(state)
        .layer(TraceLayer::new_for_http());

    if !cors_origins.is_empty() {
        let origins: Vec<HeaderValue> = cors_origins
            .iter()
            .filter_map(|o| HeaderValue::from_str(o).ok())
            .collect();
        app = app.layer(
            CorsLayer::new()
                .allow_origin(origins)
                .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
                .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE]),
        );
    }
    app
}

// ---------- auth ----------

async fn require_api_key(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Result<Response, ApiError> {
    let token = bearer_token(req.headers()).ok_or_else(ApiError::unauthorized)?;
    let keys = state.keys.clone();
    let principal = tokio::task::spawn_blocking(move || keys.authenticate(&token))
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?
        .map_err(|e| ApiError::internal(e.to_string()))?
        .ok_or_else(ApiError::unauthorized)?;

    tracing::debug!(key = %principal.key_name, id = principal.key_id, "authenticated");
    req.extensions_mut().insert::<Principal>(principal);
    Ok(next.run(req).await)
}

fn bearer_token(headers: &HeaderMap) -> Option<String> {
    let value = headers.get(header::AUTHORIZATION)?.to_str().ok()?;
    let (scheme, token) = value.split_once(' ')?;
    scheme
        .eq_ignore_ascii_case("bearer")
        .then(|| token.trim().to_string())
        .filter(|t| !t.is_empty())
}

// ---------- helpers ----------

/// Runs a blocking daemon call off the async runtime.
async fn daemon<T, F>(state: &AppState, f: F) -> ApiResult<T>
where
    T: Send + 'static,
    F: FnOnce(&Requester) -> Result<T, ErrorResponse> + Send + 'static,
{
    let requester = state.requester.clone();
    tokio::task::spawn_blocking(move || f(&requester))
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?
        .map_err(ApiError::from)
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
enum RestartParam {
    Hot,
    Cold,
}

#[derive(Debug, Deserialize)]
struct RestartQuery {
    #[serde(rename = "type")]
    restart_type: Option<RestartParam>,
}

impl RestartQuery {
    fn resolve(&self, default: &RestartType) -> RestartType {
        match self.restart_type {
            Some(RestartParam::Hot) => RestartType::Hot,
            Some(RestartParam::Cold) => RestartType::Cold,
            None => default.clone(),
        }
    }
}

// ---------- handlers ----------

async fn health(State(state): State<AppState>) -> Json<Value> {
    let requester = state.requester.clone();
    let daemon_alive = tokio::task::spawn_blocking(move || requester.client().is_alive())
        .await
        .unwrap_or(false);
    Json(json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION"),
        "daemon": if daemon_alive { "up" } else { "down" },
    }))
}

async fn list_projects(State(s): State<AppState>) -> ApiResult<Json<Vec<Project>>> {
    let projects = daemon(&s, |r| r.get_projects_info()).await?;
    Ok(Json(projects.into_iter().map(Project::from).collect()))
}

async fn get_project(
    State(s): State<AppState>,
    UrlPath(p): UrlPath<String>,
) -> ApiResult<Json<Project>> {
    Ok(Json(
        daemon(&s, move |r| r.get_project_info(&p)).await?.into(),
    ))
}

async fn get_project_settings(
    State(s): State<AppState>,
    UrlPath(p): UrlPath<String>,
) -> ApiResult<Json<Value>> {
    let raw = daemon(&s, move |r| r.get_project_settings(&p)).await?;
    let value = serde_json::from_str(&raw)
        .map_err(|e| ApiError::internal(format!("daemon returned invalid settings JSON: {e}")))?;
    Ok(Json(value))
}

/// Body is the same JSON as a conc.json file. Because the API may be used
/// remotely, `cwd` must be an absolute path on the daemon's machine.
async fn upsert_project(State(s): State<AppState>, body: Bytes) -> ApiResult<Json<Project>> {
    let json =
        std::str::from_utf8(&body).map_err(|_| ApiError::bad_request("body must be UTF-8 JSON"))?;
    let settings = ProjectSettings::parse_and_populate(json, String::new())
        .map_err(|e| ApiError::bad_request(format!("invalid project settings: {e}")))?;
    if !Path::new(&settings.cwd).is_absolute() {
        return Err(ApiError::bad_request("'cwd' must be an absolute path"));
    }
    let payload = String::try_from(&settings).map_err(|e| ApiError::internal(e.to_string()))?;
    Ok(Json(
        daemon(&s, move |r| r.upsert_project(&payload))
            .await?
            .into(),
    ))
}

async fn remove_project(
    State(s): State<AppState>,
    UrlPath(p): UrlPath<String>,
) -> ApiResult<impl IntoResponse> {
    daemon(&s, move |r| r.remove_project(&p)).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}

async fn start_project(
    State(s): State<AppState>,
    UrlPath(p): UrlPath<String>,
) -> ApiResult<Json<Project>> {
    Ok(Json(daemon(&s, move |r| r.start_project(&p)).await?.into()))
}

async fn stop_project(
    State(s): State<AppState>,
    UrlPath(p): UrlPath<String>,
) -> ApiResult<Json<Project>> {
    Ok(Json(daemon(&s, move |r| r.stop_project(&p)).await?.into()))
}

async fn restart_project(
    State(s): State<AppState>,
    UrlPath(p): UrlPath<String>,
    ApiQuery(q): ApiQuery<RestartQuery>,
) -> ApiResult<Json<Project>> {
    let rt = q.resolve(&s.default_restart);
    Ok(Json(
        daemon(&s, move |r| r.restart_project(&p, &rt))
            .await?
            .into(),
    ))
}

async fn clear_project_logs(
    State(s): State<AppState>,
    UrlPath(p): UrlPath<String>,
) -> ApiResult<impl IntoResponse> {
    daemon(&s, move |r| r.clear_project_logs(&p)).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}

async fn get_service(
    State(s): State<AppState>,
    UrlPath((p, sv)): UrlPath<(String, String)>,
) -> ApiResult<Json<Service>> {
    Ok(Json(
        daemon(&s, move |r| r.get_services_info(&p, &sv))
            .await?
            .into(),
    ))
}

async fn start_service(
    State(s): State<AppState>,
    UrlPath((p, sv)): UrlPath<(String, String)>,
) -> ApiResult<Json<Service>> {
    Ok(Json(
        daemon(&s, move |r| r.start_service(&p, &sv)).await?.into(),
    ))
}

async fn stop_service(
    State(s): State<AppState>,
    UrlPath((p, sv)): UrlPath<(String, String)>,
) -> ApiResult<Json<Service>> {
    Ok(Json(
        daemon(&s, move |r| r.stop_service(&p, &sv)).await?.into(),
    ))
}

async fn restart_service(
    State(s): State<AppState>,
    UrlPath((p, sv)): UrlPath<(String, String)>,
    ApiQuery(q): ApiQuery<RestartQuery>,
) -> ApiResult<Json<Service>> {
    let rt = q.resolve(&s.default_restart);
    Ok(Json(
        daemon(&s, move |r| r.restart_service(&p, &sv, &rt))
            .await?
            .into(),
    ))
}

async fn clear_service_logs(
    State(s): State<AppState>,
    UrlPath((p, sv)): UrlPath<(String, String)>,
) -> ApiResult<impl IntoResponse> {
    daemon(&s, move |r| r.clear_service_logs(&p, &sv)).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}

#[derive(Debug, Deserialize)]
struct LogsQuery {
    lines: Option<usize>,
}

/// Returns the last `lines` (default 200) lines of the service log as plain text.
async fn service_logs(
    State(s): State<AppState>,
    UrlPath((p, sv)): UrlPath<(String, String)>,
    ApiQuery(q): ApiQuery<LogsQuery>,
) -> ApiResult<Response> {
    let lines = q.lines.unwrap_or(200).clamp(1, MAX_LOG_LINES);
    let info = daemon(&s, move |r| r.get_services_info(&p, &sv)).await?;
    let text = tokio::task::spawn_blocking(move || tail(Path::new(&info.logfile_path), lines))
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?
        .map_err(|e| ApiError::internal(format!("unable to read log file: {e}")))?;
    Ok(([(header::CONTENT_TYPE, "text/plain; charset=utf-8")], text).into_response())
}

fn tail(path: &Path, lines: usize) -> std::io::Result<String> {
    let mut file = match fs::File::open(path) {
        Ok(f) => f,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(String::new()),
        Err(e) => return Err(e),
    };
    let size = file.metadata()?.len();
    let start = size.saturating_sub(MAX_LOG_BYTES);
    file.seek(SeekFrom::Start(start))?;
    let mut buf = Vec::with_capacity((size - start) as usize);
    file.read_to_end(&mut buf)?;
    let text = String::from_utf8_lossy(&buf);

    let mut all: Vec<&str> = text.split_terminator('\n').collect();
    if start > 0 && !all.is_empty() {
        all.remove(0); // likely a partial line
    }
    let skip = all.len().saturating_sub(lines);
    let mut out = all[skip..].join("\n");
    if !out.is_empty() {
        out.push('\n');
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bearer_parsing() {
        let mut h = HeaderMap::new();
        assert_eq!(bearer_token(&h), None);
        h.insert(header::AUTHORIZATION, "Bearer conc_abc".parse().unwrap());
        assert_eq!(bearer_token(&h).as_deref(), Some("conc_abc"));
        h.insert(header::AUTHORIZATION, "bearer  conc_abc ".parse().unwrap());
        assert_eq!(bearer_token(&h).as_deref(), Some("conc_abc"));
        h.insert(header::AUTHORIZATION, "Basic xyz".parse().unwrap());
        assert_eq!(bearer_token(&h), None);
        h.insert(header::AUTHORIZATION, "Bearer ".parse().unwrap());
        assert_eq!(bearer_token(&h), None);
    }

    #[test]
    fn tail_returns_last_lines() {
        let dir = std::env::temp_dir().join(format!("conca-tail-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let f = dir.join("log");
        fs::write(&f, "a\nb\nc\nd\n").unwrap();
        assert_eq!(tail(&f, 2).unwrap(), "c\nd\n");
        assert_eq!(tail(&f, 10).unwrap(), "a\nb\nc\nd\n");
        assert_eq!(tail(&dir.join("missing"), 5).unwrap(), "");
        fs::remove_dir_all(dir).unwrap();
    }
}
