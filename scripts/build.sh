#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DIST_DIR="$ROOT_DIR/dist"
PROFILE="${PROFILE:-release}"
LINUX_TARGET="i686-unknown-linux-gnu"
WINDOWS_TARGET="i686-pc-windows-gnu"
PLUGIN_NAME="mysql_samp"

GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

log_info() { echo -e "${GREEN}$*${NC}"; }
log_step() { echo -e "${YELLOW}$*${NC}"; }
log_err() { echo -e "${RED}$*${NC}" >&2; }

ensure_target() {
  local target="$1"
  if ! rustup target list --installed | grep -qx "$target"; then
    log_step "Instalando target Rust: $target"
    rustup target add "$target"
  fi
}

ensure_sha256() {
  if command -v sha256sum >/dev/null 2>&1; then
    return 0
  fi
  if command -v shasum >/dev/null 2>&1; then
    return 0
  fi
  log_err "Nenhum utilitário de hash encontrado (sha256sum/shasum)."
  exit 1
}

write_sha256() {
  local file="$1"
  local out="$2"
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$file" > "$out"
  else
    shasum -a 256 "$file" > "$out"
  fi
}

build_target() {
  local target="$1"
  log_step "Compilando target: $target"
  cargo build --profile "$PROFILE" --target "$target"
}

copy_linux() {
  local src="$ROOT_DIR/target/$LINUX_TARGET/$PROFILE/lib${PLUGIN_NAME}.so"
  local dst="$DIST_DIR/${PLUGIN_NAME}.so"
  [[ -f "$src" ]] || { log_err "Artefato Linux não encontrado: $src"; exit 1; }
  cp "$src" "$dst"
  write_sha256 "$dst" "${dst}.sha256"
}

copy_windows() {
  local src="$ROOT_DIR/target/$WINDOWS_TARGET/$PROFILE/${PLUGIN_NAME}.dll"
  local dst="$DIST_DIR/${PLUGIN_NAME}.dll"
  [[ -f "$src" ]] || { log_err "Artefato Windows não encontrado: $src"; exit 1; }
  cp "$src" "$dst"
  write_sha256 "$dst" "${dst}.sha256"
}

main() {
  log_info "Build do plugin SA:MP mysql_samp (Linux + Windows 32-bit)"
  mkdir -p "$DIST_DIR"
  ensure_target "$LINUX_TARGET"
  ensure_target "$WINDOWS_TARGET"
  ensure_sha256
  build_target "$LINUX_TARGET"
  build_target "$WINDOWS_TARGET"
  copy_linux
  copy_windows
  log_info "Concluído. Arquivos gerados em: $DIST_DIR"
}

main "$@"
