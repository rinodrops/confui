# ============================================================
# Settings — production build targets
# ============================================================
# DEPRECATED: use Justfile instead (`just --list`).
# This file remains during the migration period.
# Usage:
#   make binary            — release binary for parent-app bundling (current arch)
#   make check-schema      — validate SCHEMA without building the GUI
#   make settings-arm64    — GUI .app → dist/settings/darwin-arm64/
#   make settings-win      — GUI .exe → dist/settings/windows-x86_64/
#   make settings-schema-checker-arm64  — CLI → dist/settings-schema-checker/
#
#   make SCHEMA=/path/to/schema.toml [target]
#   make ICON_STYLE=outlined icons
#
# Standalone demo builds: see demo/Makefile.
#
# Windows cross-compile prerequisites (run once on the build host):
#   brew install mingw-w64
#   rustup target add x86_64-pc-windows-gnu
# ============================================================

empty     :=
space     := $(empty) $(empty)

APP_NAME       := Settings
EXE_NAME       := settings
CHECKER_NAME   := settings-schema-checker
VERSION        := $(shell awk -F'"' '/^version *=/{print $$2; exit}' Cargo.toml)
MIN_MACOS      := 11.0

ARCH_DARWIN_ARM64 := darwin-arm64
ARCH_DARWIN_X86   := darwin-x86_64
ARCH_WIN          := windows-x86_64
ARCH_LINUX        := linux-x86_64

RUST_TARGET_ARM64 := aarch64-apple-darwin
RUST_TARGET_X86   := x86_64-apple-darwin
WIN_TARGET        := x86_64-pc-windows-gnu
WIN_TARGET_DIR    := /tmp/settings-win

# Schema file to embed (production default: parent app's schema one level up).
# $(abspath $(SCHEMA)) splits SCHEMA on whitespace; resolve via the shell instead.
SCHEMA ?= ../schema.toml
ifneq ($(filter environment command line override,$(origin SETTINGS_SCHEMA)),)
ABS_SCHEMA := $(SETTINGS_SCHEMA)
else
ABS_SCHEMA := $(shell realpath '$(SCHEMA)' 2>/dev/null || python3 -c "import os,sys; print(os.path.abspath(sys.argv[1]))" '$(SCHEMA)')
endif

DIST_SETTINGS := dist/settings
DIST_CHECKER  := dist/settings-schema-checker

RELEASE_BIN := target/release/$(EXE_NAME)

APP_ARM64 := $(DIST_SETTINGS)/$(ARCH_DARWIN_ARM64)/$(APP_NAME).app
APP_X86   := $(DIST_SETTINGS)/$(ARCH_DARWIN_X86)/$(APP_NAME).app

SETTINGS_DMG_ARM64 := $(DIST_SETTINGS)/$(APP_NAME)-v$(VERSION)-darwin-arm64.dmg
SETTINGS_DMG_X86   := $(DIST_SETTINGS)/$(APP_NAME)-v$(VERSION)-darwin-x86_64.dmg
SETTINGS_ZIP_ARM64 := $(DIST_SETTINGS)/$(APP_NAME)-v$(VERSION)-darwin-arm64.zip
SETTINGS_ZIP_X86   := $(DIST_SETTINGS)/$(APP_NAME)-v$(VERSION)-darwin-x86_64.zip

WIN_EXE      := $(DIST_SETTINGS)/$(ARCH_WIN)/$(APP_NAME).exe
WIN_ZIP      := $(DIST_SETTINGS)/$(APP_NAME)-v$(VERSION)-windows-x86_64.zip
LINUX_BIN    := $(DIST_SETTINGS)/$(ARCH_LINUX)/$(EXE_NAME)

CHECKER_ARM64 := $(DIST_CHECKER)/$(ARCH_DARWIN_ARM64)/$(CHECKER_NAME)
CHECKER_X86   := $(DIST_CHECKER)/$(ARCH_DARWIN_X86)/$(CHECKER_NAME)
CHECKER_WIN   := $(DIST_CHECKER)/$(ARCH_WIN)/$(CHECKER_NAME).exe
CHECKER_LINUX := $(DIST_CHECKER)/$(ARCH_LINUX)/$(CHECKER_NAME)

