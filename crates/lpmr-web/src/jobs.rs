use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::process::{Command, Stdio};
use tokio::io::{AsyncBufReadExt, BufReader};
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

pub fn run_convert_sync(
    source: &str,
    dest: &str,
    format: &str,
) -> Result<(String, Vec<String>, f64)> {
    let mut child = Command::new("qemu-img")
        .args(["convert", "-p", "-O", format, source, dest])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .context("spawn qemu-img")?;
    let stderr = child.stderr.take();
    let mut logs = Vec::new();
    let mut pct = 0.0f64;
    if let Some(err) = stderr {
        use std::io::{BufRead, BufReader};
        let reader = BufReader::new(err);
        for line in reader.lines().map_while(Result::ok) {
            logs.push(line.clone());
            if let Some(stripped) = line.strip_prefix("    (") {
                if let Some(num) = stripped.strip_suffix("%)") {
                    if let Ok(v) = num.trim().parse::<f64>() {
                        pct = v;
                    }
                }
            }
        }
    }
    let status = child.wait()?;
    if !status.success() {
        anyhow::bail!("qemu-img convert failed");
    }
    Ok((format!("converted to {dest}"), logs, pct.max(100.0)))
}

pub async fn run_convert_async(
    source: String,
    dest: String,
    format: String,
) -> Result<(String, Vec<String>, f64)> {
    let mut child = TokioCommand::new("qemu-img")
        .args(["convert", "-p", "-O", &format, &source, &dest])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    let stderr = child.stderr.take().context("stderr")?;
    let mut reader = BufReader::new(stderr).lines();
    let mut logs = Vec::new();
    let mut pct = 0.0;
    while let Some(line) = reader.next_line().await? {
        logs.push(line.clone());
        if let Some(rest) = line.strip_prefix("    (") {
            if let Some(num) = rest.strip_suffix("%)") {
                if let Ok(v) = num.trim().parse::<f64>() {
                    pct = v;
                }
            }
        }
    }
    let status = child.wait().await?;
    if !status.success() {
        anyhow::bail!("qemu-img convert failed");
    }
    Ok((format!("converted to {dest}"), logs, pct.max(100.0)))
}
