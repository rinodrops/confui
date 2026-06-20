# Settings — embed + checker + standalone production GUI
#
# Usage:
#   just init                  — refresh derived assets (run after clone / icon changes)
#   just binary                         — release binary for parent-app bundling
#   just check-schema                   — validate schema without building the GUI
#   just settings-darwin-build-arm64    — production .app → dist/settings/
#   just checker-linux-release          — checker zip for GitHub Releases
#
#   SCHEMA=/path/to/schema.toml just binary
#   ICON_STYLE=outlined just icons
#
# Demo builds: see demo/Justfile.
#
# Windows cross-compile prerequisites (macOS host):
#   brew install mingw-w64
#   rustup target add x86_64-pc-windows-gnu

set windows-shell := ["sh", "-cu"]

app_name      := "Settings"
exe_name      := "settings"
checker_name  := "settings-schema-checker"
bundle_id     := "jp.emotiongraphics.settings"
min_macos     := "11.0"
version       := `awk -F'"' '/^version *=/{print $2; exit}' Cargo.toml`

dist_settings := "dist/settings"
dist_checker  := "dist/settings-schema-checker"

arch_darwin_arm64 := "darwin-arm64"
arch_darwin_x86   := "darwin-x86_64"
arch_win          := "windows-x86_64"
arch_linux        := "linux-x86_64"

rust_target_arm64 := "aarch64-apple-darwin"
rust_target_x86   := "x86_64-apple-darwin"
win_target        := "x86_64-pc-windows-gnu"
win_target_dir    := "/tmp/settings-win"

dmg_settings  := "demo/dmg_settings.py"
entitlements  := "demo/entitlements.plist"
icon_src      := "assets/appicon.png"
iconset       := "dist/AppIcon.iconset"
assets_dir    := "assets"
icon_ttf      := assets_dir + "/icons.ttf"
icon_cp       := assets_dir + "/icons.codepoints"
appicon_ico   := assets_dir + "/appicon.ico"

export SCHEMA := env_var_or_default('SCHEMA', '../schema.toml')

default:
    @just binary

help:
    @just --list

# Refresh derived assets. Safe to re-run after clone or when assets/appicon.png changes.
init:
    #!/usr/bin/env bash
    set -euo pipefail
    if [ -z "${SCHEMA:-}" ] || [ ! -f "${SCHEMA}" ]; then
        export SCHEMA="demo/schema.toml"
    fi
    just icons
    just appicon-ico
    just _init-hints
    echo ">>> init complete"

_init-hints:
    #!/usr/bin/env bash
    case "$(uname -s)" in
        Darwin)
            for t in {{rust_target_arm64}} {{rust_target_x86}} {{win_target}}; do
                rustup target list --installed 2>/dev/null | grep -qx "${t}" || \
                    echo "Note: rustup target add ${t}"
            done
            ;;
        Linux)
            command -v nfpm >/dev/null 2>&1 || \
                echo "Note: install nfpm  (for .deb packaging)"
            ;;
        MINGW*|MSYS*)
            ;;
    esac
    command -v magick >/dev/null 2>&1 || \
        echo "Note: install ImageMagick  (for appicon.ico)"

# ---------------------------------------------------------------------------
# Embed (parent-app bundling)
# ---------------------------------------------------------------------------

binary: _ensure-icon-font
    #!/usr/bin/env bash
    set -euo pipefail
    ABS_SCHEMA="$(just _abs-schema)"
    echo ">>> Building release binary  (schema: ${ABS_SCHEMA})"
    SETTINGS_SCHEMA="${ABS_SCHEMA}" cargo build --release -p settings
    echo ">>> Output: target/release/{{exe_name}}"

check-schema:
    #!/usr/bin/env bash
    set -euo pipefail
    ABS_SCHEMA="$(just _abs-schema)"
    echo ">>> Validating schema  (schema: ${ABS_SCHEMA})"
    cargo run --quiet -p settings-schema --bin {{checker_name}} -- "${ABS_SCHEMA}"