CHECKER_ZIP_ARM64 := $(DIST_CHECKER)/$(CHECKER_NAME)-v$(VERSION)-darwin-arm64.zip
CHECKER_ZIP_X86   := $(DIST_CHECKER)/$(CHECKER_NAME)-v$(VERSION)-darwin-x86_64.zip
CHECKER_ZIP_WIN   := $(DIST_CHECKER)/$(CHECKER_NAME)-v$(VERSION)-windows-x86_64.zip
CHECKER_ZIP_LINUX := $(DIST_CHECKER)/$(CHECKER_NAME)-v$(VERSION)-linux-x86_64.zip

DMG_SETTINGS := demo/dmg_settings.py
ICON_SRC     := assets/appicon.png
ICONSET      := dist/AppIcon.iconset

CERT      := $(APPLE_DEVELOPER_CERTIFICATE_NAME)
TEAM_ID   := $(APPLE_DEVELOPER_TEAM_ID)
APPLE_ID_ := $(APPLE_ID)
APP_PASS  := $(APPLE_DEVELOPER_APP_PASSWORD)

APPICON_ICO := assets/appicon.ico
ASSETS_DIR  := assets
ICON_TTF    := $(ASSETS_DIR)/icons.ttf
ICON_CP     := $(ASSETS_DIR)/icons.codepoints

ICON_STYLE ?= $(or $(shell awk -F'"' '/^icon_style/{print $$2}' "$(ABS_SCHEMA)" 2>/dev/null | head -1),rounded)

ifeq ($(ICON_STYLE),outlined)
  ICON_VARIANT := Outlined
else ifeq ($(ICON_STYLE),sharp)
  ICON_VARIANT := Sharp
else
  ICON_VARIANT := Rounded
endif

ICON_URL_BASE := https://raw.githubusercontent.com/google/material-design-icons/master/variablefont
ICON_TTF_FILE := MaterialSymbols$(ICON_VARIANT)%5BFILL%2CGRAD%2Copsz%2Cwght%5D.ttf
ICON_CP_FILE  := MaterialSymbols$(ICON_VARIANT)%5BFILL%2CGRAD%2Copsz%2Cwght%5D.codepoints

.PHONY: all binary check-schema schema-check icons appicon-ico clean help \
	settings-arm64 settings-x86_64 settings-win settings-linux \
	settings-dmg-arm64 settings-dmg-x86_64 settings-zip-arm64 settings-zip-x86_64 \
	settings-zip-win sign-settings-arm64 sign-settings-x86_64 \
	notarize-settings-arm64 notarize-settings-x86_64 \
	settings-schema-checker-arm64 settings-schema-checker-x86_64 \
	settings-schema-checker-win settings-schema-checker-linux \
	settings-schema-checker-zip-arm64 settings-schema-checker-zip-x86_64 \
	settings-schema-checker-zip-win settings-schema-checker-zip-linux \
	sign-settings-schema-checker-arm64 sign-settings-schema-checker-x86_64 \
	notarize-settings-schema-checker-arm64 notarize-settings-schema-checker-x86_64 \
	_mac_settings_app _plist _icns_if_present _icns_build \
	win win-zip

## Default: release binary for parent-app bundling (current arch).
all: binary

$(ICON_TTF):
	@mkdir -p $(ASSETS_DIR)
	@echo ">>> Downloading Material Symbols $(ICON_VARIANT)  (style: $(ICON_STYLE))"
	curl -fsSL "$(ICON_URL_BASE)/$(ICON_TTF_FILE)" -o "$(ICON_TTF)"
	curl -fsSL "$(ICON_URL_BASE)/$(ICON_CP_FILE)"  -o "$(ICON_CP)"
	@echo ">>> Icons saved to $(ASSETS_DIR)/"

## Force-refresh icon font assets.
icons:
	@rm -f "$(ICON_TTF)" "$(ICON_CP)"
	$(MAKE) $(ICON_TTF)

