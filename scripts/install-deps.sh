#!/usr/bin/env bash
set -euo pipefail
# LPMR system dependencies for Linux volume/partition/VM image management.
PACKAGES=(
  parted util-linux lvm2 qemu-utils
  e2fsprogs xfsprogs btrfs-progs dosfstools
  cryptsetup gdisk ntfs-3g smartmontools mdadm rsync
)
OPTIONAL=(zfsutils-linux)

echo "Installing LPMR dependencies..."
export DEBIAN_FRONTEND=noninteractive
apt-get update
apt-get install -y "${PACKAGES[@]}" || true
apt-get install -y "${OPTIONAL[@]}" || true

echo "Installed tools:"
for cmd in lsblk parted qemu-img vgs pvs lvs cryptsetup; do
  if command -v "$cmd" >/dev/null; then
    echo "  OK $cmd -> $(command -v "$cmd")"
  else
    echo "  MISSING $cmd"
  fi
done

echo "Proxmox tools (optional, only on PVE nodes):"
for cmd in pvesm qm; do
  command -v "$cmd" >/dev/null && echo "  OK $cmd" || echo "  (not present) $cmd"
done
