use crate::tools::run_command_ok;
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxmoxStorage {
    pub name: String,
    pub storage_type: String,
    pub total: String,
    pub used: String,
    pub available: String,
    pub active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxmoxVolume {
    pub volid: String,
    pub format: String,
    pub size: String,
}

pub fn is_proxmox_node() -> bool {
    which::which("pvesm").is_ok()
}

pub fn list_storages() -> Result<Vec<ProxmoxStorage>> {
    let raw = run_command_ok("pvesm", &["status"])?;
    let mut out = Vec::new();
    for line in raw.lines().skip(1) {
        let cols: Vec<_> = line.split_whitespace().collect();
        if cols.len() < 7 {
            continue;
        }
        out.push(ProxmoxStorage {
            name: cols[0].to_string(),
            storage_type: cols[1].to_string(),
            total: cols[3].to_string(),
            used: cols[4].to_string(),
            available: cols[5].to_string(),
            active: cols[6].eq_ignore_ascii_case("active"),
        });
    }
    Ok(out)
}

pub fn list_storage_volumes(storage: &str) -> Result<Vec<ProxmoxVolume>> {
    let raw = run_command_ok("pvesm", &["list", storage])?;
    let mut out = Vec::new();
    for line in raw.lines().skip(1) {
        let cols: Vec<_> = line.split_whitespace().collect();
        if cols.len() < 3 {
            continue;
        }
        out.push(ProxmoxVolume {
            volid: cols[0].to_string(),
            format: cols[1].to_string(),
            size: cols[2].to_string(),
        });
    }
    Ok(out)
}

pub fn resolve_volume_path(volid: &str) -> Result<String> {
    run_command_ok("pvesm", &["path", volid])
}

pub fn import_disk(
    vmid: u32,
    source: &str,
    storage: &str,
    format: &str,
    dry_run: bool,
) -> Result<String> {
    if dry_run {
        return Ok(format!(
            "dry-run: qm importdisk {vmid} {source} {storage} --format {format}"
        ));
    }
    run_command_ok(
        "qm",
        &[
            "importdisk",
            &vmid.to_string(),
            source,
            storage,
            "--format",
            format,
        ],
    )
}