schema-check: check-schema

icons:
    rm -f "{{icon_ttf}}" "{{icon_cp}}"
    just _download-icons

appicon-ico:
    rm -f "{{appicon_ico}}"
    just _build-appicon-ico

# ---------------------------------------------------------------------------
# Standalone production GUI → dist/settings/
# ---------------------------------------------------------------------------

settings-darwin-build: settings-darwin-build-arm64 settings-darwin-build-x86_64

settings-darwin-build-arm64: _ensure-icon-font
    just _settings-darwin-bundle {{arch_darwin_arm64}} {{rust_target_arm64}}
    @echo ">>> Output: {{dist_settings}}/{{arch_darwin_arm64}}/{{app_name}}.app"

settings-darwin-build-x86_64: _ensure-icon-font
    just _settings-darwin-bundle {{arch_darwin_x86}} {{rust_target_x86}}
    @echo ">>> Output: {{dist_settings}}/{{arch_darwin_x86}}/{{app_name}}.app"

[macos]
settings-win-build: _ensure-icon-font _ensure-appicon-ico
    #!/usr/bin/env bash
    set -euo pipefail
    ABS_SCHEMA="$(just _abs-schema)"
    echo ">>> Building Settings .exe  (schema: ${ABS_SCHEMA})"
    SETTINGS_SCHEMA="${ABS_SCHEMA}" CARGO_TARGET_DIR="{{win_target_dir}}" \
        cargo build --release -p settings --target {{win_target}}
    mkdir -p "{{dist_settings}}/{{arch_win}}"
    cp "{{win_target_dir}}/{{win_target}}/release/settings.exe" \
        "{{dist_settings}}/{{arch_win}}/{{app_name}}.exe"
    echo ">>> Output: {{dist_settings}}/{{arch_win}}/{{app_name}}.exe"

[linux]
settings-win-build:
    @echo "Error: Windows cross-compile requires a macOS host" && exit 1

[windows]
settings-win-build: _ensure-icon-font _ensure-appicon-ico
    #!/usr/bin/env bash
    set -euo pipefail
    ABS_SCHEMA="$(just _abs-schema)"
    echo ">>> Building Settings .exe  (schema: ${ABS_SCHEMA})"
    SETTINGS_SCHEMA="${ABS_SCHEMA}" cargo build --release -p settings
    mkdir -p "{{dist_settings}}/{{arch_win}}"
    cp "target/release/settings.exe" "{{dist_settings}}/{{arch_win}}/{{app_name}}.exe"
    echo ">>> Output: {{dist_settings}}/{{arch_win}}/{{app_name}}.exe"

settings-linux-build: _ensure-icon-font
    #!/usr/bin/env bash
    set -euo pipefail
    ABS_SCHEMA="$(just _abs-schema)"
    echo ">>> Building Settings binary  (schema: ${ABS_SCHEMA})"
    SETTINGS_SCHEMA="${ABS_SCHEMA}" cargo build --release -p settings
    mkdir -p "{{dist_settings}}/{{arch_linux}}"
    cp "target/release/{{exe_name}}" "{{dist_settings}}/{{arch_linux}}/{{exe_name}}"
    echo ">>> Output: {{dist_settings}}/{{arch_linux}}/{{exe_name}}"

settings-win-zip: settings-win-build
    #!/usr/bin/env bash
    set -euo pipefail
    ZIP="{{dist_settings}}/{{app_name}}-v{{version}}-windows-x86_64.zip"
    rm -f "${ZIP}"
    cd "{{dist_settings}}/{{arch_win}}" && zip "../$(basename "${ZIP}")" "{{app_name}}.exe"
    echo ">>> Package: ${ZIP}"

settings-darwin-zip-arm64: settings-darwin-build-arm64
    ditto -c -k --keepParent \
        "{{dist_settings}}/{{arch_darwin_arm64}}/{{app_name}}.app" \
        "{{dist_settings}}/{{app_name}}-v{{version}}-darwin-arm64.zip"
    @echo ">>> Package: {{dist_settings}}/{{app_name}}-v{{version}}-darwin-arm64.zip"

