use crate::convert_paths::resolve_and_validate_convert_paths;
use crate::execute_auth::authorize_execute;
use crate::jobs::{ConvertJobRequest, JobStatus, spawn_convert};
use crate::jobs::run_convert_async;
use crate::state::AppState;
use axum::extract::{ConnectInfo, Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Json};
use axum::routing::{get, get_service, post};
use axum::Router;
use lpmr_core::devices::scan_block_devices;
use lpmr_core::i18n::{Language, translate};
use lpmr_core::lvm::{lvm_available, scan_lvm};
use lpmr_core::ops::{execute, OperationKind, OperationRequest};
use lpmr_core::partition_map::partition_map_for_disk;
use lpmr_core::proxmox::{is_proxmox_node, list_storages, list_storage_volumes, resolve_volume_path};
use lpmr_core::qemu::{image_info, QemuFormat};
use lpmr_core::telemetry::host_telemetry;
use lpmr_core::themes::enterprise_themes;
use lpmr_core::tools::{discover_installed_tools, required_tools, validate_qemu_image_path, QemuPathRole};
use lpmr_core::zfs::{list_pools, zfs_available};
use serde::Deserialize;
use serde_json::json;
use tower_http::services::{ServeDir, ServeFile};

#[derive(Deserialize)]
struct ImageQuery {
    path: String,
}

#[derive(Clone, Deserialize)]
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
    fstype: Option<lpmr_core::filesystems::FilesystemKind>,
    image_format: Option<QemuFormat>,
}

pub fn api_router() -> Router<AppState> {
    Router::new()
        .route("/api/session", get(session))
        .route("/api/health", get(health))
        .route("/api/devices", get(devices))
        .route("/api/lvm", get(lvm))
        .route("/api/tools", get(tools))
        .route("/api/themes", get(themes))
        .route("/api/i18n/:lang", get(i18n_bundle))
        .route("/api/proxmox/storages", get(proxmox_storages))
        .route("/api/proxmox/resolve-path", get(proxmox_resolve_path))
        .route("/api/proxmox/storage/:name/volumes", get(proxmox_volumes))
        .route("/api/qemu/info", get(qemu_info))
        .route("/api/qemu/formats", get(qemu_formats))
        .route("/api/operations", post(run_operation))
        .route("/api/telemetry", get(telemetry))
        .route("/api/partitions", get(partitions))
        .route("/api/zfs/pools", get(zfs_pools))
        .route("/api/health-audit", get(health_audit))
        .route("/api/vm-volumes", get(vm_volumes))
        .route("/api/jobs/convert", post(job_convert))
        .route("/api/jobs/:id", get(job_status))
}

pub fn static_router(assets_dir: std::path::PathBuf) -> Router<AppState> {
    let web_root = assets_dir.join("web");
    let index_page = web_root.join("index.html");
    Router::new()
        .route_service("/", get_service(ServeFile::new(index_page.clone())))
        .nest_service("/assets", ServeDir::new(web_root.clone()))
        .fallback_service(ServeFile::new(index_page))
}

