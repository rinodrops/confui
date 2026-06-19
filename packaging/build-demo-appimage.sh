#!/usr/bin/env bash
# Build a demo AppImage from linux-build outputs.
# Requires: linuxdeploy and appimagetool (paths via LINUXDEPLOY / APPIMAGETOOL env).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

VERSION="${VERSION:?VERSION is required}"
DIST="demo/dist"
APPDIR="${DIST}/Settings-v${VERSION}-x86_64.AppDir"
OUTPUT="${DIST}/Settings-v${VERSION}-x86_64.AppImage"

LINUXDEPLOY="${LINUXDEPLOY:?LINUXDEPLOY is required}"
export APPIMAGETOOL="${APPIMAGETOOL:?APPIMAGETOOL is required}"

rm -rf "${APPDIR}"
mkdir -p "${APPDIR}"

cp "${DIST}/linux-x86_64/settings" "${APPDIR}/settings"
cp demo/config.toml "${APPDIR}/config.toml"
cp assets/appicon.png "${APPDIR}/settings-demo.png"
cp packaging/settings-demo.desktop "${APPDIR}/settings-demo.desktop"
chmod +x "${APPDIR}/settings"

cat > "${APPDIR}/AppRun" <<'EOF'
#!/bin/sh
HERE="$(dirname "$(readlink -f "$0" 2>/dev/null || realpath "$0")")"
cd "$HERE"
exec "$HERE/settings" "$HERE/config.toml"
EOF
chmod +x "${APPDIR}/AppRun"

"${LINUXDEPLOY}" --appdir "${APPDIR}" \
    --executable "${APPDIR}/settings" \
    --desktop-file "${APPDIR}/settings-demo.desktop" \
    --icon-file "${APPDIR}/settings-demo.png" \
    --output appimage

# linuxdeploy names the output; move to the expected release filename.
BUILT="$(ls -1 Settings*.AppImage 2>/dev/null | head -1 || true)"
if [ -n "${BUILT}" ] && [ "${BUILT}" != "${OUTPUT}" ]; then
    mv -f "${BUILT}" "${OUTPUT}"
fi

echo ">>> AppImage: ${OUTPUT}"