settings-darwin-zip-x86_64: settings-darwin-build-x86_64
    ditto -c -k --keepParent \
        "{{dist_settings}}/{{arch_darwin_x86}}/{{app_name}}.app" \
        "{{dist_settings}}/{{app_name}}-v{{version}}-darwin-x86_64.zip"
    @echo ">>> Package: {{dist_settings}}/{{app_name}}-v{{version}}-darwin-x86_64.zip"

settings-darwin-dmg-arm64: settings-darwin-build-arm64
    just _require-dmgbuild
    dmgbuild -s "{{dmg_settings}}" \
        -D app="{{dist_settings}}/{{arch_darwin_arm64}}/{{app_name}}.app" \
        "{{app_name}}" \
        "{{dist_settings}}/{{app_name}}-v{{version}}-darwin-arm64.dmg"
    @echo ">>> Package: {{dist_settings}}/{{app_name}}-v{{version}}-darwin-arm64.dmg"

settings-darwin-dmg-x86_64: settings-darwin-build-x86_64
    just _require-dmgbuild
    dmgbuild -s "{{dmg_settings}}" \
        -D app="{{dist_settings}}/{{arch_darwin_x86}}/{{app_name}}.app" \
        "{{app_name}}" \
        "{{dist_settings}}/{{app_name}}-v{{version}}-darwin-x86_64.dmg"
    @echo ">>> Package: {{dist_settings}}/{{app_name}}-v{{version}}-darwin-x86_64.dmg"

settings-darwin-sign-arm64: settings-darwin-build-arm64
    just _require-cert
    xattr -cr "{{dist_settings}}/{{arch_darwin_arm64}}/{{app_name}}.app"
    codesign --deep --force --options runtime \
        --entitlements "{{entitlements}}" \
        --sign "${APPLE_DEVELOPER_CERTIFICATE_NAME}" \
        "{{dist_settings}}/{{arch_darwin_arm64}}/{{app_name}}.app"
    @echo ">>> Signed: {{dist_settings}}/{{arch_darwin_arm64}}/{{app_name}}.app"

settings-darwin-sign-x86_64: settings-darwin-build-x86_64
    just _require-cert
    xattr -cr "{{dist_settings}}/{{arch_darwin_x86}}/{{app_name}}.app"
    codesign --deep --force --options runtime \
        --entitlements "{{entitlements}}" \
        --sign "${APPLE_DEVELOPER_CERTIFICATE_NAME}" \
        "{{dist_settings}}/{{arch_darwin_x86}}/{{app_name}}.app"
    @echo ">>> Signed: {{dist_settings}}/{{arch_darwin_x86}}/{{app_name}}.app"

settings-darwin-notarize-arm64: settings-darwin-sign-arm64
    just _require-notarize-env
    just _require-dmgbuild
    dmgbuild -s "{{dmg_settings}}" \
        -D app="{{dist_settings}}/{{arch_darwin_arm64}}/{{app_name}}.app" \
        "{{app_name}}" \
        "{{dist_settings}}/{{app_name}}-v{{version}}-darwin-arm64.dmg"
    xcrun notarytool submit \
        "{{dist_settings}}/{{app_name}}-v{{version}}-darwin-arm64.dmg" \
        --apple-id "${APPLE_ID}" \
        --password "${APPLE_DEVELOPER_APP_PASSWORD}" \
        --team-id "${APPLE_DEVELOPER_TEAM_ID}" \
        --wait
    xcrun stapler staple \
        "{{dist_settings}}/{{app_name}}-v{{version}}-darwin-arm64.dmg"
    @echo ">>> Notarized: {{dist_settings}}/{{app_name}}-v{{version}}-darwin-arm64.dmg"