async fn session(State(state): State<AppState>) -> Json<serde_json::Value> {
    Json(json!({
        "csrf": state.execute_csrf,
        "execute_policy": "non-dry-run requires X-LPMR-CSRF, loopback client, LPMR_EXECUTE_TOKEN, or LPMR_ALLOW_REMOTE_EXECUTE",
    }))
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
    Ok(Json(json!({
        "available": lvm_available(),
        "pvs": snap.pvs,
        "vgs": snap.vgs,
        "lvs": snap.lvs,
    })))
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
        "nav.dashboard",
        "nav.devices",
        "nav.devices.sub",
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

#[derive(Deserialize)]
struct DiskQuery {
    disk: String,
}

async fn telemetry() -> Json<serde_json::Value> {
    let host = host_telemetry();
    Json(json!({
        "host": host,
        "engine": "Rust Axum",
        "features": ["tokio", "tower-http compression", "lsblk cache"]
    }))
}

async fn partitions(Query(q): Query<DiskQuery>) -> Result<Json<serde_json::Value>, AppError> {
    let map = partition_map_for_disk(&q.disk).ok_or_else(|| anyhow::anyhow!("disk not found"))?;
    Ok(Json(json!(map)))
}

async fn zfs_pools() -> Json<serde_json::Value> {
    if !zfs_available() {
        return Json(json!({"available": false, "pools": []}));
    }
    match list_pools() {
        Ok(pools) => Json(json!({"available": true, "pools": pools})),
        Err(e) => Json(json!({
            "available": true,
            "pools": [],
            "error": e.to_string(),
        })),
    }
}

async fn health_audit() -> Json<serde_json::Value> {
    let installed = discover_installed_tools();
    let missing: Vec<_> = required_tools()
        .into_iter()
        .filter(|(name, _)| !installed.iter().any(|t| t.name == *name))
        .map(|(name, purpose)| json!({"name": name, "purpose": purpose, "ok": false}))
        .collect();
    let present: Vec<_> = installed
        .iter()
        .map(|t| json!({"name": t.name, "purpose": t.purpose, "ok": true, "path": t.path}))
        .collect();
    let (device_count, devices_error) = match scan_block_devices() {
        Ok(tree) => (tree.devices.len(), None),
        Err(e) => (0, Some(e.to_string())),
    };
    Json(json!({
        "tools_ok": present,
        "tools_missing": missing,
        "device_count": device_count,
        "zfs": zfs_available(),
        "proxmox": is_proxmox_node(),
        "error": devices_error,
    }))
}

async fn vm_volumes() -> Json<serde_json::Value> {
    if !is_proxmox_node() {
        return Json(json!({"volumes": []}));
    }
    let storages = match list_storages() {
        Ok(s) => s,
        Err(e) => {
            return Json(json!({
                "volumes": [],
                "error": e.to_string(),
            }));
        }
    };
    let mut volumes = Vec::new();
    for storage in storages {
        let list = list_storage_volumes(&storage.name).unwrap_or_default();
        for vol in list {
            volumes.push(json!({
                "volid": vol.volid,
                "format": vol.format,
                "size": vol.size,
                "storage": storage.name,
                "path": resolve_volume_path(&vol.volid).ok(),
            }));
        }
    }
    Json(json!({"volumes": volumes}))
}

#[derive(Deserialize)]
struct VolIdQuery {
    volid: String,
}

async fn proxmox_resolve_path(Query(q): Query<VolIdQuery>) -> Result<Json<serde_json::Value>, AppError> {
    if !is_proxmox_node() {
        return Err(anyhow::anyhow!("not a Proxmox node").into());
    }
    let path = resolve_volume_path(&q.volid)?;
    Ok(Json(json!({
        "volid": q.volid,
        "path": path,
    })))
}

async fn job_convert(
    State(state): State<AppState>,
    ConnectInfo(peer): ConnectInfo<std::net::SocketAddr>,
    headers: HeaderMap,
    Json(body): Json<ConvertJobRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let dry = body.dry_run.unwrap_or(true);
    if !dry {
        authorize_execute(Some(peer), &headers, &state.execute_csrf)?;
    }
    let (source, dest) = resolve_and_validate_convert_paths(&body.source, &body.dest)
        .map_err(AppError)?;
    let mut req = body;
    req.source = source.clone();
    req.dest = dest.clone();
    let record = spawn_convert(req.clone()).await;
    let id = record.id.clone();
    state.jobs.write().await.insert(id.clone(), record.clone());
    if !record.dry_run {
        let jobs = state.jobs.clone();
        let job_id = id.clone();
        let format = req.format.unwrap_or_else(|| "qcow2".into());
        tokio::spawn(async move {
            let result = run_convert_async(source, dest, format, |pct, line| {
                let jobs = jobs.clone();
                let job_id = job_id.clone();
                let line = line.to_string();
                tokio::spawn(async move {
                    let mut map = jobs.write().await;
                    if let Some(job) = map.get_mut(&job_id) {
                        job.progress_pct = pct;
                        job.message = format!("converting ({pct:.1}%)");
                        if job.logs.last() != Some(&line) {
                            job.logs.push(line);
                        }
                    }
                });
            })
            .await;
            let mut map = jobs.write().await;
            if let Some(job) = map.get_mut(&job_id) {
                match result {
                    Ok((msg, logs, pct)) => {
                        job.status = JobStatus::Done;
                        job.progress_pct = pct;
                        job.message = msg;
                        job.logs.extend(logs);
                    }
                    Err(e) => {
                        job.status = JobStatus::Failed;
                        job.message = e.to_string();
                    }
                }
            }
        });
    }
    Ok(Json(json!(record)))
}

async fn job_status(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let map = state.jobs.read().await;
    let job = map
        .get(&id)
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("job not found"))?;
    Ok(Json(json!(job)))
}

async fn run_operation(
    State(state): State<AppState>,
    ConnectInfo(peer): ConnectInfo<std::net::SocketAddr>,
    headers: HeaderMap,
    Json(body): Json<OpBody>,
) -> Result<Json<serde_json::Value>, AppError> {
    let dry = body.dry_run.unwrap_or(true);
    if !dry {
        authorize_execute(Some(peer), &headers, &state.execute_csrf)?;
    }
    if body.kind == OperationKind::ConvertImage {
        let source = body
            .source
            .as_deref()
            .ok_or_else(|| anyhow::anyhow!("source required"))?;
        let dest = body
            .dest
            .as_deref()
            .ok_or_else(|| anyhow::anyhow!("dest required"))?;
        let (s, d) = resolve_and_validate_convert_paths(source, dest)?;
        let body = OpBody {
            source: Some(s),
            dest: Some(d),
            ..body
        };
        return run_operation_inner(body, dry).await;
    }
    run_operation_inner(body, dry).await
}

async fn run_operation_inner(body: OpBody, dry: bool) -> Result<Json<serde_json::Value>, AppError> {
    if body.kind == OperationKind::CreateImage {
        if let Some(ref path) = body.path {
            validate_qemu_image_path(path, QemuPathRole::Dest)?;
        }
    }
    let req = OperationRequest {
        kind: body.kind,
        dry_run: dry,
        device: body.device,
        path: body.path,
        target: body.target,
        size: body.size,
        fstype: body.fstype,
        partition_table: None,
        start: None,
        end: None,
        mount_point: body.mount_point,
        options: None,
        source: body.source,
        dest: body.dest,
        image_format: body.image_format,
        vg: body.vg,
        lv: body.lv,
        luks_name: None,
    };
    let result = execute(req)?;
    Ok(Json(json!(result)))
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
