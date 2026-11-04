# LPMR — Linux Partitions Manager (Rust)

Lightweight **Rust** control panel for Linux block storage, LVM, LUKS, VM disk images (RAW, QCOW2, VMDK, VDI, VHDX), and optional **Proxmox VE** storage (`pvesm`, `qm importdisk`). The UI follows the **docked workspace layout** from [Raven](https://github.com/oscar-soirey/Raven) (header bar, left navigator, center details, operations column, bottom log).

**Default language:** English. Built-in UI strings also ship for **Spanish, Chinese, Hindi, Arabic, and French**.

![Web panel mockup](docs/images/web-panel-mockup.svg)

![Terminal panel mockup](docs/images/tui-panel-mockup.svg)

## Features (inspired by Cockpit, Webmin, Proxmox)

| Area | Capabilities |
|------|----------------|
| **Block devices** | Scan disks/partitions/loop (`lsblk`), sizes, models, mount points |
| **Partitions** | GPT/MBR label, create/delete/resize (`parted`) |
| **Filesystems** | ext4, XFS, Btrfs, FAT32, NTFS, swap — format, mount, unmount |
| **LVM** | PV/VG/LV listing; create VG/LV; extend LV |
| **VM images** | `qemu-img` info, create, resize, convert (raw ↔ qcow2 ↔ vmdk …) |
| **Proxmox** | Storage status, volume lists, path resolution when `pvesm` is present |
| **Encryption** | LUKS open/close via `cryptsetup` |
| **Safety** | API and CLI default to **dry-run** for destructive flows |

## System tools

Install recommended packages (Debian/Ubuntu):

```bash
sudo ./scripts/install-deps.sh
```

Verify:

```bash
./target/release/lpmr doctor
./target/release/lpmr tools
```

Core tools: `lsblk`, `parted`, `blkid`, `findmnt`, `mkfs.*`, `mount`, `umount`, `lvm`, `qemu-img`, `cryptsetup`, `rsync`. Optional: `mdadm`, `zfs`, `pvesm`, `qm`.

## Build

Requires Rust **1.85+** (stable). From the repository root:

```bash
cargo build --release
```

Binary: `target/release/lpmr`

## Usage

### Web application

Run from the **repository root** (static assets live under `assets/web/`):

```bash
./target/release/lpmr web --bind 127.0.0.1:8787
```

Open `http://127.0.0.1:8787` — switch **enterprise themes** and **language** from the header. REST API under `/api/*` (devices, LVM, tools, themes, i18n, Proxmox, qemu, operations).

### Terminal application

Raven-style three-column TUI (Ratatui):

```bash
./target/release/lpmr tui
```

| Key | Action |
|-----|--------|
| `r` | Refresh devices / LVM / tools |
| `Tab` | Cycle navigator tabs |
| `j` / `k` | Move selection |
| `t` | Cycle enterprise theme |
| `l` | Cycle language |
| `d` | Sample dry-run operation |
| `q` | Quit |

### Performance comparison (Rust panel vs shell-only)

The panel wraps system tools in a typed Rust core instead of ad-hoc shell scripts. Benchmark device scans:

```bash
./target/release/lpmr bench --iterations 100
```

![Performance comparison](docs/images/performance-comparison.svg)

**What you gain with Rust (even when raw `lsblk` time is similar):**

- Single static binary for web + TUI + API
- Structured JSON domain models (devices, LVM, qemu, Proxmox)
- Centralized validation (paths under `/dev/`, dry-run defaults)
- Lower memory footprint vs spawning bash/Python orchestration per action
- Shared i18n and theme tokens across web and terminal

Example on this environment (20 iterations): Rust core ~1.95 ms/op vs shell loop ~1.88 ms/op — the win is **architecture and safety**, not replacing `lsblk` itself.

## Enterprise themes

| ID | Name |
|----|------|
| `midnight-enterprise` | Midnight Enterprise (Raven Midnight palette) |
| `slate-pro` | Slate Pro |
| `azure-horizon` | Azure Horizon |
| `forest-admin` | Forest Admin |
| `ruby-executive` | Ruby Executive |

Themes are defined in `lpmr-core` and applied in both web CSS variables and TUI colors.

## Raven GUI (optional native)

[Raven](https://github.com/oscar-soirey/Raven) provides a Vulkan/XML docked desktop UI. LPMR mirrors that **workspace metaphor** in web and TUI. A future optional feature can embed `raven-gui` XMLC panels on workstations with Raven libraries installed; this repository focuses on the lightweight Rust web + terminal stack for servers and Proxmox nodes.

## Project layout

```
crates/lpmr-core   # Storage operations, i18n, themes
crates/lpmr-web    # Axum REST + static web UI
crates/lpmr-tui    # Ratatui terminal panel
crates/lpmr-cli    # `lpmr` binary (web | tui | doctor | tools | bench)
assets/web/        # Web shell (Raven-inspired layout)
scripts/           # Dependency installer
docs/images/       # README diagrams
```

## Security

- Run with root only when performing real mutations; use dry-run first.
- The web server binds to localhost by default — put a reverse proxy with auth in front for remote access.

## License

MIT — see [LICENSE](LICENSE).
