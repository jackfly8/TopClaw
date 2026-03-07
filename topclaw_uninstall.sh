#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'USAGE'
TopClaw uninstall helper

Usage:
  ./topclaw_uninstall.sh [--purge]

Options:
  --purge   Remove ~/.topclaw runtime data in addition to binary/service artifacts
  -h, --help

Examples:
  ./topclaw_uninstall.sh
  ./topclaw_uninstall.sh --purge
USAGE
}

info() {
  echo "==> $*"
}

warn() {
  echo "warning: $*" >&2
}

have_cmd() {
  command -v "$1" >/dev/null 2>&1
}

run_maybe_sudo() {
  if "$@"; then
    return 0
  fi

  if have_cmd sudo; then
    sudo "$@"
    return 0
  fi

  return 1
}

PURGE_DATA=0

while [[ $# -gt 0 ]]; do
  case "$1" in
    --purge)
      PURGE_DATA=1
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "error: unknown option: $1" >&2
      usage >&2
      exit 1
      ;;
  esac
done

TOPCLAW_BIN="$(command -v topclaw || true)"
CONFIG_DIR="${HOME}/.topclaw"
MACOS_SERVICE="${HOME}/Library/LaunchAgents/com.topclaw.daemon.plist"
SYSTEMD_SERVICE="${HOME}/.config/systemd/user/topclaw.service"

info "Stopping TopClaw processes"
pkill -f topclaw >/dev/null 2>&1 || true

if [[ -n "$TOPCLAW_BIN" ]]; then
  info "Removing TopClaw background service"
  "$TOPCLAW_BIN" service stop >/dev/null 2>&1 || true
  "$TOPCLAW_BIN" service uninstall >/dev/null 2>&1 || run_maybe_sudo "$TOPCLAW_BIN" service uninstall >/dev/null 2>&1 || true
else
  warn "topclaw binary is not currently on PATH; continuing with manual cleanup"
fi

if [[ -f "$SYSTEMD_SERVICE" ]]; then
  info "Removing systemd user service"
  rm -f "$SYSTEMD_SERVICE"
  if have_cmd systemctl; then
    systemctl --user daemon-reload >/dev/null 2>&1 || true
  fi
fi

if [[ -f "$MACOS_SERVICE" ]]; then
  info "Removing launchd service"
  rm -f "$MACOS_SERVICE"
fi

if have_cmd brew && brew list topclaw >/dev/null 2>&1; then
  info "Uninstalling Homebrew package"
  brew uninstall topclaw || warn "brew uninstall topclaw failed"
fi

if have_cmd cargo; then
  info "Uninstalling cargo package"
  cargo uninstall topclaw >/dev/null 2>&1 || true
fi

if [[ -x "${HOME}/.cargo/bin/topclaw" || -f "${HOME}/.cargo/bin/topclaw" ]]; then
  info "Removing ~/.cargo/bin/topclaw"
  rm -f "${HOME}/.cargo/bin/topclaw"
fi

if [[ "$PURGE_DATA" == "1" ]]; then
  info "Removing ${CONFIG_DIR}"
  rm -rf "$CONFIG_DIR"
fi

info "Uninstall complete"
echo
echo "Verify:"
echo "  command -v topclaw || echo \"topclaw binary not found\""
echo "  pgrep -fl topclaw || echo \"No running topclaw process\""
if [[ "$PURGE_DATA" != "1" ]]; then
  echo
  echo "To remove local config, logs, auth profiles, and workspace data too:"
  echo "  ./topclaw_uninstall.sh --purge"
fi
