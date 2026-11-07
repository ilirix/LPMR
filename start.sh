#!/usr/bin/env bash
# LPMR — one-shot launcher (web orchestrator + API)
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$ROOT"
export PATH="${HOME}/.cargo/bin:/usr/local/cargo/bin:${PATH}"

BIND="${LPMR_BIND:-0.0.0.0:8787}"
INSTALL_DEPS="${LPMR_INSTALL_DEPS:-0}"

if [[ "$INSTALL_DEPS" == "1" ]] && [[ -x scripts/install-deps.sh ]]; then
  sudo scripts/install-deps.sh || true
fi

if [[ ! -x target/release/lpmr ]]; then
  echo "Building lpmr (release)…"
  cargo build --release
fi

echo "LPMR orchestrator → http://${BIND/0.0.0.0/localhost}"
echo "  Web UI + API  |  Terminal: lpmr tui"
exec ./target/release/lpmr web --bind "$BIND"
