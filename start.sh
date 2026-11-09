#!/usr/bin/env bash
# LPMR — one-shot launcher (web orchestrator + API)
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$ROOT"

# shellcheck source=/dev/null
source "$ROOT/scripts/ensure-rust.sh"

INSTALL_DEPS="${LPMR_INSTALL_DEPS:-0}"
DAEMON="${LPMR_DAEMON:-0}"

if [[ -z "${LPMR_BIND:-}" ]]; then
  if command -v pvesm >/dev/null 2>&1; then
  # Proxmox node: listen on all interfaces so the UI is reachable from your workstation
    BIND="0.0.0.0:8787"
  else
    BIND="127.0.0.1:8787"
  fi
else
  BIND="$LPMR_BIND"
fi

if [[ "$INSTALL_DEPS" == "1" ]] && [[ -x scripts/install-deps.sh ]]; then
  sudo scripts/install-deps.sh || true
fi

if [[ ! -x target/release/lpmr ]]; then
  echo "Building lpmr (release)…"
  cargo build --release
fi

if [[ "$BIND" == *"0.0.0.0"* ]] && [[ -z "${LPMR_EXECUTE_TOKEN:-}" ]]; then
  echo "WARNING: public bind without LPMR_EXECUTE_TOKEN — remote real execution is blocked unless you set the token." >&2
fi

DISPLAY_HOST="${BIND/0.0.0.0/$(hostname -f 2>/dev/null || echo localhost)}"
DISPLAY_HOST="${DISPLAY_HOST/127.0.0.1/localhost}"
echo "LPMR orchestrator → http://${DISPLAY_HOST}"
echo "  Web UI + API  |  Terminal: lpmr tui"
echo "  Stop: Ctrl+C  |  Background: LPMR_DAEMON=1 ./start.sh"

if [[ "$DAEMON" == "1" ]]; then
  PIDFILE="${ROOT}/.lpmr.pid"
  LOGFILE="${LPMR_LOG:-${ROOT}/lpmr-web.log}"
  if [[ -f "$PIDFILE" ]] && kill -0 "$(cat "$PIDFILE")" 2>/dev/null; then
    echo "Already running (PID $(cat "$PIDFILE")). Stop with: kill \$(cat $PIDFILE)"
    exit 0
  fi
  nohup ./target/release/lpmr web --bind "$BIND" >>"$LOGFILE" 2>&1 &
  echo $! >"$PIDFILE"
  sleep 1
  if kill -0 "$(cat "$PIDFILE")" 2>/dev/null; then
    echo "Started in background PID $(cat "$PIDFILE"), log: $LOGFILE"
    exit 0
  fi
  echo "error: failed to start — see $LOGFILE" >&2
  exit 1
fi

exec ./target/release/lpmr web --bind "$BIND"
