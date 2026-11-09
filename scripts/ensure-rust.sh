#!/usr/bin/env bash
# Ensure cargo/rustc >= 1.88 (ratatui/instability deps). Idempotent for Debian/PVE apt rust.
set -euo pipefail

if [[ -f "${HOME}/.cargo/env" ]]; then
  # shellcheck disable=SC1091
  source "${HOME}/.cargo/env"
fi
export PATH="${HOME}/.cargo/bin:/usr/local/cargo/bin:/usr/bin:${PATH}"

msrv_ok() {
  command -v rustc >/dev/null || return 1
  local major minor
  read -r major minor _ < <(rustc --version | sed -n 's/rustc \([0-9]*\)\.\([0-9]*\).*/\1 \2/p')
  [[ "${major:-0}" -gt 1 ]] && return 0
  [[ "${major:-0}" -eq 1 && "${minor:-0}" -ge 88 ]]
}

install_rustup() {
  echo "LPMR: installing Rust via rustup (needs rustc >= 1.88; Debian apt cargo 1.85 is too old)…"
  export RUSTUP_INIT_SKIP_PATH_CHECK=yes
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable --profile minimal
  # shellcheck disable=SC1091
  source "${HOME}/.cargo/env"
  rustup component add rustfmt >/dev/null 2>&1 || true
}

if ! command -v cargo >/dev/null; then
  if command -v apt-get >/dev/null; then
    echo "LPMR: cargo not found — install build tools or run: curl … | sh (rustup)"
    echo "  Optional: apt install -y curl build-essential pkg-config"
  fi
  install_rustup
elif ! msrv_ok; then
  if command -v rustup >/dev/null; then
    rustup update stable
    rustup default stable
  else
    install_rustup
  fi
fi

if ! msrv_ok; then
  echo "error: rustc $(rustc --version 2>/dev/null || echo missing) — LPMR requires rustc >= 1.88" >&2
  echo "  Run: source \"\$HOME/.cargo/env\" && rustup update stable" >&2
  exit 1
fi
