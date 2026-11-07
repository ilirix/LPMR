use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostTelemetry {
    pub hostname: String,
    pub kernel: String,
    pub rss_mb: f64,
    pub online: bool,
}

pub fn host_telemetry() -> HostTelemetry {
    let hostname = fs::read_to_string("/etc/hostname")
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|_| "localhost".into());
    let kernel = fs::read_to_string("/proc/sys/kernel/osrelease")
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|_| "unknown".into());
    let rss_mb = current_process_rss_mb().unwrap_or(0.0);
    HostTelemetry {
        hostname,
        kernel,
        rss_mb,
        online: true,
    }
}

fn current_process_rss_mb() -> Option<f64> {
    let status = fs::read_to_string("/proc/self/status").ok()?;
    for line in status.lines() {
        if let Some(kb) = line.strip_prefix("VmRSS:") {
            let kb: f64 = kb.trim().replace(" kB", "").parse().ok()?;
            return Some(kb / 1024.0);
        }
    }
    None
}