settings-darwin-notarize-x86_64: settings-darwin-sign-x86_64
    just _require-notarize-env
    just _require-dmgbuild
    dmgbuild -s "{{dmg_settings}}" \
        -D app="{{dist_settings}}/{{arch_darwin_x86}}/{{app_name}}.app" \
        "{{app_name}}" \
        "{{dist_settings}}/{{app_name}}-v{{version}}-darwin-x86_64.dmg"
    xcrun notarytool submit \
        "{{dist_settings}}/{{app_name}}-v{{version}}-darwin-x86_64.dmg" \
        --apple-id "${APPLE_ID}" \
        --password "${APPLE_DEVELOPER_APP_PASSWORD}" \
        --team-id "${APPLE_DEVELOPER_TEAM_ID}" \
        --wait
    xcrun stapler staple \
        "{{dist_settings}}/{{app_name}}-v{{version}}-darwin-x86_64.dmg"
    @echo ">>> Notarized: {{dist_settings}}/{{app_name}}-v{{version}}-darwin-x86_64.dmg"

# Makefile aliases (deprecated)
settings-arm64: settings-darwin-build-arm64
settings-x86_64: settings-darwin-build-x86_64
settings-win: settings-win-build
settings-linux: settings-linux-build
settings-zip-win: settings-win-zip
settings-zip-arm64: settings-darwin-zip-arm64
settings-zip-x86_64: settings-darwin-zip-x86_64
settings-dmg-arm64: settings-darwin-dmg-arm64
settings-dmg-x86_64: settings-darwin-dmg-x86_64
sign-settings-arm64: settings-darwin-sign-arm64
sign-settings-x86_64: settings-darwin-sign-x86_64
notarize-settings-arm64: settings-darwin-notarize-arm64
notarize-settings-x86_64: settings-darwin-notarize-x86_64
win: settings-win-build
win-zip: settings-win-zip

# ---------------------------------------------------------------------------
# settings-schema-checker → dist/settings-schema-checker/
# ---------------------------------------------------------------------------

checker-darwin-build: checker-darwin-build-arm64 checker-darwin-build-x86_64

checker-darwin-build-arm64:
    @echo ">>> Building {{checker_name}}  (arch: {{arch_darwin_arm64}})"
    cargo build --release -p settings-schema --bin {{checker_name}} --target {{rust_target_arm64}}
    mkdir -p "{{dist_checker}}/{{arch_darwin_arm64}}"
    cp "target/{{rust_target_arm64}}/release/{{checker_name}}" \
        "{{dist_checker}}/{{arch_darwin_arm64}}/{{checker_name}}"
    @echo ">>> Output: {{dist_checker}}/{{arch_darwin_arm64}}/{{checker_name}}"

checker-darwin-build-x86_64:
    @echo ">>> Building {{checker_name}}  (arch: {{arch_darwin_x86}})"
    cargo build --release -p settings-schema --bin {{checker_name}} --target {{rust_target_x86}}
    mkdir -p "{{dist_checker}}/{{arch_darwin_x86}}"
    cp "target/{{rust_target_x86}}/release/{{checker_name}}" \
        "{{dist_checker}}/{{arch_darwin_x86}}/{{checker_name}}"
    @echo ">>> Output: {{dist_checker}}/{{arch_darwin_x86}}/{{checker_name}}"

[macos]
checker-win-build:
    @echo ">>> Building {{checker_name}}  (arch: {{arch_win}})"
    CARGO_TARGET_DIR="{{win_target_dir}}" \
        cargo build --release -p settings-schema --bin {{checker_name}} --target {{win_target}}
    mkdir -p "{{dist_checker}}/{{arch_win}}"
    cp "{{win_target_dir}}/{{win_target}}/release/{{checker_name}}.exe" \
        "{{dist_checker}}/{{arch_win}}/{{checker_name}}.exe"
    @echo ">>> Output: {{dist_checker}}/{{arch_win}}/{{checker_name}}.exe"

[linux]
checker-win-build:
    @echo "Error: Windows cross-compile requires a macOS host" && exit 1