$(APPICON_ICO): assets/appicon.png
	@command -v magick >/dev/null 2>&1 || \
		(echo "Error: ImageMagick not found. Run: brew install imagemagick" && exit 1)
	magick assets/appicon.png -define icon:auto-resize=256,48,32,16 "$(APPICON_ICO)"
	@echo ">>> Generated: $(APPICON_ICO)"

## Regenerate assets/appicon.ico from assets/appicon.png.
appicon-ico:
	@rm -f "$(APPICON_ICO)"
	$(MAKE) $(APPICON_ICO)

## Validate schema.toml without building the GUI binary.
check-schema:
	@echo ">>> Validating schema  (schema: $(ABS_SCHEMA))"
	cargo run --quiet -p settings-schema --bin $(CHECKER_NAME) -- "$(ABS_SCHEMA)"

schema-check: check-schema

## macOS / Linux release binary (current arch, for bundling into parent app).
binary: $(ICON_TTF)
	@echo ">>> Building release binary  (schema: $(ABS_SCHEMA))"
	SETTINGS_SCHEMA="$(ABS_SCHEMA)" cargo build --release -p settings
	@echo ">>> Output: $(RELEASE_BIN)"

# -----------------------------------------------------------------------
# Production GUI → dist/settings/
# -----------------------------------------------------------------------

settings-arm64: $(ICON_TTF)
	$(MAKE) _mac_settings_app PM_ARCH_SLUG=$(ARCH_DARWIN_ARM64) \
		PM_RUST_TARGET=$(RUST_TARGET_ARM64) \
		PM_SETTINGS_BIN=target/$(RUST_TARGET_ARM64)/release/$(EXE_NAME)
	@echo ">>> Output: $(APP_ARM64)"

settings-x86_64: $(ICON_TTF)
	$(MAKE) _mac_settings_app PM_ARCH_SLUG=$(ARCH_DARWIN_X86) \
		PM_RUST_TARGET=$(RUST_TARGET_X86) \
		PM_SETTINGS_BIN=target/$(RUST_TARGET_X86)/release/$(EXE_NAME)
	@echo ">>> Output: $(APP_X86)"

.PHONY: _mac_settings_app
_mac_settings_app:
	@test -n "$(PM_ARCH_SLUG)" && test -n "$(PM_RUST_TARGET)" && test -n "$(PM_SETTINGS_BIN)"
	@echo ">>> Building Settings .app  (arch: $(PM_ARCH_SLUG), schema: $(ABS_SCHEMA))"
	MACOSX_DEPLOYMENT_TARGET=$(MIN_MACOS) SETTINGS_SCHEMA="$(ABS_SCHEMA)" \
		cargo build --release -p settings --target $(PM_RUST_TARGET)
	@mkdir -p "$(DIST_SETTINGS)/$(PM_ARCH_SLUG)/$(APP_NAME).app/Contents/MacOS"
	@mkdir -p "$(DIST_SETTINGS)/$(PM_ARCH_SLUG)/$(APP_NAME).app/Contents/Resources"
	cp "$(PM_SETTINGS_BIN)" \
		"$(DIST_SETTINGS)/$(PM_ARCH_SLUG)/$(APP_NAME).app/Contents/MacOS/$(APP_NAME)"
	$(MAKE) _plist PLIST_CONTENTS="$(DIST_SETTINGS)/$(PM_ARCH_SLUG)/$(APP_NAME).app/Contents"
	$(MAKE) _icns_if_present \
		ICNS_RES_DIR="$(DIST_SETTINGS)/$(PM_ARCH_SLUG)/$(APP_NAME).app/Contents/Resources" \
		ICNS_OUT="$(DIST_SETTINGS)/$(PM_ARCH_SLUG)/$(APP_NAME).app/Contents/Resources/AppIcon.icns"

settings-win: $(ICON_TTF) $(APPICON_ICO)
	@echo ">>> Building Settings .exe  (schema: $(ABS_SCHEMA))"
	SETTINGS_SCHEMA="$(ABS_SCHEMA)" CARGO_TARGET_DIR="$(WIN_TARGET_DIR)" \
		cargo build --release -p settings --target $(WIN_TARGET)
	@mkdir -p "$(DIST_SETTINGS)/$(ARCH_WIN)"
	@cp "$(WIN_TARGET_DIR)/$(WIN_TARGET)/release/settings.exe" "$(WIN_EXE)"
	@echo ">>> Output: $(WIN_EXE)"

