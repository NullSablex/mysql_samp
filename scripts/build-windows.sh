#!/usr/bin/env bash
# Builds the plugin for Windows and Linux from Windows (Git Bash).
#
# Outputs:
#   dist/<plugin>.dll — Windows (i686-pc-windows-msvc)
#   dist/<plugin>.so  — Linux   (i686-unknown-linux-gnu via WSL or Docker/cross)
#
# Always builds with SA-MP + native Open Multiplayer support.
#
# Linux (.so) build:
#   WSL    — autodetected. Requires Rust inside WSL (https://rustup.rs).
#   Docker — fallback when WSL is unavailable. Requires Docker Desktop + cross.
#   Force a mode with --wsl or --docker.
#
# Usage:
#   ./scripts/build-windows.sh              # autodetect
#   ./scripts/build-windows.sh --wsl        # force WSL for Linux
#   ./scripts/build-windows.sh --docker     # force Docker for Linux
#   PROFILE=dev ./scripts/build-windows.sh  # dev build (default: release)
#
# PLUGIN_NAME is read from the project's Cargo.toml.

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DIST_DIR="$ROOT_DIR/dist"
PROFILE="${PROFILE:-release}"
PLUGIN_NAME="$(grep -m1 '^name' "$ROOT_DIR/Cargo.toml" | sed 's/.*= *"\(.*\)"/\1/' | tr '-' '_')"
LINUX_BUILD=""  # "wsl" | "docker" | "" (autodetect)

GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

log_info() { echo -e "${GREEN}[build] $*${NC}"; }
log_step() { echo -e "${YELLOW}[build] $*${NC}"; }
log_err()  { echo -e "${RED}[build] $*${NC}" >&2; }

for arg in "$@"; do
  case "$arg" in
    --wsl)    LINUX_BUILD="wsl" ;;
    --docker) LINUX_BUILD="docker" ;;
    *) log_err "Unknown argument: $arg"; exit 1 ;;
  esac
done

ensure_target() {
  if ! rustup target list --installed | grep -qx "$1"; then
    log_step "Installing target: $1"
    rustup target add "$1"
  fi
}

ensure_cross() {
  if ! command -v cross >/dev/null 2>&1; then
    log_step "Installing cross..."
    cargo install cross
  fi
}

build_windows() {
  local target="i686-pc-windows-msvc"
  ensure_target "$target"
  log_step "Building: $target"
  cargo build --profile "$PROFILE" --target "$target"

  local src="$ROOT_DIR/target/$target/$PROFILE/${PLUGIN_NAME}.dll"
  local dst="$DIST_DIR/${PLUGIN_NAME}.dll"
  [[ -f "$src" ]] || { log_err "Artifact not found: $src"; exit 1; }
  cp "$src" "$dst"
  log_info "Windows: $dst"
}

build_linux_wsl() {
  local target="i686-unknown-linux-gnu"
  local wsl_root="/mnt${ROOT_DIR}"
  log_step "Building: $target (via WSL)"
  wsl bash -c "rustup target add '$target' 2>/dev/null; cd '$wsl_root' && cargo build --profile '$PROFILE' --target '$target'"

  local src="$ROOT_DIR/target/$target/$PROFILE/lib${PLUGIN_NAME}.so"
  local dst="$DIST_DIR/${PLUGIN_NAME}.so"
  [[ -f "$src" ]] || { log_err "Artifact not found: $src"; exit 1; }
  cp "$src" "$dst"
  log_info "Linux:   $dst"
}

build_linux_docker() {
  local target="i686-unknown-linux-gnu"
  ensure_target "$target"
  ensure_cross
  log_step "Building: $target (via cross/Docker)"
  cross build --profile "$PROFILE" --target "$target"

  local src="$ROOT_DIR/target/$target/$PROFILE/lib${PLUGIN_NAME}.so"
  local dst="$DIST_DIR/${PLUGIN_NAME}.so"
  [[ -f "$src" ]] || { log_err "Artifact not found: $src"; exit 1; }
  cp "$src" "$dst"
  log_info "Linux:   $dst"
}

build_linux() {
  case "$LINUX_BUILD" in
    wsl)    build_linux_wsl ;;
    docker) build_linux_docker ;;
    *)
      if command -v wsl >/dev/null 2>&1; then
        log_step "WSL detected."
        build_linux_wsl
      elif command -v docker >/dev/null 2>&1; then
        log_step "Docker detected."
        build_linux_docker
      else
        log_err "Neither WSL nor Docker found. Install one or force a mode with --wsl/--docker."
        exit 1
      fi
      ;;
  esac
}

main() {
  mkdir -p "$DIST_DIR"
  log_info "Mode: SA-MP + native Open Multiplayer"
  build_windows
  build_linux
  log_info "Done: $DIST_DIR/"
}

main
