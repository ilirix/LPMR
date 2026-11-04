use crate::filesystems::FilesystemKind;
use crate::qemu::QemuFormat;
use crate::tools::{ensure_tool, run_command_ok, validate_block_path};
use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OperationKind {
    CreatePartition,
    DeletePartition,
    ResizePartition,
    CreatePartitionTable,
    FormatFilesystem,
    Mount,
    Unmount,
    MoveData,
    ConvertImage,
    ResizeImage,
    CreateImage,
    CreateVolumeGroup,
    CreateLogicalVolume,
    ExtendLogicalVolume,
    LuksOpen,
    LuksClose,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationRequest {
    pub kind: OperationKind,
    pub dry_run: bool,
    pub device: Option<String>,
    pub path: Option<String>,
    pub target: Option<String>,
    pub size: Option<String>,
    pub fstype: Option<FilesystemKind>,
    pub partition_table: Option<String>,
    pub start: Option<String>,
    pub end: Option<String>,
    pub mount_point: Option<String>,
    pub options: Option<String>,
    pub source: Option<String>,
    pub dest: Option<String>,
    pub image_format: Option<QemuFormat>,
    pub vg: Option<String>,
    pub lv: Option<String>,
    pub luks_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationResult {
    pub success: bool,
    pub message: String,
    pub command: String,
}

pub fn execute(req: OperationRequest) -> Result<OperationResult> {
    let result = match req.kind {
        OperationKind::CreatePartitionTable => create_partition_table(&req)?,
        OperationKind::CreatePartition => create_partition(&req)?,
        OperationKind::DeletePartition => delete_partition(&req)?,
        OperationKind::ResizePartition => resize_partition(&req)?,
        OperationKind::FormatFilesystem => format_fs(&req)?,
        OperationKind::Mount => mount_fs(&req)?,
        OperationKind::Unmount => unmount_fs(&req)?,
        OperationKind::MoveData => move_data(&req)?,
        OperationKind::ConvertImage => convert_img(&req)?,
        OperationKind::ResizeImage => resize_img(&req)?,
        OperationKind::CreateImage => create_img(&req)?,
        OperationKind::CreateVolumeGroup => create_vg(&req)?,
        OperationKind::CreateLogicalVolume => create_lv(&req)?,
        OperationKind::ExtendLogicalVolume => extend_lv(&req)?,
        OperationKind::LuksOpen => luks_open(&req)?,
        OperationKind::LuksClose => luks_close(&req)?,
    };
    Ok(result)
}

fn create_partition_table(req: &OperationRequest) -> Result<OperationResult> {
    let device = req.device.as_deref().ok_or_else(|| anyhow::anyhow!("device required"))?;
    validate_block_path(device)?;
    let table = req.partition_table.as_deref().unwrap_or("gpt");
    let cmd = format!("parted -s {device} mklabel {table}");
    if req.dry_run {
        return Ok(dry_ok(&cmd));
    }
    ensure_tool("parted")?;
    run_command_ok("parted", &["-s", device, "mklabel", table])?;
    Ok(ok(&cmd))
}

fn create_partition(req: &OperationRequest) -> Result<OperationResult> {
    let device = req.device.as_deref().ok_or_else(|| anyhow::anyhow!("device required"))?;
    validate_block_path(device)?;
    let start = req.start.as_deref().unwrap_or("1MiB");
    let end = req.end.as_deref().unwrap_or("100%");
    let cmd = format!("parted -s {device} mkpart primary {start} {end}");
    if req.dry_run {
        return Ok(dry_ok(&cmd));
    }
    run_command_ok("parted", &["-s", device, "mkpart", "primary", start, end])?;
    Ok(ok(&cmd))
}

fn delete_partition(req: &OperationRequest) -> Result<OperationResult> {
    let device = req.device.as_deref().ok_or_else(|| anyhow::anyhow!("device required"))?;
    validate_block_path(device)?;
    let part_num = req.target.as_deref().ok_or_else(|| anyhow::anyhow!("partition number required"))?;
    let cmd = format!("parted -s {device} rm {part_num}");
    if req.dry_run {
        return Ok(dry_ok(&cmd));
    }
    run_command_ok("parted", &["-s", device, "rm", part_num])?;
    Ok(ok(&cmd))
}

fn resize_partition(req: &OperationRequest) -> Result<OperationResult> {
    let device = req.device.as_deref().ok_or_else(|| anyhow::anyhow!("device required"))?;
    validate_block_path(device)?;
    let part_num = req.target.as_deref().ok_or_else(|| anyhow::anyhow!("partition number required"))?;
    let end = req.end.as_deref().ok_or_else(|| anyhow::anyhow!("end required"))?;
    let cmd = format!("parted -s {device} resizepart {part_num} {end}");
    if req.dry_run {
        return Ok(dry_ok(&cmd));
    }
    run_command_ok("parted", &["-s", device, "resizepart", part_num, end])?;
    Ok(ok(&cmd))
}

fn format_fs(req: &OperationRequest) -> Result<OperationResult> {
    let device = req.device.as_deref().ok_or_else(|| anyhow::anyhow!("device required"))?;
    validate_block_path(device)?;
    let fstype = req.fstype.unwrap_or(FilesystemKind::Ext4);
    let Some(mkfs) = fstype.mkfs_command() else {
        bail!("Unsupported filesystem for formatting");
    };
    ensure_tool(mkfs)?;
    let cmd = format!("{mkfs} {device}");
    if req.dry_run {
        return Ok(dry_ok(&cmd));
    }
    run_command_ok(mkfs, &[device])?;
    Ok(ok(&cmd))
}

fn mount_fs(req: &OperationRequest) -> Result<OperationResult> {
    let device = req.device.as_deref().ok_or_else(|| anyhow::anyhow!("device required"))?;
    validate_block_path(device)?;
    let mount_point = req
        .mount_point
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("mount_point required"))?;
    let mut args = vec!["mount"];
    if let Some(opts) = &req.options {
        args.push("-o");
        args.push(opts.as_str());
    }
    args.push(device);
    args.push(mount_point);
    let cmd = args.join(" ");
    if req.dry_run {
        return Ok(dry_ok(&cmd));
    }
    run_command_ok("mount", &args[1..])?;
    Ok(ok(&cmd))
}

fn unmount_fs(req: &OperationRequest) -> Result<OperationResult> {
    let target = req
        .mount_point
        .as_deref()
        .or(req.device.as_deref())
        .ok_or_else(|| anyhow::anyhow!("mount_point or device required"))?;
    let cmd = format!("umount {target}");
    if req.dry_run {
        return Ok(dry_ok(&cmd));
    }
    run_command_ok("umount", &[target])?;
    Ok(ok(&cmd))
}

fn move_data(req: &OperationRequest) -> Result<OperationResult> {
    let source = req.source.as_deref().ok_or_else(|| anyhow::anyhow!("source required"))?;
    let dest = req.dest.as_deref().ok_or_else(|| anyhow::anyhow!("dest required"))?;
    let cmd = format!("rsync -aHAX --info=progress2 {source}/ {dest}/");
    if req.dry_run {
        return Ok(dry_ok(&cmd));
    }
    ensure_tool("rsync")?;
    run_command_ok("rsync", &["-aHAX", source, dest])?;
    Ok(ok(&cmd))
}

fn convert_img(req: &OperationRequest) -> Result<OperationResult> {
    let source = req.source.as_deref().ok_or_else(|| anyhow::anyhow!("source required"))?;
    let dest = req.dest.as_deref().ok_or_else(|| anyhow::anyhow!("dest required"))?;
    let format = req.image_format.unwrap_or(QemuFormat::Qcow2);
    let msg = crate::qemu::convert_image(source, dest, None, format, req.dry_run)?;
    Ok(OperationResult {
        success: true,
        message: msg.clone(),
        command: msg,
    })
}

fn resize_img(req: &OperationRequest) -> Result<OperationResult> {
    let path = req.path.as_deref().ok_or_else(|| anyhow::anyhow!("path required"))?;
    let size = req.size.as_deref().ok_or_else(|| anyhow::anyhow!("size required"))?;
    let msg = crate::qemu::resize_image(path, size, req.dry_run)?;
    Ok(OperationResult {
        success: true,
        message: msg.clone(),
        command: msg,
    })
}

fn create_img(req: &OperationRequest) -> Result<OperationResult> {
    let path = req.path.as_deref().ok_or_else(|| anyhow::anyhow!("path required"))?;
    let size = req.size.as_deref().ok_or_else(|| anyhow::anyhow!("size required"))?;
    let format = req.image_format.unwrap_or(QemuFormat::Qcow2);
    let msg = crate::qemu::create_image(path, size, format, req.dry_run)?;
    Ok(OperationResult {
        success: true,
        message: msg.clone(),
        command: msg,
    })
}

fn create_vg(req: &OperationRequest) -> Result<OperationResult> {
    let vg = req.vg.as_deref().ok_or_else(|| anyhow::anyhow!("vg required"))?;
    let device = req.device.as_deref().ok_or_else(|| anyhow::anyhow!("device required"))?;
    validate_block_path(device)?;
    let msg = crate::lvm::create_volume_group(vg, &[device], req.dry_run)?;
    Ok(OperationResult {
        success: true,
        message: msg.clone(),
        command: msg,
    })
}

fn create_lv(req: &OperationRequest) -> Result<OperationResult> {
    let vg = req.vg.as_deref().ok_or_else(|| anyhow::anyhow!("vg required"))?;
    let lv = req.lv.as_deref().ok_or_else(|| anyhow::anyhow!("lv required"))?;
    let size = req.size.as_deref().ok_or_else(|| anyhow::anyhow!("size required"))?;
    let msg = crate::lvm::create_logical_volume(vg, lv, size, req.dry_run)?;
    Ok(OperationResult {
        success: true,
        message: msg.clone(),
        command: msg,
    })
}

fn extend_lv(req: &OperationRequest) -> Result<OperationResult> {
    let vg = req.vg.as_deref().ok_or_else(|| anyhow::anyhow!("vg required"))?;
    let lv = req.lv.as_deref().ok_or_else(|| anyhow::anyhow!("lv required"))?;
    let size = req.size.as_deref().ok_or_else(|| anyhow::anyhow!("size required"))?;
    let msg = crate::lvm::extend_logical_volume(vg, lv, size, req.dry_run)?;
    Ok(OperationResult {
        success: true,
        message: msg.clone(),
        command: msg,
    })
}

fn luks_open(req: &OperationRequest) -> Result<OperationResult> {
    let device = req.device.as_deref().ok_or_else(|| anyhow::anyhow!("device required"))?;
    validate_block_path(device)?;
    let name = req
        .luks_name
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("luks_name required"))?;
    let cmd = format!("cryptsetup open {device} {name}");
    if req.dry_run {
        return Ok(dry_ok(&cmd));
    }
    run_command_ok("cryptsetup", &["open", device, name])?;
    Ok(ok(&cmd))
}

fn luks_close(req: &OperationRequest) -> Result<OperationResult> {
    let name = req
        .luks_name
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("luks_name required"))?;
    let cmd = format!("cryptsetup close {name}");
    if req.dry_run {
        return Ok(dry_ok(&cmd));
    }
    run_command_ok("cryptsetup", &["close", name])?;
    Ok(ok(&cmd))
}

fn ok(command: &str) -> OperationResult {
    OperationResult {
        success: true,
        message: "completed".into(),
        command: command.into(),
    }
}

fn dry_ok(command: &str) -> OperationResult {
    OperationResult {
        success: true,
        message: "dry-run".into(),
        command: command.into(),
    }
}