settings-linux: $(ICON_TTF)
	@echo ">>> Building Settings binary  (schema: $(ABS_SCHEMA))"
	SETTINGS_SCHEMA="$(ABS_SCHEMA)" cargo build --release -p settings
	@mkdir -p "$(DIST_SETTINGS)/$(ARCH_LINUX)"
	@cp "$(RELEASE_BIN)" "$(LINUX_BIN)"
	@echo ">>> Output: $(LINUX_BIN)"

settings-zip-win: settings-win
	@rm -f "$(WIN_ZIP)"
	cd "$(DIST_SETTINGS)/$(ARCH_WIN)" && \
		zip "../../$(notdir $(WIN_ZIP))" "$(APP_NAME).exe"
	@echo ">>> Package: $(WIN_ZIP)"

settings-zip-arm64: settings-arm64
	ditto -c -k --keepParent "$(APP_ARM64)" "$(SETTINGS_ZIP_ARM64)"
	@echo ">>> Package: $(SETTINGS_ZIP_ARM64)"

settings-zip-x86_64: settings-x86_64
	ditto -c -k --keepParent "$(APP_X86)" "$(SETTINGS_ZIP_X86)"
	@echo ">>> Package: $(SETTINGS_ZIP_X86)"

settings-dmg-arm64: settings-arm64
	@command -v dmgbuild >/dev/null 2>&1 || (echo "Error: dmgbuild not found. Run: pipx install dmgbuild" && exit 1)
	dmgbuild -s "$(DMG_SETTINGS)" -D app="$(APP_ARM64)" "$(APP_NAME)" "$(SETTINGS_DMG_ARM64)"
	@echo ">>> Package: $(SETTINGS_DMG_ARM64)"

settings-dmg-x86_64: settings-x86_64
	@command -v dmgbuild >/dev/null 2>&1 || (echo "Error: dmgbuild not found. Run: pipx install dmgbuild" && exit 1)
	dmgbuild -s "$(DMG_SETTINGS)" -D app="$(APP_X86)" "$(APP_NAME)" "$(SETTINGS_DMG_X86)"
	@echo ">>> Package: $(SETTINGS_DMG_X86)"

sign-settings-arm64: settings-arm64
	@test -n "$(CERT)" || (echo "Error: APPLE_DEVELOPER_CERTIFICATE_NAME is not set" && exit 1)
	xattr -cr "$(APP_ARM64)"
	codesign --deep --force --options runtime \
		--entitlements demo/entitlements.plist \
		--sign "$(CERT)" \
		"$(APP_ARM64)"
	@echo ">>> Signed: $(APP_ARM64)"

sign-settings-x86_64: settings-x86_64
	@test -n "$(CERT)" || (echo "Error: APPLE_DEVELOPER_CERTIFICATE_NAME is not set" && exit 1)
	xattr -cr "$(APP_X86)"
	codesign --deep --force --options runtime \
		--entitlements demo/entitlements.plist \
		--sign "$(CERT)" \
		"$(APP_X86)"
	@echo ">>> Signed: $(APP_X86)"

notarize-settings-arm64: sign-settings-arm64
	@test -n "$(TEAM_ID)"   || (echo "Error: APPLE_DEVELOPER_TEAM_ID is not set"      && exit 1)
	@test -n "$(APPLE_ID_)" || (echo "Error: APPLE_ID is not set"                     && exit 1)
	@test -n "$(APP_PASS)"  || (echo "Error: APPLE_DEVELOPER_APP_PASSWORD is not set" && exit 1)
	@command -v dmgbuild >/dev/null 2>&1 || (echo "Error: dmgbuild not found. Run: pipx install dmgbuild" && exit 1)
	dmgbuild -s "$(DMG_SETTINGS)" -D app="$(APP_ARM64)" "$(APP_NAME)" "$(SETTINGS_DMG_ARM64)"
	xcrun notarytool submit "$(SETTINGS_DMG_ARM64)" \
		--apple-id  "$(APPLE_ID_)" \
		--password  "$(APP_PASS)" \
		--team-id   "$(TEAM_ID)" \
		--wait
	xcrun stapler staple "$(SETTINGS_DMG_ARM64)"
	@echo ">>> Notarized and stapled: $(SETTINGS_DMG_ARM64)"

