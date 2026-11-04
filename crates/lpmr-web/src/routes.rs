use crate::state::AppState;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Json};
use axum::routing::{get, post};
use axum::Router;
use lpmr_core::devices::scan_block_devices;
use lpmr_core::i18n::{Language, translate};
use lpmr_core::lvm::scan_lvm;
use lpmr_core::ops::{execute, OperationKind, OperationRequest};
use lpmr_core::proxmox::{is_proxmox_node, list_storages, list_storage_volumes};
use lpmr_core::qemu::{image_info, QemuFormat};
use lpmr_core::themes::{enterprise_themes, theme_by_id};
use lpmr_core::tools::discover_installed_tools;
use serde::Deserialize;
use serde_json::json;
use tower_http::services::{ServeDir, ServeFile};

#[derive(Deserialize)]
struct LangQuery {
    lang: Option<String>,
}

#[derive(Deserialize)]
struct ImageQuery {
    path: String,
}

#[derive(Deserialize)]
struct OpBody {
    kind: OperationKind,
    dry_run: Option<bool>,
    device: Option<String>,
    path: Option<String>,
    target: Option<String>,
    size: Option<String>,
    source: Option<String>,
    dest: Option<String>,
    mount_point: Option<String>,
    vg: Option<String>,
    lv: Option<String>,
}

pub fn api_router() -> Router<AppState> {
    Router::new()
        .route("/api/health", get(health))
        .route("/api/devices", get(devices))
        .route("/api/lvm", get(lvm))
        .route("/api/tools", get(tools))
        .route("/api/themes", get(themes))
        .route("/api/i18n/:lang", get(i18n_bundle))
        .route("/api/proxmox/storages", get(proxmox_storages))
        .route("/api/proxmox/storage/:name/volumes", get(proxmox_volumes))
        .route("/api/qemu/info", get(qemu_info))
        .route("/api/qemu/formats", get(qemu_formats))
        .route("/api/operations", post(run_operation))
}

pub fn static_router() -> Router<AppState> {
    Router::new()
        .route("/", get(index))
        .nest_service("/assets", ServeDir::new("assets/web"))
        .fallback_service(ServeFile::new("assets/web/index.html"))
}

async fn health() -> Json<serde_json::Value> {
    Json(json!({"status":"ok","panel":"lpmr-web"}))
}

async fn devices() -> Result<Json<serde_json::Value>, AppError> {
    let tree = scan_block_devices()?;
    Ok(Json(json!(tree)))
}

async fn lvm() -> Result<Json<serde_json::Value>, AppError> {
    let snap = scan_lvm()?;
    Ok(Json(json!(snap)))
}

async fn tools() -> Json<serde_json::Value> {
    let tools = discover_installed_tools();
    Json(json!({"tools": tools}))
}

async fn themes() -> Json<serde_json::Value> {
    Json(json!({"themes": enterprise_themes()}))
}

async fn i18n_bundle(Path(lang): Path<String>) -> Json<serde_json::Value> {
    let language = Language::from_code(&lang);
    let keys = [
        "app.title",
        "nav.devices",
        "nav.lvm",
        "nav.images",
        "nav.proxmox",
        "nav.operations",
        "nav.tools",
        "action.refresh",
        "action.dry_run",
        "action.execute",
        "panel.details",
        "panel.log",
        "theme.label",
        "lang.label",
        "warn.privilege",
    ];
    let mut map = serde_json::Map::new();
    for key in keys {
        map.insert(key.to_string(), json!(translate(language, key)));
    }
    Json(json!({"lang": language.code(), "strings": map}))
}

async fn proxmox_storages() -> Result<Json<serde_json::Value>, AppError> {
    if !is_proxmox_node() {
        return Ok(Json(json!({"available": false, "storages": []})));
    }
    let storages = list_storages()?;
    Ok(Json(json!({"available": true, "storages": storages})))
}

async fn proxmox_volumes(Path(name): Path<String>) -> Result<Json<serde_json::Value>, AppError> {
    let volumes = list_storage_volumes(&name)?;
    Ok(Json(json!({"storage": name, "volumes": volumes})))
}

async fn qemu_info(Query(q): Query<ImageQuery>) -> Result<Json<serde_json::Value>, AppError> {
    let info = image_info(&q.path)?;
    Ok(Json(json!(info)))
}

async fn qemu_formats() -> Json<serde_json::Value> {
    let formats: Vec<_> = QemuFormat::all().iter().map(|f| f.as_str()).collect();
    Json(json!({"formats": formats}))
}

async fn run_operation(Json(body): Json<OpBody>) -> Result<Json<serde_json::Value>, AppError> {
    let req = OperationRequest {
        kind: body.kind,
        dry_run: body.dry_run.unwrap_or(true),
        device: body.device,
        path: body.path,
        target: body.target,
        size: body.size,
        fstype: None,
        partition_table: None,
        start: None,
        end: None,
        mount_point: body.mount_point,
        options: None,
        source: body.source,
        dest: body.dest,
        image_format: None,
        vg: body.vg,
        lv: body.lv,
        luks_name: None,
    };
    let result = execute(req)?;
    Ok(Json(json!(result)))
}

async fn index(State(state): State<AppState>, Query(q): Query<LangQuery>) -> Html<String> {
    let lang = q
        .lang
        .as_deref()
        .map(Language::from_code)
        .unwrap_or(state.default_lang);
    let theme = theme_by_id(&state.default_theme_id).unwrap_or_else(lpmr_core::themes::default_theme);
    let title = translate(lang, "app.title");
    Html(format!(
        r#"<!DOCTYPE html>
<html lang="{}">
<head>
  <meta charset="utf-8"/>
  <meta name="viewport" content="width=device-width, initial-scale=1"/>
  <title>{title}</title>
  <link rel="stylesheet" href="/assets/app.css"/>
  <style>:root{{{}}}</style>
</head>
<body>
  <div id="app" data-lang="{}" data-theme="{}"></div>
  <script src="/assets/app.js"></script>
</body>
</html>"#,
        lang.code(),
        theme.css_vars(),
        lang.code(),
        theme.id
    ))
}

#[derive(Debug)]
struct AppError(anyhow::Error);

impl From<anyhow::Error> for AppError {
    fn from(value: anyhow::Error) -> Self {
        Self(value)
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": self.0.to_string()})),
        )
            .into_response()
    }
}
