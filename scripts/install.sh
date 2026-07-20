#!/bin/sh

set -eu

BRASS_VERSION="${BRASS_VERSION:-latest}"
BRASS_REPO_BASE_URL="https://github.com/brass-cz/brass"

# print host-tuple
print_host_tuple() {
    if [ -z "${TOOLCHAIN_OS:-}" ]; then
        # Get OS
        case "$(uname -s)" in
            Linux)
                TOOLCHAIN_OS="unknown-linux-gnu"
                ;;
            Darwin)
                TOOLCHAIN_OS="apple-darwin"
                ;;
            CYGWIN*|MINGW32*|MSYS*|MINGW*)
                TOOLCHAIN_OS="pc-windows-msvc"
                ;;
            *)
                echo "Unsupported OS: $(uname -s)" >&2
                exit 1
                ;;
        esac
    fi

    if [ -z "${TOOLCHAIN_ARCH:-}" ]; then
        # Get architecture
        case "$(uname -m)" in
            arm64|aarch64)
                TOOLCHAIN_ARCH="aarch64"
                ;;
            x86_64|amd64)
                TOOLCHAIN_ARCH="x86_64"
                ;;
            *)
                echo "Unsupported architecture: $(uname -m)" >&2
                exit 1
                ;;
        esac
    fi

    echo "$TOOLCHAIN_ARCH-$TOOLCHAIN_OS"
}

HOST_TUPLE="$(print_host_tuple)"

#
# main
#
dest="$HOME/.brass"
if ls "$dest" > /dev/null 2>&1; then
    echo -n "$dest is already exists! Would you like to delete it? [Y/n]: "
    read delete_exists < /dev/tty
    if [ "$(echo "$delete_exists" | awk '{ print tolower($0) }')" = "y" ]; then
        rm -rf "$dest"
    else
        exit 1
    fi
fi
mkdir -p "$dest"
tar_file="brass-$HOST_TUPLE.tar.gz"
if [ "$BRASS_VERSION" = "latest" ]; then
    url="$BRASS_REPO_BASE_URL/releases/latest/download/$tar_file"
else
    url="$BRASS_REPO_BASE_URL/releases/download/$BRASS_VERSION/$tar_file"
fi
download_dir="$(mktemp -d)"
trap 'rm -rf "$download_dir"' 0
echo
echo "Downloading toolchain..."
curl -fSL "$url" -o "$download_dir/$tar_file"
echo
echo "Extracting toolchain..."
tar -xzf "$download_dir/$tar_file" -C "$dest"

echo
echo 'Add $HOME/.brass/bin to your PATH to complete the installation.'
