# LPMR — Proxmox VE Storage Orchestrator (Rust)

Lightweight **Rust** control plane for Linux block storage, partitions, LVM, LUKS, ZFS, Proxmox VE (`pvesm`, `qm`), and VM disk images (**RAW, QCOW2, VMDK, VHDX**). The web UI follows the **Raven // Baremetal** orchestrator layout (orange Proxmox enterprise theme, toolchain audit rail, integrated log, conversion modals).

**Languages:** English (default) plus Spanish, Chinese, Hindi, Arabic, French.

## Quick start (one script)

```bash
chmod +x start.sh
./start.sh
```

Open **http://localhost:8787** (bind override: `LPMR_BIND=127.0.0.1:8787 ./start.sh`).

Optional dependency install: `LPMR_INSTALL_DEPS=1 ./start.sh`

Equivalent CLI:

```bash
cargo build --release
./target/release/lpmr start          # 0.0.0.0:8787
./target/release/lpmr tui            # terminal panel
```

## GUI screenshots

### Main orchestrator (disks, partition visualizer, toolchain audit, log)

![Main orchestrator](docs/images/gui-orchestrator-main.png)

### VHDX conversion & dynamic resize modal

![VHDX conversion](docs/images/gui-vhdx-conversion.png)

### RAW conversion & export pipeline modal

![RAW / NVMe-oF pipeline](docs/images/gui-raw-nvmeof-pipeline.png)

## Feature reference

### Top bar & navigation

| Element | Function |
|---------|----------|
| **Search** | Filters block devices and VM volume table client-side |
| **Open Terminal** | Hint to run `lpmr tui` for full-screen terminal UI on the host |
| **+ Create Volume** | Modal to create a new `qcow2`/`raw` image (dry-run by default) |
| **Tabs: Disks / VM Images / ZFS / Health** | Switches the center workspace |

### Metrics strip

Live data from `GET /api/telemetry`:

- Hostname & kernel (`/etc/hostname`, `/proc`)
- **Rust RSS** — resident memory of the panel process (typical single-digit MiB)
- Device & tool counts

### Disks workspace

| Feature | Backend |
|---------|---------|
| **Partition visualizer** | `GET /api/partitions?disk=/dev/...` (child partitions vs disk size) |
| **Device list** | `GET /api/devices` (`lsblk` JSON) |
| **Mount / Format / Unmount** | `POST /api/operations` with `dry_run` default |
| **Convert image** | Opens conversion modal → `POST /api/jobs/convert` (`qemu-img convert -p`) |

### VM Images workspace

| Feature | Backend |
|---------|---------|
| Volume table | `GET /api/vm-volumes` (aggregates `pvesm list` on Proxmox nodes) |
| Per-row **Convert** | Pre-fills VHDX/QCOW pipeline modal |

### ZFS Pools workspace

| Feature | Backend |
|---------|---------|
| Pool cards (health, size, free) | `GET /api/zfs/pools` (`zpool list` when `zpool` is installed) |

### Health Audit workspace

| Feature | Backend |
|---------|---------|
| Toolchain grid (OK / missing) | `GET /api/health-audit` |
| Proxmox / ZFS detection flags | Same endpoint |

### Right rail

| Panel | Function |
|-------|----------|
| **System toolchain** | Mirrors Cockpit/Webmin-style dependency audit (`lpmr doctor` data) |
| **Integrated log** | UI + API events; mirrors conversion progress lines |

### Modals (Raven-style pipelines)

1. **VHDX conversion & resize** — source/dest/format, dry-run, async job with progress polling (`GET /api/jobs/:id`).
2. **RAW / QCOW2 pipeline** — same job engine (real `qemu-img` when dry-run off; requires root/capabilities).
3. **Create volume** — `CreateImage` operation via `/api/operations`.

### Terminal UI (`lpmr tui`)

Keyboard-driven Ratatui panel for the same data sets when SSH’d into a node without a browser.

### Enterprise themes (web)

**Orange** (default), **Obsidian**, **Arctic** — CSS variables on `<html data-ui-theme>`.

## Performance notes

| Technique | Why |
|-----------|-----|
| **Rust + Axum** | Single binary, typed `lsblk` parsing, no shell orchestration per request |
| **tower-http gzip** | Compresses JSON/CSS/JS responses for remote Proxmox UI access |
| **Tokio job tasks** | `qemu-img convert` runs off the request thread; UI polls progress |
| **Thin front-end** | Vanilla JS (no React build); fast first paint |

Benchmark device scan: `./target/release/lpmr bench --iterations 100`

![Performance comparison](docs/images/performance-comparison.svg)

## System tools

```bash
sudo ./scripts/install-deps.sh
./target/release/lpmr doctor
```

Includes: `lsblk`, `parted`, `qemu-img`, `lvm`, `cryptsetup`, `ntfs-3g`, `zfsutils-linux` (optional), `pvesm`/`qm` (on PVE).

## REST API summary

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/health` | Liveness |
| GET | `/api/telemetry` | Host + engine metadata |
| GET | `/api/devices` | Block device tree |
| GET | `/api/partitions?disk=` | Partition map for visualizer |
| GET | `/api/vm-volumes` | Proxmox VM disks |
| GET | `/api/zfs/pools` | ZFS pool list |
| GET | `/api/health-audit` | Toolchain audit |
| POST | `/api/jobs/convert` | Start conversion job |
| GET | `/api/jobs/:id` | Job status & logs |
| POST | `/api/operations` | Partition/mount/format/LUKS/LVM/image ops |

## Project layout

```
start.sh                 # One-shot launcher
crates/lpmr-core         # Storage, ZFS, telemetry, jobs helpers
crates/lpmr-web          # Axum API + static orchestrator UI
crates/lpmr-tui          # Terminal UI
crates/lpmr-cli          # `lpmr` binary
assets/web/              # Orchestrator CSS/JS
docs/images/             # Screenshots & diagrams
```

## Security

- Bind defaults to `0.0.0.0:8787` in `./start.sh` — use firewall or `127.0.0.1` on untrusted networks.
- Destructive actions default to **dry-run**. Run the panel as root only when executing real mutations.

## License

MIT — see [LICENSE](LICENSE).
