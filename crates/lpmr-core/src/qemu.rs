use crate::tools::{run_command_ok, validate_block_path};
use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum QemuFormat {
    Raw,
    Qcow2,
    Vmdk,
    Vdi,
    Vhdx,
    Qed,
}

impl QemuFormat {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Raw => "raw",
            Self::Qcow2 => "qcow2",
            Self::Vmdk => "vmdk",
            Self::Vdi => "vdi",
            Self::Vhdx => "vhdx",
            Self::Qed => "qed",
        }
    }

    pub fn all() -> &'static [QemuFormat] {
        &[
            Self::Raw,
            Self::Qcow2,
            Self::Vmdk,
            Self::Vdi,
            Self::Vhdx,
            Self::Qed,
        ]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QemuImageInfo {
    pub path: String,
    pub format: String,
    pub virtual_size: String,
    pub disk_size: String,
    pub cluster_size: Option<String>,
    pub compressed: bool,
    pub raw_json: serde_json::Value,
}

pub fn image_info(path: &str) -> Result<QemuImageInfo> {
    let json = run_command_ok("qemu-img", &["info", "--output=json", path])?;
    let value: serde_json::Value = serde_json::from_str(&json)?;
    let format = value
        .get("format")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();
    let virtual_size = value
        .get("virtual-size")
        .map(|v| v.to_string())
        .unwrap_or_else(|| "?".into());
    let disk_size = value
        .get("actual-size")
        .map(|v| v.to_string())
        .unwrap_or_else(|| "?".into());
    let cluster_size = value.get("cluster-size").map(|v| v.to_string());
    let compressed = value
        .get("compressed")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    Ok(QemuImageInfo {
        path: path.to_string(),
        format,
        virtual_size,
        disk_size,
        cluster_size,
        compressed,
        raw_json: value,
    })
}

pub fn convert_image(
    source: &str,
    dest: &str,
    from: Option<QemuFormat>,
    to: QemuFormat,
    dry_run: bool,
) -> Result<String> {
    let mut args = vec!["convert", "-p", "-O", to.as_str()];
    if let Some(f) = from {
        args.push("-f");
        args.push(f.as_str());
    }
    args.push(source);
    args.push(dest);
    if dry_run {
        return Ok(format!("dry-run: qemu-img {}", args.join(" ")));
    }
    run_command_ok("qemu-img", &args)
}

pub fn resize_image(path: &str, size: &str, dry_run: bool) -> Result<String> {
    if dry_run {
        return Ok(format!("dry-run: qemu-img resize {path} {size}"));
    }
    run_command_ok("qemu-img", &["resize", path, size])
}

pub fn create_image(path: &str, size: &str, format: QemuFormat, dry_run: bool) -> Result<String> {
    if dry_run {
        return Ok(format!(
            "dry-run: qemu-img create -f {} {path} {size}",
            format.as_str()
        ));
    }
    run_command_ok(
        "qemu-img",
        &["create", "-f", format.as_str(), path, size],
    )
}

pub fn attach_to_loop(path: &str, dry_run: bool) -> Result<String> {
    if !path.starts_with('/') {
        bail!("Image path must be absolute");
    }
    if dry_run {
        return Ok(format!("dry-run: losetup --find --show {path}"));
    }
    let dev = run_command_ok("losetup", &["--find", "--show", path])?;
    validate_block_path(&dev)?;
    Ok(dev)
}