notarize-settings-x86_64: sign-settings-x86_64
	@test -n "$(TEAM_ID)"   || (echo "Error: APPLE_DEVELOPER_TEAM_ID is not set"      && exit 1)
	@test -n "$(APPLE_ID_)" || (echo "Error: APPLE_ID is not set"                     && exit 1)
	@test -n "$(APP_PASS)"  || (echo "Error: APPLE_DEVELOPER_APP_PASSWORD is not set" && exit 1)
	@command -v dmgbuild >/dev/null 2>&1 || (echo "Error: dmgbuild not found. Run: pipx install dmgbuild" && exit 1)
	dmgbuild -s "$(DMG_SETTINGS)" -D app="$(APP_X86)" "$(APP_NAME)" "$(SETTINGS_DMG_X86)"
	xcrun notarytool submit "$(SETTINGS_DMG_X86)" \
		--apple-id  "$(APPLE_ID_)" \
		--password  "$(APP_PASS)" \
		--team-id   "$(TEAM_ID)" \
		--wait
	xcrun stapler staple "$(SETTINGS_DMG_X86)"
	@echo ">>> Notarized and stapled: $(SETTINGS_DMG_X86)"

# Backward-compatible aliases.
win: settings-win
win-zip: settings-zip-win

# -----------------------------------------------------------------------
# settings-schema-checker → dist/settings-schema-checker/
# -----------------------------------------------------------------------

settings-schema-checker-arm64:
	@echo ">>> Building $(CHECKER_NAME)  (arch: $(ARCH_DARWIN_ARM64))"
	cargo build --release -p settings-schema --bin $(CHECKER_NAME) --target $(RUST_TARGET_ARM64)
	@mkdir -p "$(DIST_CHECKER)/$(ARCH_DARWIN_ARM64)"
	cp target/$(RUST_TARGET_ARM64)/release/$(CHECKER_NAME) "$(CHECKER_ARM64)"
	@echo ">>> Output: $(CHECKER_ARM64)"

settings-schema-checker-x86_64:
	@echo ">>> Building $(CHECKER_NAME)  (arch: $(ARCH_DARWIN_X86))"
	cargo build --release -p settings-schema --bin $(CHECKER_NAME) --target $(RUST_TARGET_X86)
	@mkdir -p "$(DIST_CHECKER)/$(ARCH_DARWIN_X86)"
	cp target/$(RUST_TARGET_X86)/release/$(CHECKER_NAME) "$(CHECKER_X86)"
	@echo ">>> Output: $(CHECKER_X86)"

settings-schema-checker-win:
	@echo ">>> Building $(CHECKER_NAME)  (arch: $(ARCH_WIN))"
	CARGO_TARGET_DIR="$(WIN_TARGET_DIR)" \
		cargo build --release -p settings-schema --bin $(CHECKER_NAME) --target $(WIN_TARGET)
	@mkdir -p "$(DIST_CHECKER)/$(ARCH_WIN)"
	cp "$(WIN_TARGET_DIR)/$(WIN_TARGET)/release/$(CHECKER_NAME).exe" "$(CHECKER_WIN)"
	@echo ">>> Output: $(CHECKER_WIN)"

settings-schema-checker-linux:
	@echo ">>> Building $(CHECKER_NAME)  (arch: $(ARCH_LINUX))"
	cargo build --release -p settings-schema --bin $(CHECKER_NAME)
	@mkdir -p "$(DIST_CHECKER)/$(ARCH_LINUX)"
	cp target/release/$(CHECKER_NAME) "$(CHECKER_LINUX)"
	@echo ">>> Output: $(CHECKER_LINUX)"

