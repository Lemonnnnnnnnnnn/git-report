use std::path::{Path, PathBuf};

use axum::extract::{Path as AxumPath, Query, State};
use axum::http::{header, HeaderMap, HeaderValue, StatusCode};
use axum::response::{Html, IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use include_dir::{include_dir, Dir};
use serde::{Deserialize, Serialize};
use tower_http::cors::CorsLayer;

use crate::config::EffectiveConfig;
use crate::model::{Dashboard, Report};
use crate::report;

static WEB_DIST: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/web/dist");

#[derive(Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub open: bool,
    pub report: EffectiveConfig,
}

#[derive(Clone)]
struct AppState {
    config: ServerConfig,
}

#[derive(Debug, Default, Deserialize)]
struct ReportQuery {
    since: Option<String>,
    until: Option<String>,
    branch: Option<String>,
    no_merge: Option<bool>,
    #[serde(default)]
    exclude_dir: Vec<String>,
    #[serde(default)]
    exclude_ext: Vec<String>,
}

#[derive(Debug, Serialize)]
struct Health {
    status: &'static str,
}

pub async fn serve(config: ServerConfig) -> Result<(), String> {
    let state = AppState {
        config: config.clone(),
    };
    let app = Router::new()
        .route("/api/health", get(health))
        .route("/api/report", get(report_handler))
        .route("/api/dashboard", get(dashboard_handler))
        .route("/", get(index))
        .route("/{*path}", get(static_asset))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind((config.host.as_str(), config.port))
        .await
        .map_err(|err| format!("failed to bind web server: {err}"))?;
    let addr = listener
        .local_addr()
        .map_err(|err| format!("failed to read local addr: {err}"))?;
    let url = format!("http://{}", addr);
    println!("Listening on {url}");
    if config.open {
        let _ = webbrowser::open(&url);
    }

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .map_err(|err| format!("web server failed: {err}"))
}

async fn shutdown_signal() {
    let _ = tokio::signal::ctrl_c().await;
}

async fn health() -> Json<Health> {
    Json(Health { status: "ok" })
}

async fn report_handler(
    State(state): State<AppState>,
    Query(query): Query<ReportQuery>,
) -> Result<Json<Report>, String> {
    let config = merge_query(&state.config.report, query);
    report::build_report(&config).map(Json)
}

async fn dashboard_handler(
    State(state): State<AppState>,
    Query(query): Query<ReportQuery>,
) -> Result<Json<Dashboard>, String> {
    let config = merge_query(&state.config.report, query);
    report::build_dashboard(&config).map(Json)
}

async fn index() -> impl IntoResponse {
    serve_embedded("index.html")
}

async fn static_asset(AxumPath(path): AxumPath<String>) -> impl IntoResponse {
    if path.starts_with("api/") {
        return StatusCode::NOT_FOUND.into_response();
    }

    if let Some(response) = serve_asset_path(&path) {
        response
    } else {
        serve_embedded("index.html")
    }
}

fn serve_asset_path(path: &str) -> Option<Response> {
    let normalized = path.trim_start_matches('/');
    if normalized.is_empty() {
        Some(serve_embedded("index.html"))
    } else {
        WEB_DIST.get_file(normalized).map(file_response)
    }
}

fn serve_embedded(path: &str) -> Response {
    WEB_DIST
        .get_file(path)
        .map(file_response)
        .unwrap_or_else(|| {
            Html(
                r#"<!doctype html><html><body><h1>git-report web</h1><p>Frontend assets are missing. Build the web app with <code>npm install && npm run build</code> in <code>web/</code>.</p></body></html>"#,
            )
            .into_response()
        })
}

fn file_response(file: &include_dir::File<'_>) -> Response {
    let mut headers = HeaderMap::new();
    if let Some(content_type) = content_type_for(file.path()) {
        headers.insert(header::CONTENT_TYPE, content_type);
    }
    (headers, file.contents().to_owned()).into_response()
}

fn content_type_for(path: &Path) -> Option<HeaderValue> {
    let content_type = match path.extension().and_then(|ext| ext.to_str()) {
        Some("html") => "text/html; charset=utf-8",
        Some("css") => "text/css; charset=utf-8",
        Some("js") => "application/javascript; charset=utf-8",
        Some("json") => "application/json; charset=utf-8",
        Some("svg") => "image/svg+xml",
        Some("png") => "image/png",
        _ => "application/octet-stream",
    };
    HeaderValue::from_str(content_type).ok()
}

fn merge_query(base: &EffectiveConfig, query: ReportQuery) -> EffectiveConfig {
    let mut exclude_dirs = base.exclude_dirs.clone();
    exclude_dirs.extend(query.exclude_dir);
    let mut exclude_extensions = base.exclude_extensions.clone();
    exclude_extensions.extend(query.exclude_ext);

    EffectiveConfig {
        repo: PathBuf::from(&base.repo),
        since: query.since.or_else(|| base.since.clone()),
        until: query.until.or_else(|| base.until.clone()),
        branch: query.branch.or_else(|| base.branch.clone()),
        no_merge: query.no_merge.unwrap_or(base.no_merge),
        exclude_dirs,
        exclude_extensions,
    }
}