[windows]
checker-win-build:
    @echo ">>> Building {{checker_name}}  (arch: {{arch_win}})"
    cargo build --release -p settings-schema --bin {{checker_name}}
    mkdir -p "{{dist_checker}}/{{arch_win}}"
    cp "target/release/{{checker_name}}.exe" \
        "{{dist_checker}}/{{arch_win}}/{{checker_name}}.exe"
    @echo ">>> Output: {{dist_checker}}/{{arch_win}}/{{checker_name}}.exe"

checker-linux-build:
    @echo ">>> Building {{checker_name}}  (arch: {{arch_linux}})"
    cargo build --release -p settings-schema --bin {{checker_name}}
    mkdir -p "{{dist_checker}}/{{arch_linux}}"
    cp "target/release/{{checker_name}}" \
        "{{dist_checker}}/{{arch_linux}}/{{checker_name}}"
    @echo ">>> Output: {{dist_checker}}/{{arch_linux}}/{{checker_name}}"

_checker-darwin-zip-arm64:
    #!/usr/bin/env bash
    set -euo pipefail
    ZIP="{{dist_checker}}/{{checker_name}}-v{{version}}-darwin-arm64.zip"
    rm -f "${ZIP}"
    cd "{{dist_checker}}/{{arch_darwin_arm64}}" && \
        zip "../$(basename "${ZIP}")" "{{checker_name}}"
    echo ">>> Package: ${ZIP}"

_checker-darwin-zip-x86_64:
    #!/usr/bin/env bash
    set -euo pipefail
    ZIP="{{dist_checker}}/{{checker_name}}-v{{version}}-darwin-x86_64.zip"
    rm -f "${ZIP}"
    cd "{{dist_checker}}/{{arch_darwin_x86}}" && \
        zip "../$(basename "${ZIP}")" "{{checker_name}}"
    echo ">>> Package: ${ZIP}"

checker-darwin-zip-arm64: checker-darwin-build-arm64
    just _checker-darwin-zip-arm64

checker-darwin-zip-x86_64: checker-darwin-build-x86_64
    just _checker-darwin-zip-x86_64

checker-win-zip: checker-win-build
    #!/usr/bin/env bash
    set -euo pipefail
    ZIP="{{dist_checker}}/{{checker_name}}-v{{version}}-windows-x86_64.zip"
    rm -f "${ZIP}"
    cd "{{dist_checker}}/{{arch_win}}" && \
        zip "../$(basename "${ZIP}")" "{{checker_name}}.exe"
    echo ">>> Package: ${ZIP}"

checker-linux-zip: checker-linux-build
    #!/usr/bin/env bash
    set -euo pipefail
    ZIP="{{dist_checker}}/{{checker_name}}-v{{version}}-linux-x86_64.zip"
    rm -f "${ZIP}"
    cd "{{dist_checker}}/{{arch_linux}}" && \
        zip "../$(basename "${ZIP}")" "{{checker_name}}"
    echo ">>> Package: ${ZIP}"

checker-linux-deb: checker-linux-build
    #!/usr/bin/env bash
    set -euo pipefail
    command -v nfpm >/dev/null 2>&1 || \
        { echo "Error: nfpm not found. See packaging/checker-nfpm.yaml" >&2; exit 1; }
    VERSION="{{version}}" nfpm package \
        -f packaging/checker-nfpm.yaml \
        --packager deb \
        --target "{{dist_checker}}"
    DEB="{{dist_checker}}/{{checker_name}}-v{{version}}-linux-x86_64.deb"
    GENERATED="{{dist_checker}}/{{checker_name}}_{{version}}_amd64.deb"
    rm -f "${DEB}"
    mv "${GENERATED}" "${DEB}"
    echo ">>> Package: ${DEB}"

checker-linux-release: checker-linux-zip checker-linux-deb