settings-schema-checker-zip-arm64: settings-schema-checker-arm64
	@rm -f "$(CHECKER_ZIP_ARM64)"
	cd "$(DIST_CHECKER)/$(ARCH_DARWIN_ARM64)" && \
		zip "../../$(notdir $(CHECKER_ZIP_ARM64))" "$(CHECKER_NAME)"
	@echo ">>> Package: $(CHECKER_ZIP_ARM64)"

settings-schema-checker-zip-x86_64: settings-schema-checker-x86_64
	@rm -f "$(CHECKER_ZIP_X86)"
	cd "$(DIST_CHECKER)/$(ARCH_DARWIN_X86)" && \
		zip "../../$(notdir $(CHECKER_ZIP_X86))" "$(CHECKER_NAME)"
	@echo ">>> Package: $(CHECKER_ZIP_X86)"

settings-schema-checker-zip-win: settings-schema-checker-win
	@rm -f "$(CHECKER_ZIP_WIN)"
	cd "$(DIST_CHECKER)/$(ARCH_WIN)" && \
		zip "../../$(notdir $(CHECKER_ZIP_WIN))" "$(CHECKER_NAME).exe"
	@echo ">>> Package: $(CHECKER_ZIP_WIN)"

settings-schema-checker-zip-linux: settings-schema-checker-linux
	@rm -f "$(CHECKER_ZIP_LINUX)"
	cd "$(DIST_CHECKER)/$(ARCH_LINUX)" && \
		zip "../../$(notdir $(CHECKER_ZIP_LINUX))" "$(CHECKER_NAME)"
	@echo ">>> Package: $(CHECKER_ZIP_LINUX)"

sign-settings-schema-checker-arm64: settings-schema-checker-arm64
	@test -n "$(CERT)" || (echo "Error: APPLE_DEVELOPER_CERTIFICATE_NAME is not set" && exit 1)
	codesign --force --options runtime --sign "$(CERT)" "$(CHECKER_ARM64)"
	@echo ">>> Signed: $(CHECKER_ARM64)"

sign-settings-schema-checker-x86_64: settings-schema-checker-x86_64
	@test -n "$(CERT)" || (echo "Error: APPLE_DEVELOPER_CERTIFICATE_NAME is not set" && exit 1)
	codesign --force --options runtime --sign "$(CERT)" "$(CHECKER_X86)"
	@echo ">>> Signed: $(CHECKER_X86)"

notarize-settings-schema-checker-arm64: sign-settings-schema-checker-arm64
	@test -n "$(TEAM_ID)"   || (echo "Error: APPLE_DEVELOPER_TEAM_ID is not set"      && exit 1)
	@test -n "$(APPLE_ID_)" || (echo "Error: APPLE_ID is not set"                     && exit 1)
	@test -n "$(APP_PASS)"  || (echo "Error: APPLE_DEVELOPER_APP_PASSWORD is not set" && exit 1)
	$(MAKE) settings-schema-checker-zip-arm64
	xcrun notarytool submit "$(CHECKER_ZIP_ARM64)" \
		--apple-id  "$(APPLE_ID_)" \
		--password  "$(APP_PASS)" \
		--team-id   "$(TEAM_ID)" \
		--wait
	xcrun stapler staple "$(CHECKER_ZIP_ARM64)"
	@echo ">>> Notarized and stapled: $(CHECKER_ZIP_ARM64)"

notarize-settings-schema-checker-x86_64: sign-settings-schema-checker-x86_64
	@test -n "$(TEAM_ID)"   || (echo "Error: APPLE_DEVELOPER_TEAM_ID is not set"      && exit 1)
	@test -n "$(APPLE_ID_)" || (echo "Error: APPLE_ID is not set"                     && exit 1)
	@test -n "$(APP_PASS)"  || (echo "Error: APPLE_DEVELOPER_APP_PASSWORD is not set" && exit 1)
	$(MAKE) settings-schema-checker-zip-x86_64
	xcrun notarytool submit "$(CHECKER_ZIP_X86)" \
		--apple-id  "$(APPLE_ID_)" \
		--password  "$(APP_PASS)" \
		--team-id   "$(TEAM_ID)" \
		--wait
	xcrun stapler staple "$(CHECKER_ZIP_X86)"
	@echo ">>> Notarized and stapled: $(CHECKER_ZIP_X86)"

