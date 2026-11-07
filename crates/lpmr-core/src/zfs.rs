use crate::tools::run_command_ok;
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZfsPool {
    pub name: String,
    pub size: String,
    pub alloc: String,
    pub free: String,
    pub health: String,
}

pub fn zfs_available() -> bool {
    which::which("zpool").is_ok()
}

pub fn list_pools() -> Result<Vec<ZfsPool>> {
    let raw = run_command_ok("zpool", &["list", "-H", "-o", "name,size,alloc,free,health"])?;
    let mut out = Vec::new();
    for line in raw.lines() {
        let cols: Vec<_> = line.split('\t').collect();
        if cols.len() < 5 {
            continue;
        }
        out.push(ZfsPool {
            name: cols[0].to_string(),
            size: cols[1].to_string(),
            alloc: cols[2].to_string(),
            free: cols[3].to_string(),
            health: cols[4].to_string(),
        });
    }
    Ok(out)
}