checker-darwin-sign-arm64: checker-darwin-build-arm64
    just _require-cert
    codesign --force --options runtime \
        --sign "${APPLE_DEVELOPER_CERTIFICATE_NAME}" \
        "{{dist_checker}}/{{arch_darwin_arm64}}/{{checker_name}}"
    @echo ">>> Signed: {{dist_checker}}/{{arch_darwin_arm64}}/{{checker_name}}"

checker-darwin-sign-x86_64: checker-darwin-build-x86_64
    just _require-cert
    codesign --force --options runtime \
        --sign "${APPLE_DEVELOPER_CERTIFICATE_NAME}" \
        "{{dist_checker}}/{{arch_darwin_x86}}/{{checker_name}}"
    @echo ">>> Signed: {{dist_checker}}/{{arch_darwin_x86}}/{{checker_name}}"

checker-darwin-notarize-arm64: checker-darwin-sign-arm64
    #!/usr/bin/env bash
    set -euo pipefail
    just _require-notarize-env
    just _checker-darwin-zip-arm64
    ZIP="{{dist_checker}}/{{checker_name}}-v{{version}}-darwin-arm64.zip"
    xcrun notarytool submit "${ZIP}" \
        --apple-id "${APPLE_ID}" \
        --password "${APPLE_DEVELOPER_APP_PASSWORD}" \
        --team-id "${APPLE_DEVELOPER_TEAM_ID}" \
        --wait
    echo ">>> Notarized: ${ZIP}"

checker-darwin-notarize-x86_64: checker-darwin-sign-x86_64
    #!/usr/bin/env bash
    set -euo pipefail
    just _require-notarize-env
    just _checker-darwin-zip-x86_64
    ZIP="{{dist_checker}}/{{checker_name}}-v{{version}}-darwin-x86_64.zip"
    xcrun notarytool submit "${ZIP}" \
        --apple-id "${APPLE_ID}" \
        --password "${APPLE_DEVELOPER_APP_PASSWORD}" \
        --team-id "${APPLE_DEVELOPER_TEAM_ID}" \
        --wait
    echo ">>> Notarized: ${ZIP}"

# Makefile aliases (deprecated)
settings-schema-checker-arm64: checker-darwin-build-arm64
settings-schema-checker-x86_64: checker-darwin-build-x86_64
settings-schema-checker-win: checker-win-build
settings-schema-checker-linux: checker-linux-build
settings-schema-checker-zip-arm64: checker-darwin-zip-arm64
settings-schema-checker-zip-x86_64: checker-darwin-zip-x86_64
settings-schema-checker-zip-win: checker-win-zip
settings-schema-checker-zip-linux: checker-linux-zip
sign-settings-schema-checker-arm64: checker-darwin-sign-arm64
sign-settings-schema-checker-x86_64: checker-darwin-sign-x86_64
notarize-settings-schema-checker-arm64: checker-darwin-notarize-arm64
notarize-settings-schema-checker-x86_64: checker-darwin-notarize-x86_64

# ---------------------------------------------------------------------------
# Internal helpers (shared with demo/Justfile)
# ---------------------------------------------------------------------------

_abs-schema:
    #!/usr/bin/env bash
    set -euo pipefail
    if [ -n "${SETTINGS_SCHEMA:-}" ]; then
        printf '%s' "${SETTINGS_SCHEMA}"
    else
        realpath "${SCHEMA:-../schema.toml}" 2>/dev/null || \
            python3 -c "import os,sys; print(os.path.abspath(sys.argv[1]))" "${SCHEMA:-../schema.toml}"
    fi

_ensure-icon-font:
    #!/usr/bin/env bash
    test -f "{{icon_ttf}}" || just _download-icons

_ensure-appicon-ico:
    #!/usr/bin/env bash
    test -f "{{appicon_ico}}" || just _build-appicon-ico

