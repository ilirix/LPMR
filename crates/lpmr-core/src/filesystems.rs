use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FilesystemKind {
    Ext4,
    Xfs,
    Btrfs,
    Fat32,
    Ntfs,
    Swap,
    Zfs,
    Unknown,
}

impl FilesystemKind {
    pub fn mkfs_command(self) -> Option<&'static str> {
        match self {
            Self::Ext4 => Some("mkfs.ext4"),
            Self::Xfs => Some("mkfs.xfs"),
            Self::Btrfs => Some("mkfs.btrfs"),
            Self::Fat32 => Some("mkfs.vfat"),
            Self::Ntfs => Some("mkfs.ntfs"),
            Self::Swap => Some("mkswap"),
            Self::Zfs | Self::Unknown => None,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Ext4 => "ext4",
            Self::Xfs => "xfs",
            Self::Btrfs => "btrfs",
            Self::Fat32 => "vfat",
            Self::Ntfs => "ntfs",
            Self::Swap => "swap",
            Self::Zfs => "zfs",
            Self::Unknown => "unknown",
        }
    }

    pub fn all_supported() -> &'static [FilesystemKind] {
        &[
            Self::Ext4,
            Self::Xfs,
            Self::Btrfs,
            Self::Fat32,
            Self::Ntfs,
            Self::Swap,
        ]
    }
}

pub fn detect_from_fstype(fstype: &str) -> FilesystemKind {
    match fstype.to_lowercase().as_str() {
        "ext4" | "ext3" | "ext2" => FilesystemKind::Ext4,
        "xfs" => FilesystemKind::Xfs,
        "btrfs" => FilesystemKind::Btrfs,
        "vfat" | "fat32" | "msdos" => FilesystemKind::Fat32,
        "ntfs" | "ntfs3" => FilesystemKind::Ntfs,
        "swap" => FilesystemKind::Swap,
        "zfs" => FilesystemKind::Zfs,
        _ => FilesystemKind::Unknown,
    }
}
