use crate::tools::run_command_ok;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MountPoint {
    pub target: String,
    pub fstype: Option<String>,
    pub options: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockDevice {
    pub name: String,
    pub path: String,
    pub size_bytes: u64,
    pub model: Option<String>,
    pub serial: Option<String>,
    pub transport: Option<String>,
    pub rota: bool,
    pub removable: bool,
    pub mountpoints: Vec<Option<String>>,
    pub fstype: Option<String>,
    pub parttype: Option<String>,
    pub partlabel: Option<String>,
    pub uuid: Option<String>,
    pub pkname: Option<String>,
    pub children: Vec<BlockDevice>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockDeviceTree {
    pub devices: Vec<BlockDevice>,
    pub scanned_at: String,
}

#[derive(Deserialize)]
struct LsblkNode {
    name: String,
    path: Option<String>,
    size: Option<u64>,
    model: Option<String>,
    serial: Option<String>,
    tran: Option<String>,
    rota: Option<bool>,
    rm: Option<bool>,
    mountpoints: Option<Vec<Option<String>>>,
    fstype: Option<String>,
    parttype: Option<String>,
    partlabel: Option<String>,
    uuid: Option<String>,
    pkname: Option<String>,
    children: Option<Vec<LsblkNode>>,
}

pub fn scan_block_devices() -> Result<BlockDeviceTree> {
    let json = run_command_ok("lsblk", &["--json", "-b", "-O"])?;
    let root: serde_json::Value = serde_json::from_str(&json).context("parse lsblk json")?;
    let nodes = root
        .get("blockdevices")
        .and_then(|v| serde_json::from_value::<Vec<LsblkNode>>(v.clone()).ok())
        .unwrap_or_default();
    let devices = nodes.into_iter().map(map_node).collect();
    Ok(BlockDeviceTree {
        devices,
        scanned_at: chrono_now(),
    })
}

fn map_node(node: LsblkNode) -> BlockDevice {
    BlockDevice {
        name: node.name,
        path: node.path.unwrap_or_default(),
        size_bytes: node.size.unwrap_or(0),
        model: node.model.filter(|s| !s.is_empty()),
        serial: node.serial.filter(|s| !s.is_empty()),
        transport: node.tran.filter(|s| !s.is_empty()),
        rota: node.rota.unwrap_or(false),
        removable: node.rm.unwrap_or(false),
        mountpoints: node.mountpoints.unwrap_or_default(),
        fstype: node.fstype.filter(|s| !s.is_empty()),
        parttype: node.parttype.filter(|s| !s.is_empty()),
        partlabel: node.partlabel.filter(|s| !s.is_empty()),
        uuid: node.uuid.filter(|s| !s.is_empty()),
        pkname: node.pkname.filter(|s| !s.is_empty()),
        children: node
            .children
            .unwrap_or_default()
            .into_iter()
            .map(map_node)
            .collect(),
    }
}

fn chrono_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!("{secs}")
}

pub fn format_size(bytes: u64) -> String {
    const UNITS: [&str; 6] = ["B", "KiB", "MiB", "GiB", "TiB", "PiB"];
    if bytes == 0 {
        return "0 B".into();
    }
    let mut value = bytes as f64;
    let mut unit = 0;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{} {}", bytes, UNITS[unit])
    } else {
        format!("{value:.2} {}", UNITS[unit])
    }
}
