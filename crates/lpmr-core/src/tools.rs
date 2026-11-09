use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use which::which;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemTool {
    pub name: &'static str,
    pub path: PathBuf,
    pub purpose: &'static str,
}

pub fn required_tools() -> Vec<(&'static str, &'static str)> {
    vec![
        ("lsblk", "List block devices (JSON)"),
        ("parted", "Partition tables and resize"),
        ("sfdisk", "Scriptable partition editing"),
        ("blkid", "Filesystem and LUKS labels"),
        ("findmnt", "Mount topology"),
        ("mkfs.ext4", "Create ext4 (optional alias mkfs.ext4)"),
        ("mkfs.xfs", "Create XFS"),
        ("mkfs.btrfs", "Create Btrfs"),
        ("mkfs.vfat", "Create FAT32"),
        ("mkfs.ntfs", "Create NTFS (ntfs-3g)"),
        ("mount", "Mount filesystems"),
        ("umount", "Unmount filesystems"),
        ("lvm", "LVM2 management"),
        ("pvs", "Physical volumes"),
        ("vgs", "Volume groups"),
        ("lvs", "Logical volumes"),
        ("qemu-img", "QCOW2/RAW/VMDK/VDI convert & resize"),
        ("cryptsetup", "LUKS encryption"),
        ("mdadm", "Linux RAID"),
        ("zfs", "ZFS pools (optional)"),
        ("pvesm", "Proxmox storage (optional on PVE nodes)"),
        ("qm", "Proxmox VM disk import (optional)"),
    ]
}

pub fn discover_installed_tools() -> Vec<SystemTool> {
    required_tools()
        .into_iter()
        .filter_map(|(name, purpose)| {
            which(name).ok().map(|path| SystemTool {
                name,
                path,
                purpose,
            })
        })
        .collect()
}

pub fn ensure_tool(name: &str) -> Result<PathBuf> {
    which(name).with_context(|| format!("Required tool `{name}` is not installed"))
}

pub fn run_command(program: &str, args: &[&str]) -> Result<Output> {
    let output = Command::new(program)
        .args(args)
        .output()
        .with_context(|| format!("Failed to execute `{program}`"))?;
    Ok(output)
}

pub fn run_command_ok(program: &str, args: &[&str]) -> Result<String> {
    let output = run_command(program, args)?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!(
            "`{} {}` failed ({}): {}",
            program,
            args.join(" "),
            output.status,
            stderr.trim()
        );
    }
}

pub fn dry_run_note(dry_run: bool) -> Option<String> {
    if dry_run {
        Some("dry-run: command not executed".into())
    } else {
        None
    }
}

pub fn validate_block_path(path: &str) -> Result<()> {
    if path.is_empty() || path.contains("..") {
        bail!("Invalid device path");
    }
    if !Path::new(path).starts_with("/dev/") {
        bail!("Path must be under /dev/");
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QemuPathRole {
    Source,
    Dest,
}

const QEMU_DEST_DENY_PREFIXES: &[&str] = &[
    "/dev/", "/proc/", "/sys/", "/etc/", "/usr/", "/bin/", "/sbin/", "/boot/", "/run/",
];

/// Guards qemu-img source/dest paths for API-triggered converts (blocks raw disk overwrites as dest).
pub fn validate_qemu_image_path(path: &str, role: QemuPathRole) -> Result<()> {
    if path.is_empty() || path.contains("..") {
        bail!("Invalid image path");
    }
    let p = Path::new(path);
    if !p.is_absolute() {
        bail!("Image path must be absolute");
    }
    if role == QemuPathRole::Dest {
        for prefix in QEMU_DEST_DENY_PREFIXES {
            if path.starts_with(prefix) {
                bail!("Destination path not allowed: {path}");
            }
        }
    }
    Ok(())
}
