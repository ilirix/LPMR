use crate::devices::{scan_block_devices, BlockDevice};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartitionSegment {
    pub name: String,
    pub path: String,
    pub size_bytes: u64,
    pub pct: f64,
    pub fstype: Option<String>,
    pub label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskPartitionMap {
    pub disk_path: String,
    pub disk_name: String,
    pub model: Option<String>,
    pub size_bytes: u64,
    pub segments: Vec<PartitionSegment>,
}

pub fn partition_map_for_disk(disk_path: &str) -> Option<DiskPartitionMap> {
    let tree = scan_block_devices().ok()?;
    let disk = resolve_disk_node(&tree.devices, disk_path)?;
    let total = disk.size_bytes.max(1);
    let segments: Vec<_> = disk
        .children
        .iter()
        .map(|child| PartitionSegment {
            name: child.name.clone(),
            path: child.path.clone(),
            size_bytes: child.size_bytes,
            pct: (child.size_bytes as f64 / total as f64) * 100.0,
            fstype: child.fstype.clone(),
            label: child
                .partlabel
                .clone()
                .or_else(|| child.fstype.clone())
                .unwrap_or_else(|| child.name.clone()),
        })
        .collect();
    Some(DiskPartitionMap {
        disk_path: disk.path.clone(),
        disk_name: disk.name.clone(),
        model: disk.model.clone(),
        size_bytes: disk.size_bytes,
        segments,
    })
}

fn resolve_disk_node<'a>(devs: &'a [BlockDevice], path: &str) -> Option<&'a BlockDevice> {
    if let Some(node) = find_by_path(devs, path) {
        if !node.children.is_empty() {
            return Some(node);
        }
    }
    for parent in devs {
        if parent.children.iter().any(|c| c.path == path) {
            return Some(parent);
        }
        if let Some(p) = resolve_disk_node(&parent.children, path) {
            if p.children.iter().any(|c| c.path == path) {
                return Some(p);
            }
        }
    }
    find_by_path(devs, path)
}

fn find_by_path<'a>(devs: &'a [BlockDevice], path: &str) -> Option<&'a BlockDevice> {
    for d in devs {
        if d.path == path {
            return Some(d);
        }
        if let Some(inner) = find_by_path(&d.children, path) {
            return Some(inner);
        }
    }
    None
}

pub fn top_level_disks() -> Result<Vec<BlockDevice>> {
    Ok(scan_block_devices()?.devices)
}

use anyhow::Result;
