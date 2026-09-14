#!/bin/sh
# Install runswmmrs from a GitHub release.
# Usage: curl --proto '=https' --tlsv1.2 -LsSf https://raw.githubusercontent.com/swmm-rs/swmmrs/main/scripts/install.sh | sh

set -eu

REPOSITORY="swmm-rs/swmmrs"
BINARY="runswmmrs"
GITHUB_BASE_URL="${SWMMRS_GITHUB_BASE_URL:-https://github.com}"
GITHUB_API_URL="${SWMMRS_GITHUB_API_URL:-https://api.github.com}"
VERSION="${SWMMRS_VERSION:-latest}"
INSTALL_DIR="${SWMMRS_INSTALL_DIR:-${HOME:?HOME must be set}/.local/bin}"
NO_MODIFY_PATH="${SWMMRS_NO_MODIFY_PATH:-0}"

say() {
    printf '%s\n' "$*"
}

err() {
    say "error: $*" >&2
    exit 1
}

need_cmd() {
    command -v "$1" >/dev/null 2>&1 || err "required command not found: $1"
}

download() {
    url=$1
    destination=$2

    if command -v curl >/dev/null 2>&1; then
        case "$url" in
            https://*) curl --proto '=https' --tlsv1.2 -LsSf "$url" -o "$destination" ;;
            *) curl -LsSf "$url" -o "$destination" ;;
        esac
    elif command -v wget >/dev/null 2>&1; then
        wget -q "$url" -O "$destination"
    else
        err "curl or wget is required to download runswmmrs"
    fi
}

fetch() {
    url=$1

    if command -v curl >/dev/null 2>&1; then
        case "$url" in
            https://*) curl --proto '=https' --tlsv1.2 -LsSf "$url" ;;
            *) curl -LsSf "$url" ;;
        esac
    elif command -v wget >/dev/null 2>&1; then
        wget -q "$url" -O -
    else
        err "curl or wget is required to find the latest runswmmrs release"
    fi
}

resolve_latest_rust_tag() {
    page=1
    while [ "$page" -le 100 ]; do
        releases=$(fetch "$GITHUB_API_URL/repos/$REPOSITORY/releases?per_page=100&page=$page")
        tag=$(printf '%s\n' "$releases" | awk '
        function string_field(json, key, value, pattern) {
            value = json
            pattern = "^.*\"" key "\"[[:space:]]*:[[:space:]]*\""
            if (value !~ pattern) return ""
            sub(pattern, "", value)
            sub(/\".*$/, "", value)
            return value
        }
        function bool_field(json, key, value, pattern) {
            value = json
            pattern = "^.*\"" key "\"[[:space:]]*:[[:space:]]*"
            if (value !~ pattern) return 0
            sub(pattern, "", value)
            return value ~ /^true/
        }
        function consider(json, candidate) {
            candidate = string_field(json, "tag_name")
            if (!bool_field(json, "draft") && !bool_field(json, "prerelease") &&
                candidate ~ /^rs-(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)$/) {
                print candidate
                exit
            }
        }
        {
            line = $0 "\n"
            for (i = 1; i <= length(line); i++) {
                character = substr(line, i, 1)
                if (in_string) {
                    if (depth > 0) object = object character
                    if (escaped) escaped = 0
                    else if (character == "\\") escaped = 1
                    else if (character == "\"") in_string = 0
                } else if (character == "\"") {
                    in_string = 1
                    if (depth > 0) object = object character
                } else if (character == "{") {
                    if (depth == 0) object = ""
                    depth++
                    object = object character
                } else if (character == "}") {
                    if (depth > 0) object = object character
                    depth--
                    if (depth == 0) consider(object)
                } else if (depth > 0) {
                    object = object character
                }
            }
        }
        ')
        if [ -n "$tag" ]; then
            printf '%s\n' "$tag"
            return 0
        fi
        case "$releases" in
            *'{'*) page=$((page + 1)) ;;
            *) break ;;
        esac
    done
    err "no stable rs-* GitHub release is available"
}

detect_target() {
    os=$(uname -s)
    arch=$(uname -m)

    case "$os" in
        Linux)
            command -v getconf >/dev/null 2>&1 \
                || err "getconf is required to detect the system C library"
            libc=$(getconf GNU_LIBC_VERSION 2>/dev/null || true)
            case "$libc" in
                glibc\ *) glibc_version=${libc#glibc } ;;
                *) err "only glibc-based Linux systems are supported by the current release binaries" ;;
            esac
            if ! printf '%s\n' "$glibc_version" | awk -F. \
                '{ exit !(($1 > 2) || ($1 == 2 && $2 >= 35)) }'; then
                err "glibc 2.35 or newer is required (found $glibc_version)"
            fi
            os="unknown-linux-gnu"
            ;;
        Darwin)
            if [ "$arch" = "x86_64" ] && command -v sysctl >/dev/null 2>&1 \
                && [ "$(sysctl -in sysctl.proc_translated 2>/dev/null || true)" = "1" ]; then
                arch="arm64"
            fi
            os="apple-darwin"
            ;;
        *) err "unsupported operating system: $os" ;;
    esac

    case "$arch" in
        x86_64 | amd64) arch="x86_64" ;;
        arm64 | aarch64) arch="aarch64" ;;
        *) err "unsupported architecture: $arch" ;;
    esac

    printf '%s-%s\n' "$arch" "$os"
}

