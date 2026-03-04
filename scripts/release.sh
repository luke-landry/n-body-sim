#!/usr/bin/env bash
set -euo pipefail

echo "[INFO] Running n-body-sim release script..."

require_cmd() {
    local cmd="$1"
    if ! command -v "$cmd" >/dev/null 2>&1; then
        echo "[ERROR] Missing required command: $cmd"
        exit 1
    fi
}

echo "[INFO] Checking required tools..."
require_cmd cargo
require_cmd tar
require_cmd zip
require_cmd sha256sum

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DIST_DIR="$ROOT_DIR/dist"
WINDOWS_TARGET="x86_64-pc-windows-gnu"

RELEASE_VERSION="$(cargo pkgid | sed 's/.*#//')"

echo "[INFO] Detected release version: $RELEASE_VERSION"

if [[ -z "$RELEASE_VERSION" ]]; then
    echo "[ERROR] Could not determine release version from Cargo"
    exit 1
fi

RELEASE_TAG="v$RELEASE_VERSION"

LINUX_NAME="n-body-sim-${RELEASE_TAG}-linux-x86_64"
WINDOWS_NAME="n-body-sim-${RELEASE_TAG}-windows-x86_64"

LINUX_STAGE="$DIST_DIR/$LINUX_NAME"
WINDOWS_STAGE="$DIST_DIR/$WINDOWS_NAME"

LINUX_ARCHIVE="$DIST_DIR/${LINUX_NAME}.tar.gz"
WINDOWS_ARCHIVE="$DIST_DIR/${WINDOWS_NAME}.zip"
CHECKSUMS_FILE="$DIST_DIR/SHA256SUMS"

copy_common_assets() {
    local destination="$1"

    sed "s/{{VERSION}}/$RELEASE_TAG/g" "$ROOT_DIR/README_release_template.txt" > "$destination/README.txt"
    cp "$ROOT_DIR/requirements.txt" "$destination/requirements.txt"

    cp -a "$ROOT_DIR/gui" "$destination/gui"
    find "$destination/gui" -type d -name "__pycache__" -prune -exec rm -rf {} +
    find "$destination/gui" -type f -name "*.pyc" -delete

    cp -a "$ROOT_DIR/data/examples" "$destination/data/examples"
}

echo "[INFO] Building Linux release binary..."
cargo build --release --manifest-path "$ROOT_DIR/Cargo.toml"

echo "[INFO] Building Windows release binary ($WINDOWS_TARGET)..."
cargo build --release --target "$WINDOWS_TARGET" --manifest-path "$ROOT_DIR/Cargo.toml"

LINUX_BIN="$ROOT_DIR/target/release/n-body-sim"
WINDOWS_BIN="$ROOT_DIR/target/$WINDOWS_TARGET/release/n-body-sim.exe"

if [[ ! -f "$LINUX_BIN" ]]; then
    echo "[ERROR] Linux binary not found: $LINUX_BIN"
    exit 1
fi

if [[ ! -f "$WINDOWS_BIN" ]]; then
    echo "[ERROR] Windows binary not found: $WINDOWS_BIN"
    exit 1
fi

echo "[INFO] Preparing dist directory..."
rm -rf "$DIST_DIR"
mkdir -p "$LINUX_STAGE/bin" "$WINDOWS_STAGE/bin"
mkdir -p "$LINUX_STAGE/data" "$WINDOWS_STAGE/data"

copy_common_assets "$LINUX_STAGE"
copy_common_assets "$WINDOWS_STAGE"

cp "$ROOT_DIR/install.sh" "$LINUX_STAGE/install.sh"
cp "$ROOT_DIR/run.sh" "$LINUX_STAGE/run.sh"
cp "$ROOT_DIR/install.bat" "$WINDOWS_STAGE/install.bat"
cp "$ROOT_DIR/run.bat" "$WINDOWS_STAGE/run.bat"

cp "$LINUX_BIN" "$LINUX_STAGE/bin/n-body-sim"
cp "$WINDOWS_BIN" "$WINDOWS_STAGE/bin/n-body-sim.exe"

chmod +x "$LINUX_STAGE/install.sh" "$LINUX_STAGE/run.sh" "$LINUX_STAGE/bin/n-body-sim"

echo "[INFO] Creating release archives..."
(
    cd "$DIST_DIR"

    if [[ ! -d "$LINUX_NAME" || ! -d "$WINDOWS_NAME" ]]; then
        echo "[ERROR] Staging directories missing in $DIST_DIR"
        exit 1
    fi

    tar -czf "$(basename "$LINUX_ARCHIVE")" "$LINUX_NAME"
    zip -r "$(basename "$WINDOWS_ARCHIVE")" "$WINDOWS_NAME" >/dev/null

    echo "[INFO] Writing checksums..."
    sha256sum "$(basename "$LINUX_ARCHIVE")" "$(basename "$WINDOWS_ARCHIVE")" >"$(basename "$CHECKSUMS_FILE")"
)

echo "[SUCCESS] Release artifacts created in: $DIST_DIR"
echo "  - $LINUX_ARCHIVE"
echo "  - $WINDOWS_ARCHIVE"
echo "  - $CHECKSUMS_FILE"