_download-icons:
    #!/usr/bin/env bash
    set -euo pipefail
    ABS_SCHEMA="$(just _abs-schema)"
    ICON_STYLE="${ICON_STYLE:-$(awk -F'"' '/^icon_style/{print $2; exit}' "${ABS_SCHEMA}" 2>/dev/null | head -1)}"
    ICON_STYLE="${ICON_STYLE:-rounded}"
    case "${ICON_STYLE}" in
        outlined) ICON_VARIANT="Outlined" ;;
        sharp)    ICON_VARIANT="Sharp" ;;
        *)        ICON_VARIANT="Rounded" ;;
    esac
    mkdir -p "{{assets_dir}}"
    BASE="https://raw.githubusercontent.com/google/material-design-icons/master/variablefont"
    TTF="MaterialSymbols${ICON_VARIANT}%5BFILL%2CGRAD%2Copsz%2Cwght%5D.ttf"
    CP="MaterialSymbols${ICON_VARIANT}%5BFILL%2CGRAD%2Copsz%2Cwght%5D.codepoints"
    echo ">>> Downloading Material Symbols ${ICON_VARIANT}  (style: ${ICON_STYLE})"
    curl -fsSL "${BASE}/${TTF}" -o "{{icon_ttf}}"
    curl -fsSL "${BASE}/${CP}" -o "{{icon_cp}}"
    echo ">>> Icons saved to {{assets_dir}}/"

_build-appicon-ico:
    #!/usr/bin/env bash
    set -euo pipefail
    command -v magick >/dev/null 2>&1 || \
        { echo "Error: ImageMagick not found. Run: brew install imagemagick" >&2; exit 1; }
    magick assets/appicon.png -define icon:auto-resize=256,48,32,16 "{{appicon_ico}}"
    echo ">>> Generated: {{appicon_ico}}"

_settings-darwin-bundle arch_slug rust_target:
    #!/usr/bin/env bash
    set -euo pipefail
    ABS_SCHEMA="$(just _abs-schema)"
    APP="{{dist_settings}}/{{arch_slug}}/{{app_name}}.app"
    echo ">>> Building Settings .app  (arch: {{arch_slug}}, schema: ${ABS_SCHEMA})"
    MACOSX_DEPLOYMENT_TARGET="{{min_macos}}" SETTINGS_SCHEMA="${ABS_SCHEMA}" \
        cargo build --release -p settings --target {{rust_target}}
    mkdir -p "${APP}/Contents/MacOS" "${APP}/Contents/Resources"
    cp "target/{{rust_target}}/release/{{exe_name}}" "${APP}/Contents/MacOS/{{app_name}}"
    just _darwin-write-plist "${APP}/Contents" jp.emotiongraphics.settings
    just _darwin-icns-if-present "{{icon_src}}" \
        "${APP}/Contents/Resources/AppIcon.icns"

# Usage: just _darwin-write-plist path/to/Contents [plist_bundle_id]
_darwin-write-plist contents_dir plist_bundle_id="jp.emotiongraphics.settings":
    #!/usr/bin/env bash
    set -euo pipefail
    mkdir -p "{{contents_dir}}"
    cat > "{{contents_dir}}/Info.plist" <<'PLIST'
    <?xml version="1.0" encoding="UTF-8"?>
    <!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
    <plist version="1.0">
    <dict>
        <key>CFBundleName</key>              <string>Settings</string>
        <key>CFBundleIdentifier</key>        <string>BUNDLE_ID_PLACEHOLDER</string>
        <key>CFBundleExecutable</key>        <string>Settings</string>
        <key>CFBundleVersion</key>           <string>VERSION_PLACEHOLDER</string>
        <key>CFBundleShortVersionString</key><string>VERSION_PLACEHOLDER</string>
        <key>CFBundlePackageType</key>       <string>APPL</string>
        <key>CFBundleDevelopmentRegion</key> <string>en</string>
        <key>LSMinimumSystemVersion</key>    <string>MIN_MACOS_PLACEHOLDER</string>
        <key>NSHighResolutionCapable</key>   <true/>
        <key>NSHumanReadableCopyright</key>  <string>Copyright 2026 eMotionGraphics Inc.</string>
        <key>CFBundleIconFile</key>          <string>AppIcon</string>
        <key>LSUIElement</key>               <false/>
    </dict>
    </plist>
    PLIST
    sed -i '' \
        -e "s/BUNDLE_ID_PLACEHOLDER/{{plist_bundle_id}}/" \
        -e "s/VERSION_PLACEHOLDER/{{version}}/" \
        -e "s/MIN_MACOS_PLACEHOLDER/{{min_macos}}/" \
        "{{contents_dir}}/Info.plist"

