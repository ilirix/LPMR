use crate::tools::run_command_ok;
use anyhow::Result;
use serde::{Deserialize, Serialize};

pub fn lvm_available() -> bool {
    which::which("pvs").is_ok()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysicalVolume {
    pub name: String,
    pub vg: Option<String>,
    pub size: String,
    pub free: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeGroup {
    pub name: String,
    pub size: String,
    pub free: String,
    pub pvs: String,
    pub lvs: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogicalVolume {
    pub name: String,
    pub vg: String,
    pub size: String,
    pub pool: Option<String>,
    pub attr: String,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LvmSnapshot {
    pub pvs: Vec<PhysicalVolume>,
    pub vgs: Vec<VolumeGroup>,
    pub lvs: Vec<LogicalVolume>,
}

pub fn scan_lvm() -> Result<LvmSnapshot> {
    if !lvm_available() {
        return Ok(LvmSnapshot {
            pvs: Vec::new(),
            vgs: Vec::new(),
            lvs: Vec::new(),
        });
    }
    let pvs = parse_pvs(run_command_ok("pvs", &["--noheadings", "-o", "pv_name,vg_name,pv_size,pv_free"])?);
    let vgs = parse_vgs(run_command_ok("vgs", &["--noheadings", "-o", "vg_name,vg_size,vg_free,pv_count,lv_count"])?);
    let lvs = parse_lvs(run_command_ok(
        "lvs",
        &["--noheadings", "-o", "lv_name,vg_name,lv_size,pool_lv,lv_attr,lv_path"],
    )?);
    Ok(LvmSnapshot { pvs, vgs, lvs })
}

fn parse_pvs(raw: String) -> Vec<PhysicalVolume> {
    raw.lines()
        .filter_map(|line| {
            let cols: Vec<_> = line.split_whitespace().collect();
            if cols.len() < 4 {
                return None;
            }
            Some(PhysicalVolume {
                name: cols[0].to_string(),
                vg: if cols[1].is_empty() {
                    None
                } else {
                    Some(cols[1].to_string())
                },
                size: cols[2].to_string(),
                free: cols[3].to_string(),
            })
        })
        .collect()
}

fn parse_vgs(raw: String) -> Vec<VolumeGroup> {
    raw.lines()
        .filter_map(|line| {
            let cols: Vec<_> = line.split_whitespace().collect();
            if cols.len() < 5 {
                return None;
            }
            Some(VolumeGroup {
                name: cols[0].to_string(),
                size: cols[1].to_string(),
                free: cols[2].to_string(),
                pvs: cols[3].to_string(),
                lvs: cols[4].to_string(),
            })
        })
        .collect()
}

fn parse_lvs(raw: String) -> Vec<LogicalVolume> {
    raw.lines()
        .filter_map(|line| {
            let cols: Vec<_> = line.split_whitespace().collect();
            if cols.len() < 6 {
                return None;
            }
            Some(LogicalVolume {
                name: cols[0].to_string(),
                vg: cols[1].to_string(),
                size: cols[2].to_string(),
                pool: if cols[3].is_empty() {
                    None
                } else {
                    Some(cols[3].to_string())
                },
                attr: cols[4].to_string(),
                path: cols[5].to_string(),
            })
        })
        .collect()
}

pub fn create_volume_group(name: &str, devices: &[&str], dry_run: bool) -> Result<String> {
    let mut args: Vec<&str> = vec!["vgcreate", name];
    args.extend(devices.iter().copied());
    exec_lvm(&args, dry_run)
}

pub fn create_logical_volume(vg: &str, lv: &str, size: &str, dry_run: bool) -> Result<String> {
    exec_lvm(&["lvcreate", "-L", size, "-n", lv, vg], dry_run)
}

pub fn extend_logical_volume(vg: &str, lv: &str, size: &str, dry_run: bool) -> Result<String> {
    let plus = format!("+{size}");
    exec_lvm(&["lvextend", "-L", &plus, "-n", lv, vg], dry_run)
}

fn exec_lvm(args: &[&str], dry_run: bool) -> Result<String> {
    if dry_run {
        return Ok(format!("dry-run: lvm {}", args.join(" ")));
    }
    run_command_ok("lvm", args)
}
