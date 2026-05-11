#!/bin/sh
# Memcrate installer.
#
#   curl -fsSL https://raw.githubusercontent.com/bradtraversy/memcrate/main/install.sh | sh
#
# Optional environment variables:
#   MEMCRATE_VERSION      Specific version tag to install (default: latest).
#   MEMCRATE_INSTALL_DIR  Where to put the binary (default: /usr/local/bin,
#                         falls back to $HOME/.local/bin if not writable).

set -eu

REPO="bradtraversy/memcrate"
BIN_NAME="memcrate"

err() { printf 'error: %s\n' "$*" >&2; exit 1; }
info() { printf '%s\n' "$*"; }

require_cmd() {
    command -v "$1" >/dev/null 2>&1 || err "missing required command: $1"
}

detect_target() {
    os=$(uname -s)
    arch=$(uname -m)
    case "$os" in
        Linux)
            case "$arch" in
                x86_64|amd64) echo "x86_64-unknown-linux-gnu" ;;
                *) err "unsupported Linux architecture: $arch (only x86_64 is published in v0.1)" ;;
            esac
            ;;
        Darwin)
            case "$arch" in
                x86_64) echo "x86_64-apple-darwin" ;;
                arm64|aarch64) echo "aarch64-apple-darwin" ;;
                *) err "unsupported macOS architecture: $arch" ;;
            esac
            ;;
        *) err "unsupported OS: $os (Linux and macOS are published in v0.1; Windows users see the docs at https://memcrate.dev)" ;;
    esac
}

resolve_install_dir() {
    if [ -n "${MEMCRATE_INSTALL_DIR:-}" ]; then
        echo "$MEMCRATE_INSTALL_DIR"
        return
    fi

    if [ -w /usr/local/bin ] 2>/dev/null; then
        echo "/usr/local/bin"
    elif [ "$(id -u)" -eq 0 ]; then
        echo "/usr/local/bin"
    elif command -v sudo >/dev/null 2>&1; then
        # /usr/local/bin via sudo is the standard expectation; fall back to user-local if sudo is unavailable.
        echo "/usr/local/bin"
    else
        echo "$HOME/.local/bin"
    fi
}

main() {
    require_cmd curl
    require_cmd tar
    require_cmd uname

    target=$(detect_target)
    version="${MEMCRATE_VERSION:-latest}"
    install_dir=$(resolve_install_dir)

    if [ "$version" = "latest" ]; then
        url="https://github.com/${REPO}/releases/latest/download/${BIN_NAME}-${target}.tar.gz"
    else
        url="https://github.com/${REPO}/releases/download/${version}/${BIN_NAME}-${target}.tar.gz"
    fi

    info "Memcrate installer"
    info "  target:      $target"
    info "  version:     $version"
    info "  install dir: $install_dir"
    info ""
    info "Downloading $url"

    tmp=$(mktemp -d)
    trap 'rm -rf "$tmp"' EXIT

    if ! curl --fail --silent --show-error --location "$url" -o "$tmp/${BIN_NAME}.tar.gz"; then
        err "download failed. Check that version '$version' has a release asset for '$target' at https://github.com/${REPO}/releases"
    fi

    tar -xzf "$tmp/${BIN_NAME}.tar.gz" -C "$tmp"

    if [ ! -f "$tmp/${BIN_NAME}" ]; then
        err "tarball did not contain expected binary '${BIN_NAME}'. Contents: $(ls "$tmp")"
    fi

    chmod +x "$tmp/${BIN_NAME}"

    mkdir -p "$install_dir"
    dest="$install_dir/${BIN_NAME}"

    if [ -w "$install_dir" ]; then
        mv "$tmp/${BIN_NAME}" "$dest"
    else
        info "Elevating to write to $install_dir (sudo password may be required)"
        sudo mv "$tmp/${BIN_NAME}" "$dest"
    fi

    info ""
    info "Installed: $dest"
    "$dest" --version

    case ":$PATH:" in
        *":$install_dir:"*) ;;
        *)
            info ""
            info "Note: $install_dir is not on your PATH. Add this to your shell profile:"
            info "  export PATH=\"$install_dir:\$PATH\""
            ;;
    esac

    info ""
    info "Next:"
    info "  memcrate init ~/vault    # scaffold a vault"
    info "  memcrate --help          # see available commands"
    info "  https://memcrate.dev     # docs"
}

main "$@"