# Usage: just _darwin-icns-if-present icon.png out.icns
_darwin-icns-if-present icon_src icns_out:
    #!/usr/bin/env bash
    if [ -f "{{icon_src}}" ]; then
        just _darwin-icns-build "{{icon_src}}" "{{icns_out}}"
    else
        echo "Note: {{icon_src}} not found — skipping icon generation."
    fi

# Usage: just _darwin-icns-build icon.png out.icns
_darwin-icns-build icon_src icns_out:
    #!/usr/bin/env bash
    set -euo pipefail
    ICONSET_WORK="$(mktemp -d)"
    ICONSET="${ICONSET_WORK}/AppIcon.iconset"
    mkdir -p "${ICONSET}"
    SRC_NORM="${ICONSET_WORK}/source-1024.png"
    sips -z 1024 1024 "{{icon_src}}" --out "${SRC_NORM}" >/dev/null
    sips --deleteColorManagementProperties "${SRC_NORM}" >/dev/null 2>&1 || true
    sips -z 16   16   "${SRC_NORM}" --out "${ICONSET}/icon_16x16.png"
    sips -z 32   32   "${SRC_NORM}" --out "${ICONSET}/icon_16x16@2x.png"
    sips -z 32   32   "${SRC_NORM}" --out "${ICONSET}/icon_32x32.png"
    sips -z 64   64   "${SRC_NORM}" --out "${ICONSET}/icon_32x32@2x.png"
    sips -z 128  128  "${SRC_NORM}" --out "${ICONSET}/icon_128x128.png"
    sips -z 256  256  "${SRC_NORM}" --out "${ICONSET}/icon_128x128@2x.png"
    sips -z 256  256  "${SRC_NORM}" --out "${ICONSET}/icon_256x256.png"
    sips -z 512  512  "${SRC_NORM}" --out "${ICONSET}/icon_256x256@2x.png"
    sips -z 512  512  "${SRC_NORM}" --out "${ICONSET}/icon_512x512.png"
    cp "${SRC_NORM}" "${ICONSET}/icon_512x512@2x.png"
    mkdir -p "$(dirname "{{icns_out}}")"
    ICNS_OUT_ABS="$(cd "$(dirname "{{icns_out}}")" && pwd)/$(basename "{{icns_out}}")"
    iconutil -c icns "${ICONSET}" -o "${ICNS_OUT_ABS}"
    rm -rf "${ICONSET_WORK}"

_require-dmgbuild:
    #!/usr/bin/env bash
    command -v dmgbuild >/dev/null 2>&1 || \
        { echo "Error: dmgbuild not found. Run: pipx install dmgbuild" >&2; exit 1; }

_require-cert:
    #!/usr/bin/env bash
    test -n "${APPLE_DEVELOPER_CERTIFICATE_NAME:-}" || \
        { echo "Error: APPLE_DEVELOPER_CERTIFICATE_NAME is not set" >&2; exit 1; }

_require-notarize-env:
    #!/usr/bin/env bash
    test -n "${APPLE_DEVELOPER_TEAM_ID:-}" || \
        { echo "Error: APPLE_DEVELOPER_TEAM_ID is not set" >&2; exit 1; }
    test -n "${APPLE_ID:-}" || \
        { echo "Error: APPLE_ID is not set" >&2; exit 1; }
    test -n "${APPLE_DEVELOPER_APP_PASSWORD:-}" || \
        { echo "Error: APPLE_DEVELOPER_APP_PASSWORD is not set" >&2; exit 1; }

clean:
    cargo clean
    rm -rf dist "{{win_target_dir}}"
