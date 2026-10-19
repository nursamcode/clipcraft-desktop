#!/usr/bin/env bash
# ClipCraft Studio Multi-Platform Packaging Matrix
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DIST="${ROOT}/dist/release"
mkdir -p "$DIST"

echo "=== Packaging ClipCraft Studio Releases ==="

# 1. Linux Packaging
build_linux() {
    echo "[+] Building Linux x86_64 binaries..."
    cargo build --release -p filmcraft -p filmcraft-cli
    
    local pkg_dir="${DIST}/clipcraft-studio-linux-x64"
    mkdir -p "$pkg_dir"
    cp target/release/filmcraft "$pkg_dir/clipcraft"
    cp target/release/filmcraft-cli "$pkg_dir/clipcraft-cli"
    cp README.md LICENSE-MIT LICENSE-APACHE "$pkg_dir/"
    
    cd "$DIST"
    tar -czvf clipcraft-studio-linux-x64.tar.gz clipcraft-studio-linux-x64
    rm -rf "$pkg_dir"
    echo "[✓] Linux package ready: $DIST/clipcraft-studio-linux-x64.tar.gz"
}

# 2. Windows Packaging (via MinGW or Native)
build_windows() {
    echo "[+] Building Windows x86_64 package..."
    local pkg_dir="${DIST}/clipcraft-studio-windows-x64"
    mkdir -p "$pkg_dir"
    if [ -f target/release/filmcraft.exe ]; then
        cp target/release/filmcraft.exe "$pkg_dir/clipcraft.exe"
        cp target/release/filmcraft-cli.exe "$pkg_dir/clipcraft-cli.exe"
        cp README.md LICENSE-MIT LICENSE-APACHE "$pkg_dir/"
        cd "$DIST"
        zip -r clipcraft-studio-windows-x64.zip clipcraft-studio-windows-x64
        rm -rf "$pkg_dir"
        echo "[✓] Windows package ready: $DIST/clipcraft-studio-windows-x64.zip"
    else
        echo "[!] No Windows binaries in target/release, skipping Windows archive."
    fi
}

# 3. macOS Universal Packaging
build_macos() {
    echo "[+] Building macOS Universal package..."
    local pkg_dir="${DIST}/clipcraft-studio-macos-universal"
    mkdir -p "$pkg_dir"
    if [ -f target/aarch64-apple-darwin/release/filmcraft ] && [ -f target/x86_64-apple-darwin/release/filmcraft ]; then
        lipo -create -output "$pkg_dir/clipcraft" \
            target/aarch64-apple-darwin/release/filmcraft \
            target/x86_64-apple-darwin/release/filmcraft
        lipo -create -output "$pkg_dir/clipcraft-cli" \
            target/aarch64-apple-darwin/release/filmcraft-cli \
            target/x86_64-apple-darwin/release/filmcraft-cli
        cp README.md LICENSE-MIT LICENSE-APACHE "$pkg_dir/"
        cd "$DIST"
        zip -r clipcraft-studio-macos-universal.zip clipcraft-studio-macos-universal
        rm -rf "$pkg_dir"
        echo "[✓] macOS package ready: $DIST/clipcraft-studio-macos-universal.zip"
    else
        echo "[!] macOS binaries not found in target directories, skipping macOS archive."
    fi
}

case "${1:-all}" in
    linux)   build_linux ;;
    windows) build_windows ;;
    macos)   build_macos ;;
    all)
        build_linux || true
        build_windows || true
        build_macos || true
        ;;
    *)
        echo "Usage: $0 [linux|windows|macos|all]"
        exit 1
        ;;
esac
