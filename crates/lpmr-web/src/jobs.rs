use crate::state::JobStore;
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

/// Parses qemu-img `-p` progress: `(12.34/100%)` or legacy `12%`.
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

async fn push_job_progress(jobs: &JobStore, job_id: &str, pct: f64, line: &str) {
    let mut map = jobs.write().await;
    if let Some(job) = map.get_mut(job_id) {
        job.status = JobStatus::Running;
        job.progress_pct = pct;
        job.message = format!("converting ({pct:.1}%)");
        if job.logs.last().map(|s| s.as_str()) != Some(line) {
            job.logs.push(line.to_string());
        }
    }
}

fn drain_progress_fragments(buffer: &mut String, logs: &mut Vec<String>) -> Vec<(f64, String)> {
    let mut updates = Vec::new();
    while let Some(pos) = buffer.find(['\r', '\n']) {
        let line = buffer[..pos].to_string();
        buffer.drain(..=pos);
        if line.trim().is_empty() {
            continue;
        }
        logs.push(line.clone());
        if let Some(p) = parse_qemu_progress(&line) {
            updates.push((p, line));
        }
    }
    updates
}

pub async fn run_convert_async(
    source: String,
    dest: String,
    format: String,
    jobs: JobStore,
    job_id: String,
) -> Result<(String, Vec<String>, f64)> {
    let mut child = TokioCommand::new("qemu-img")
        .args(["convert", "-p", "-O", &format, &source, &dest])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .context("spawn qemu-img")?;

    let mut stdout = child.stdout.take().context("stdout")?;
    let stderr = child.stderr.take().context("stderr")?;

    // Drain stderr concurrently so qemu-img cannot block when the pipe fills.
    let stderr_task = tokio::spawn(async move {
        let mut err = stderr;
        let mut err_buf = Vec::new();
        err.read_to_end(&mut err_buf).await?;
        Ok::<Vec<u8>, anyhow::Error>(err_buf)
    });

    let mut logs = Vec::new();
    let mut pct = 0.0f64;
    let mut scratch = vec![0u8; 1024];
    let mut carry = String::new();

    loop {
        let n = stdout.read(&mut scratch).await?;
        if n == 0 {
            break;
        }
        carry.push_str(&String::from_utf8_lossy(&scratch[..n]));
        for (p, line) in drain_progress_fragments(&mut carry, &mut logs) {
            pct = p;
            push_job_progress(&jobs, &job_id, p, &line).await;
        }
        if let Some(p) = parse_qemu_progress(&carry.trim()) {
            pct = p;
            push_job_progress(&jobs, &job_id, p, carry.trim()).await;
        }
    }

    if !carry.trim().is_empty() {
        if let Some(p) = parse_qemu_progress(&carry) {
            pct = p;
            push_job_progress(&jobs, &job_id, p, carry.trim()).await;
        }
        logs.push(carry.trim().to_string());
    }

    let err_buf = stderr_task.await.context("stderr task join")??;
    if !err_buf.is_empty() {
        let text = String::from_utf8_lossy(&err_buf);
        for line in text.split(['\r', '\n']) {
            if line.trim().is_empty() {
                continue;
            }
            logs.push(line.to_string());
            if let Some(p) = parse_qemu_progress(line) {
                pct = p;
                push_job_progress(&jobs, &job_id, p, line).await;
            }
        }
    }

    let status = child.wait().await?;
    if !status.success() {
        anyhow::bail!("qemu-img convert failed");
    }
    push_job_progress(&jobs, &job_id, 100.0, "complete").await;
    Ok((format!("converted to {dest}"), logs, pct.max(100.0)))
}

#[cfg(test)]
mod tests {
    use super::parse_qemu_progress;

    #[test]
    fn parse_progress_fraction() {
        assert!((parse_qemu_progress("    (64.00/100%)").unwrap() - 64.0).abs() < f64::EPSILON);
        assert!((parse_qemu_progress("(12.34/100%)").unwrap() - 12.34).abs() < 0.001);
    }

    #[test]
    fn parse_progress_plain_percent() {
        assert_eq!(parse_qemu_progress("42%"), Some(42.0));
    }
}