# -----------------------------------------------------------------------
# Info.plist / ICNS helpers
# -----------------------------------------------------------------------

PLIST_CONTENTS ?=

.PHONY: _plist
_plist:
	@test -n "$(PLIST_CONTENTS)"
	@printf '%s\n' \
		'<?xml version="1.0" encoding="UTF-8"?>' \
		'<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">' \
		'<plist version="1.0">' \
		'<dict>' \
		'	<key>CFBundleName</key>              <string>$(APP_NAME)</string>' \
		'	<key>CFBundleIdentifier</key>        <string>jp.emotiongraphics.settings</string>' \
		'	<key>CFBundleExecutable</key>        <string>$(APP_NAME)</string>' \
		'	<key>CFBundleVersion</key>           <string>$(VERSION)</string>' \
		'	<key>CFBundleShortVersionString</key><string>$(VERSION)</string>' \
		'	<key>CFBundlePackageType</key>       <string>APPL</string>' \
		'	<key>CFBundleDevelopmentRegion</key> <string>en</string>' \
		'	<key>LSMinimumSystemVersion</key>    <string>$(MIN_MACOS)</string>' \
		'	<key>NSHighResolutionCapable</key>   <true/>' \
		'	<key>NSHumanReadableCopyright</key>  <string>Copyright 2026 eMotionGraphics Inc.</string>' \
		'	<key>CFBundleIconFile</key>          <string>AppIcon</string>' \
		'	<key>LSUIElement</key>               <false/>' \
		'</dict>' \
		'</plist>' \
		> "$(PLIST_CONTENTS)/Info.plist"

ICNS_RES_DIR ?=
ICNS_OUT       ?=

.PHONY: _icns_if_present
_icns_if_present:
	@test -n "$(ICNS_RES_DIR)" && test -n "$(ICNS_OUT)"
	@if [ -f "$(ICON_SRC)" ]; then \
		$(MAKE) _icns_build ICNS_RES_DIR="$(ICNS_RES_DIR)" ICNS_OUT="$(ICNS_OUT)"; \
	else \
		echo "Note: $(ICON_SRC) not found — skipping icon generation."; \
	fi

.PHONY: _icns_build
_icns_build:
	@test -n "$(ICNS_RES_DIR)" && test -n "$(ICNS_OUT)"
	mkdir -p "$(ICONSET)"
	sips -z 16    16    "$(ICON_SRC)" --out "$(ICONSET)/icon_16x16.png"      >/dev/null
	sips -z 32    32    "$(ICON_SRC)" --out "$(ICONSET)/icon_16x16@2x.png"   >/dev/null
	sips -z 32    32    "$(ICON_SRC)" --out "$(ICONSET)/icon_32x32.png"      >/dev/null
	sips -z 64    64    "$(ICON_SRC)" --out "$(ICONSET)/icon_32x32@2x.png"   >/dev/null
	sips -z 128   128   "$(ICON_SRC)" --out "$(ICONSET)/icon_128x128.png"    >/dev/null
	sips -z 256   256   "$(ICON_SRC)" --out "$(ICONSET)/icon_128x128@2x.png" >/dev/null
	sips -z 256   256   "$(ICON_SRC)" --out "$(ICONSET)/icon_256x256.png"    >/dev/null
	sips -z 512   512   "$(ICON_SRC)" --out "$(ICONSET)/icon_256x256@2x.png" >/dev/null
	sips -z 512   512   "$(ICON_SRC)" --out "$(ICONSET)/icon_512x512.png"    >/dev/null
	sips -z 1024  1024  "$(ICON_SRC)" --out "$(ICONSET)/icon_512x512@2x.png" >/dev/null
	iconutil -c icns "$(ICONSET)" -o "$(ICNS_OUT)"
	rm -rf "$(ICONSET)"

## Remove build artefacts.
clean:
	cargo clean
	rm -rf dist $(WIN_TARGET_DIR)

## Show this help.
help:
	@grep -E '^##' Makefile | sed 's/^## //'
