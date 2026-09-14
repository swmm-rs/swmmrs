#!/bin/sh
# Test the shell installer entirely against a local, mocked release server.

set -eu

SCRIPT_DIR=$(CDPATH= cd "$(dirname "$0")" && pwd)
ROOT=$(CDPATH= cd "$SCRIPT_DIR/.." && pwd)
INSTALLER="$SCRIPT_DIR/install.sh"

need_cmd() {
    command -v "$1" >/dev/null 2>&1 || {
        printf 'test-install.sh: required command not found: %s\n' "$1" >&2
        exit 1
    }
}

need_cmd awk
need_cmd chmod
need_cmd curl
need_cmd mkdir
need_cmd mktemp
need_cmd python3
need_cmd tar

TEMP_DIR=$(mktemp -d)
SERVER_PID=
cleanup() {
    if [ -n "$SERVER_PID" ]; then
        kill "$SERVER_PID" 2>/dev/null || true
        wait "$SERVER_PID" 2>/dev/null || true
    fi
    rm -rf "$TEMP_DIR"
}
trap cleanup EXIT HUP INT TERM

TAG=rs-9.9.9
TARGETS="x86_64-unknown-linux-gnu aarch64-unknown-linux-gnu x86_64-apple-darwin aarch64-apple-darwin"
RELEASE_DIR="$TEMP_DIR/swmm-rs/swmmrs/releases/download/$TAG"
API_DIR="$TEMP_DIR/api/repos/swmm-rs/swmmrs"
INSTALL_DIR="$TEMP_DIR/install"
HOME_DIR="$TEMP_DIR/home"
PORT_FILE="$TEMP_DIR/server.port"
SERVER_SCRIPT="$TEMP_DIR/server.py"

mkdir -p "$RELEASE_DIR" "$API_DIR" "$HOME_DIR"
cat > "$API_DIR/releases" <<'EOF'
[{"draft":false,"prerelease":false,"tag_name":"py-9.9.10"},{"prerelease":true,"tag_name":"rs-10.0.0-beta.1","draft":false},{"prerelease":false,"tag_name":"rs-9.9.9","draft":false}]
EOF
cat > "$TEMP_DIR/runswmmrs" <<'EOF'
#!/bin/sh
if [ "${1:-}" = "--version" ]; then
    printf '%s\n' 'runswmmrs mock 9.9.9'
else
    printf '%s\n' 'runswmmrs mock executable'
fi
EOF
chmod 755 "$TEMP_DIR/runswmmrs"
for target in $TARGETS; do
    asset="runswmmrs-$target.tar.gz"
    (
        cd "$TEMP_DIR"
        tar -czf "$RELEASE_DIR/$asset" runswmmrs
    )
    python3 - "$RELEASE_DIR/$asset" "$RELEASE_DIR/$asset.sha256" <<'PY'
import hashlib
import pathlib
import sys

archive = pathlib.Path(sys.argv[1])
checksum = hashlib.sha256(archive.read_bytes()).hexdigest()
pathlib.Path(sys.argv[2]).write_text(f"{checksum}  {archive.name}\n", encoding="ascii")
PY
done

cat > "$SERVER_SCRIPT" <<'PY'
import http.server
import pathlib
import socketserver
import sys

root = pathlib.Path(sys.argv[1]).resolve()
port_file = pathlib.Path(sys.argv[2])
handler = lambda *args, **kwargs: http.server.SimpleHTTPRequestHandler(
    *args, directory=str(root), **kwargs
)
with socketserver.TCPServer(("127.0.0.1", 0), handler) as server:
    port_file.write_text(str(server.server_address[1]), encoding="ascii")
    server.serve_forever()
PY
python3 "$SERVER_SCRIPT" "$TEMP_DIR" "$PORT_FILE" >/dev/null 2>&1 &
SERVER_PID=$!
for _ in $(awk 'BEGIN { for (i = 1; i <= 100; i++) print i }'); do
    [ -s "$PORT_FILE" ] && break
    sleep 0.05
done
[ -s "$PORT_FILE" ] || {
    printf '%s\n' 'test-install.sh: local release server did not start' >&2
    exit 1
}
PORT=$(cat "$PORT_FILE")

HOME="$HOME_DIR" \
SWMMRS_GITHUB_BASE_URL="http://127.0.0.1:$PORT" \
SWMMRS_GITHUB_API_URL="http://127.0.0.1:$PORT/api" \
SWMMRS_VERSION=latest \
SWMMRS_INSTALL_DIR="$INSTALL_DIR" \
SWMMRS_NO_MODIFY_PATH=1 \
PATH="$PATH" \
sh "$INSTALLER"

[ -f "$INSTALL_DIR/runswmmrs" ]
[ "$("$INSTALL_DIR/runswmmrs" --version)" = 'runswmmrs mock 9.9.9' ]

LEGACY_INSTALL_DIR="$TEMP_DIR/install-legacy-version"
HOME="$HOME_DIR" \
SWMMRS_GITHUB_BASE_URL="http://127.0.0.1:$PORT" \
SWMMRS_GITHUB_API_URL="http://127.0.0.1:$PORT/api" \
SWMMRS_VERSION=v9.9.9 \
SWMMRS_INSTALL_DIR="$LEGACY_INSTALL_DIR" \
SWMMRS_NO_MODIFY_PATH=1 \
PATH="$PATH" \
sh "$INSTALLER"
[ -f "$LEGACY_INSTALL_DIR/runswmmrs" ]
[ "$("$LEGACY_INSTALL_DIR/runswmmrs" --version)" = 'runswmmrs mock 9.9.9' ]

[ ! -e "$HOME_DIR/.profile" ]
[ ! -e "$HOME_DIR/.bashrc" ]

printf '%s\n' 'shell installer mock test passed'
