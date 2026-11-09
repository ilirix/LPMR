#!/usr/bin/env bash
# Fresh Debian-style smoke test (clone → build → API + HTML checks)
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DEST="${LPMR_TEST_DIR:-/tmp/lpmr-fresh-test}"
BRANCH="${LPMR_BRANCH:-cursor/lpmr-volume-panel-5c83}"
BIND="${LPMR_BIND:-127.0.0.1:18787}"
export PATH="${HOME}/.cargo/bin:/usr/local/cargo/bin:${PATH}"

rm -rf "$DEST"
git clone --depth 1 --branch "$BRANCH" https://github.com/ilirix/LPMR.git "$DEST"
cd "$DEST"

if ! command -v cargo >/dev/null; then
  echo "error: rustc/cargo not installed (curl https://sh.rustup.rs | sh)"
  exit 1
fi

cargo build --release
./target/release/lpmr doctor || true

./target/release/lpmr web --bind "$BIND" &
PID=$!
trap 'kill "$PID" 2>/dev/null || true' EXIT
sleep 2

fail=0
check() {
  local path="$1"
  local expect="${2:-200}"
  local code
  code=$(curl -sS -o /dev/null -w "%{http_code}" "http://${BIND}${path}") || code="000"
  if [[ "$code" != "$expect" ]]; then
    echo "FAIL $path (HTTP $code, wanted $expect)"
    fail=1
  else
    echo "OK   $path"
  fi
}

check / 200
check /assets/app.js?v=7 200
check /assets/app.css?v=7 200
check /api/telemetry 200
check /api/devices 200
check /api/lvm 200
check /api/health-audit 200

html=$(curl -sS "http://${BIND}/")
echo "$html" | grep -q "fonts.googleapis.com" || { echo "FAIL / missing Google Fonts (use assets/web/index.html)"; fail=1; }
echo "$html" | grep -q "app.js?v=7" || { echo "FAIL stale asset cache version"; fail=1; }

if [[ "$fail" -eq 0 ]]; then
  echo "debian-fresh-test: all checks passed"
else
  exit 1
fi
