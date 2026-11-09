use anyhow::{bail, Context, Result};
use lpmr_core::proxmox::{is_proxmox_node, resolve_volume_path};
use lpmr_core::tools::{validate_qemu_image_path, QemuPathRole};

/// Resolve Proxmox volids (`storage:volume`) via `pvesm path`, then validate qemu-img paths.
pub fn resolve_and_validate_convert_paths(source: &str, dest: &str) -> Result<(String, String)> {
    let source_path = resolve_convert_source(source)?;
    validate_qemu_image_path(&source_path, QemuPathRole::Source)?;
    validate_qemu_image_path(dest, QemuPathRole::Dest)?;
    Ok((source_path, dest.to_string()))
}

fn resolve_convert_source(source: &str) -> Result<String> {
    let trimmed = source.trim();
    if trimmed.is_empty() {
        bail!("source required");
    }
    if trimmed.starts_with('/') {
        return Ok(trimmed.to_string());
    }
    if trimmed.contains(':') {
        if !is_proxmox_node() {
            bail!("Proxmox volid source requires pvesm on this host");
        }
        return resolve_volume_path(trimmed).context("pvesm path");
    }
    bail!("source must be an absolute path or Proxmox volid (storage:name)");
}
