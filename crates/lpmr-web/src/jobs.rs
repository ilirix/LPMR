use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::process::Stdio;
use tokio::io::AsyncReadExt;
use tokio::process::Command as TokioCommand;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JobKind {
    Convert,
    Resize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JobStatus {
    Queued,
    Running,
    Done,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobRecord {
    pub id: String,
    pub kind: JobKind,
    pub status: JobStatus,
    pub progress_pct: f64,
    pub message: String,
    pub logs: Vec<String>,
    pub dry_run: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ConvertJobRequest {
    pub source: String,
    pub dest: String,
    pub format: Option<String>,
    pub dry_run: Option<bool>,
}

pub async fn spawn_convert(req: ConvertJobRequest) -> JobRecord {
    let id = Uuid::new_v4().to_string();
    let dry = req.dry_run.unwrap_or(true);
    let format = req.format.unwrap_or_else(|| "qcow2".into());
    if dry {
        return JobRecord {
            id,
            kind: JobKind::Convert,
            status: JobStatus::Done,
            progress_pct: 100.0,
            message: "dry-run complete".into(),
            logs: vec![format!(
                "dry-run: qemu-img convert -p -O {format} {} {}",
                req.source, req.dest
            )],
            dry_run: true,
        };
    }
    JobRecord {
        id: id.clone(),
        kind: JobKind::Convert,
        status: JobStatus::Running,
        progress_pct: 0.0,
        message: "starting".into(),
        logs: vec!["[CONVERT] spawning qemu-img".into()],
        dry_run: false,
    }
}

pub fn parse_qemu_progress(fragment: &str) -> Option<f64> {
    let t = fragment.trim();
    if let Some(inner) = t.strip_prefix('(').and_then(|s| s.strip_suffix("%)")) {
        if let Some(num) = inner.split('/').next() {
            return num.trim().parse::<f64>().ok();
        }
    }
    if let Some(num) = t.strip_suffix('%') {
        return num.trim().parse::<f64>().ok();
    }
    None
}

pub async fn run_convert_async<F>(source: String, dest: String, format: String, mut on_update: F) -> Result<(String, Vec<String>, f64)>
where
    F: FnMut(f64, &str) + Send,
{
    let mut child = TokioCommand::new("qemu-img")
        .args(["convert", "-p", "-O", &format, &source, &dest])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .context("spawn qemu-img")?;

    let mut stdout = child.stdout.take().context("stdout")?;
    let mut stderr = child.stderr.take().context("stderr")?;

    let mut logs = Vec::new();
    let mut pct = 0.0f64;
    let mut scratch = Vec::with_capacity(512);

    loop {
        scratch.resize(512, 0);
        let n = stdout.read(&mut scratch).await?;
        if n == 0 {
            break;
        }
        let chunk = String::from_utf8_lossy(&scratch[..n]);
        for piece in chunk.split(['\r', '\n']) {
            if piece.is_empty() {
                continue;
            }
            logs.push(piece.to_string());
            if let Some(p) = parse_qemu_progress(piece) {
                pct = p;
                on_update(pct, piece);
            }
        }
    }

    let mut err_buf = Vec::new();
    stderr.read_to_end(&mut err_buf).await?;
    if !err_buf.is_empty() {
        let text = String::from_utf8_lossy(&err_buf);
        for line in text.lines() {
            if !line.trim().is_empty() {
                logs.push(line.to_string());
            }
        }
    }

    let status = child.wait().await?;
    if !status.success() {
        anyhow::bail!("qemu-img convert failed");
    }
    on_update(100.0, "complete");
    Ok((format!("converted to {dest}"), logs, pct.max(100.0)))
}
