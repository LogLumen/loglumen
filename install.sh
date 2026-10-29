#!/bin/sh
set -eu

REPO="LogLumen/loglumen"
BINARIES="loglumen-server loglumen-cli"

err() {
    echo "error: $*" >&2
    exit 1
}

detect_target() {
    os=$(uname -s)
    arch=$(uname -m)
    if [ "$os" = "Darwin" ] && [ "$arch" = "x86_64" ] && [ "$(sysctl -n hw.optional.arm64 2>/dev/null)" = "1" ]; then
        arch=arm64
    fi
    case "$os/$arch" in
        Linux/x86_64 | Linux/amd64) echo x86_64-unknown-linux-musl ;;
        Linux/aarch64 | Linux/arm64) echo aarch64-unknown-linux-gnu ;;
        Darwin/arm64 | Darwin/aarch64) echo aarch64-apple-darwin ;;
        *) err "unsupported platform: $os/$arch" ;;
    esac
}

download() {
    if command -v curl >/dev/null 2>&1; then
        curl -fsSL "$1" -o "$2"
    elif command -v wget >/dev/null 2>&1; then
        wget -qO "$2" "$1"
    else
        err "curl or wget is required"
    fi
}

target=$(detect_target)
version="${LOGLUMEN_VERSION:-latest}"
if [ "$version" = "latest" ]; then
    url="https://github.com/$REPO/releases/latest/download/loglumen-$target.tar.gz"
else
    url="https://github.com/$REPO/releases/download/$version/loglumen-$target.tar.gz"
fi

if [ -n "${LOGLUMEN_INSTALL_DIR:-}" ]; then
    install_dir="$LOGLUMEN_INSTALL_DIR"
elif [ "$(id -u)" -eq 0 ]; then
    install_dir="/usr/local/bin"
else
    install_dir="$HOME/.local/bin"
fi

tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT

echo "Downloading $url"
download "$url" "$tmp/loglumen.tar.gz" || err "download failed"
tar -xzf "$tmp/loglumen.tar.gz" -C "$tmp"

mkdir -p "$install_dir"
for bin in $BINARIES; do
    install -m 755 "$tmp/$bin" "$install_dir/$bin"
    echo "Installed $install_dir/$bin"
done

case ":$PATH:" in
    *":$install_dir:"*) ;;
    *) echo "warning: $install_dir is not in your PATH" >&2 ;;
esac
