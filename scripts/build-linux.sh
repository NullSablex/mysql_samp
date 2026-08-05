#!/usr/bin/env bash
# Builds the plugin for Linux and Windows from Linux.
#
# Outputs:
#   dist/<plugin>.so  — Linux  (i686-unknown-linux-gnu)
#   dist/<plugin>.dll — Windows (i686-pc-windows-msvc via cargo-xwin)
#
# Always builds with SA-MP + native Open Multiplayer support.
# Requires cargo-xwin (installed automatically if missing).
#
# Usage:
#   ./scripts/build-linux.sh             # release
#   PROFILE=dev ./scripts/build-linux.sh # dev build
#
# PLUGIN_NAME is read from the project's Cargo.toml.

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DIST_DIR="$ROOT_DIR/dist"
PROFILE="${PROFILE:-release}"
PLUGIN_NAME="$(grep -m1 '^name' "$ROOT_DIR/Cargo.toml" | sed 's/.*= *"\(.*\)"/\1/' | tr '-' '_')"

GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

log_info() { echo -e "${GREEN}[build] $*${NC}"; }
log_step() { echo -e "${YELLOW}[build] $*${NC}"; }
log_err()  { echo -e "${RED}[build] $*${NC}" >&2; }

for arg in "$@"; do
  log_err "Unknown argument: $arg"
  exit 1
done

ensure_target() {
  if ! rustup target list --installed | grep -qx "$1"; then
    log_step "Installing target: $1"
    rustup target add "$1"
  fi
}

ensure_xwin() {
  if ! command -v cargo-xwin >/dev/null 2>&1; then
    log_step "Installing cargo-xwin..."
    cargo install cargo-xwin
  fi
}

# `ring` (the TLS backend behind MYSQL_OPT_SSL) compiles C, so the MSVC target
# needs `llvm-lib` to archive the objects. cargo-xwin provides clang-cl and
# lld-link but not this one, and LLVM ships it un-suffixed only inside its own
# libdir — so expose it on PATH when it is missing.
ensure_llvm_lib() {
  command -v llvm-lib >/dev/null 2>&1 && return 0

  local found
  found="$(ls -d /usr/lib/llvm-*/bin/llvm-lib 2>/dev/null | sort -V | tail -1)"
  if [[ -z "$found" ]]; then
    log_err "llvm-lib not found. Install LLVM (e.g. 'apt install llvm') to build the Windows target."
    exit 1
  fi

  local shim="$HOME/.cache/cargo-xwin"
  mkdir -p "$shim"
  ln -sf "$found" "$shim/llvm-lib"
  export PATH="$shim:$PATH"
  log_info "llvm-lib: $found"
}

# The TLS backend (`ring`) compiles C and 32-bit assembly, so the i686 target
# now needs 32-bit libc headers. Before TLS was enabled this build was pure
# Rust and worked without them, and cc-rs fails with an opaque missing-header
# error rather than naming the real cause.
ensure_multilib() {
  if echo 'int main(){return 0;}' | gcc -m32 -x c - -o /dev/null >/dev/null 2>&1; then
    return 0
  fi
  log_err "32-bit C support is missing: 'gcc -m32' cannot build."
  log_err "The TLS backend compiles C for i686. Install it with:"
  log_err "  Debian/Ubuntu:  sudo apt install gcc-multilib g++-multilib"
  log_err "  Fedora:         sudo dnf install glibc-devel.i686 libstdc++-devel.i686"
  log_err "  Arch:           sudo pacman -S lib32-glibc lib32-gcc-libs"
  exit 1
}

build_linux() {
  local target="i686-unknown-linux-gnu"
  ensure_multilib
  ensure_target "$target"
  log_step "Building: $target"
  cargo build --profile "$PROFILE" --target "$target"

  local src="$ROOT_DIR/target/$target/$PROFILE/lib${PLUGIN_NAME}.so"
  local dst="$DIST_DIR/${PLUGIN_NAME}.so"
  [[ -f "$src" ]] || { log_err "Artifact not found: $src"; exit 1; }
  cp "$src" "$dst"
  log_info "Linux:   $dst"
}

build_windows() {
  local target="i686-pc-windows-msvc"
  ensure_target "$target"
  ensure_xwin
  ensure_llvm_lib
  log_step "Building: $target"
  cargo xwin build --xwin-arch x86 --profile "$PROFILE" --target "$target"

  local src="$ROOT_DIR/target/$target/$PROFILE/${PLUGIN_NAME}.dll"
  local dst="$DIST_DIR/${PLUGIN_NAME}.dll"
  [[ -f "$src" ]] || { log_err "Artifact not found: $src"; exit 1; }
  cp "$src" "$dst"
  log_info "Windows: $dst"
}

main() {
  mkdir -p "$DIST_DIR"
  log_info "Mode: SA-MP + native Open Multiplayer"
  build_linux
  build_windows
  log_info "Done: $DIST_DIR/"
}

main