verify_checksum() {
    archive=$1
    checksum_file=$2
    expected=$(awk 'NR == 1 { print tolower($1) }' "$checksum_file")

    case "$expected" in
        *[!0-9A-Fa-f]* | '') err "release checksum is not valid SHA-256" ;;
    esac
    [ "${#expected}" -eq 64 ] || err "release checksum is not valid SHA-256"

    if command -v sha256sum >/dev/null 2>&1; then
        actual=$(sha256sum "$archive" | awk '{ print $1 }')
    elif command -v shasum >/dev/null 2>&1; then
        actual=$(shasum -a 256 "$archive" | awk '{ print $1 }')
    elif command -v openssl >/dev/null 2>&1; then
        actual=$(openssl dgst -sha256 "$archive" | awk '{ print $NF }')
    else
        err "sha256sum, shasum, or openssl is required to verify the download"
    fi

    [ "$actual" = "$expected" ] || err "checksum verification failed for $(basename "$archive")"
}

path_contains() {
    case ":${PATH:-}:" in
        *":$INSTALL_DIR:"*) return 0 ;;
        *) return 1 ;;
    esac
}

escape_double_quotes() {
    printf '%s' "$1" | sed 's/[\\"`$]/\\&/g'
}

append_path_line() {
    profile=$1
    line=$2

    mkdir -p "$(dirname "$profile")"
    if [ ! -f "$profile" ] || ! grep -Fqx "$line" "$profile"; then
        {
            printf '\n# Added by the swmmrs installer\n'
            printf '%s\n' "$line"
        } >> "$profile"
        say "Added $INSTALL_DIR to PATH in $profile"
    fi
}

add_to_path() {
    [ "$NO_MODIFY_PATH" = "0" ] || return 0
    path_contains && return 0

    shell_name=$(basename "${SHELL:-sh}")
    escaped_dir=$(escape_double_quotes "$INSTALL_DIR")

    if [ "$shell_name" = "fish" ]; then
        append_path_line \
            "${XDG_CONFIG_HOME:-$HOME/.config}/fish/conf.d/swmmrs.fish" \
            "fish_add_path \"$escaped_dir\""
    else
        line="export PATH=\"$escaped_dir:\$PATH\""
        case "$shell_name" in
            bash)
                append_path_line "$HOME/.bashrc" "$line"
                append_path_line "$HOME/.profile" "$line"
                ;;
            zsh) append_path_line "${ZDOTDIR:-$HOME}/.zshrc" "$line" ;;
            *) append_path_line "$HOME/.profile" "$line" ;;
        esac
    fi
}

main() {
    need_cmd uname
    need_cmd awk
    need_cmd tar
    need_cmd mktemp
    need_cmd mkdir
    need_cmd dirname
    need_cmd basename
    need_cmd grep
    need_cmd rm
    need_cmd chmod
    need_cmd cp
    need_cmd mv

    case "$VERSION" in
        latest) tag=$(resolve_latest_rust_tag) ;;
        rs-*) tag=$VERSION ;;
        v*) tag="rs-${VERSION#v}" ;;
        *) tag="rs-$VERSION" ;;
    esac
    if ! printf '%s\n' "$tag" \
        | awk '/^rs-(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)([-.+][0-9A-Za-z.-]+)?$/ { valid = 1 } END { exit !valid }'; then
        err "SWMMRS_VERSION must be latest or a Rust release such as rs-0.2.1"
    fi

    target=$(detect_target)
    asset="$BINARY-$target.tar.gz"
    release_path="download/$tag"
    display_version="$tag"

    base_url="$GITHUB_BASE_URL/$REPOSITORY/releases/$release_path"
    tmp_dir=$(mktemp -d 2>/dev/null || mktemp -d -t swmmrs)
    trap 'rm -rf "$tmp_dir"; [ -z "${staged_binary:-}" ] || rm -f "$staged_binary"' EXIT HUP INT TERM
    archive="$tmp_dir/$asset"
    checksum="$tmp_dir/$asset.sha256"

    say "Downloading runswmmrs $display_version for $target"
    download "$base_url/$asset" "$archive"
    download "$base_url/$asset.sha256" "$checksum"
    verify_checksum "$archive" "$checksum"

    mkdir -p "$tmp_dir/unpacked"
    tar -xzf "$archive" -C "$tmp_dir/unpacked"
    [ -f "$tmp_dir/unpacked/$BINARY" ] || err "$asset does not contain $BINARY"

    mkdir -p "$INSTALL_DIR"
    staged_binary="$INSTALL_DIR/.$BINARY.new.$$"
    cp "$tmp_dir/unpacked/$BINARY" "$staged_binary"
    chmod 755 "$staged_binary"
    mv -f "$staged_binary" "$INSTALL_DIR/$BINARY"
    add_to_path

    say "Installed $BINARY to $INSTALL_DIR/$BINARY"
    if ! path_contains; then
        say "Restart your shell or run: export PATH=\"$INSTALL_DIR:\$PATH\""
    fi
}

main "$@"
