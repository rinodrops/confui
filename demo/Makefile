# ============================================================
# ConfUI — demo build targets
# ============================================================
# Generates standalone demo binaries for all platforms, embedding
# ../../schema.toml (the project's test schema) as the schema.
#
# Usage:
#   make               — macOS .app bundle (universal binary)
#   make win           — Windows .exe (cross-compiled from macOS)
#   make linux         — Linux binary (run natively on a Linux host)
#   make win-zip       — zip the Windows .exe
#   make mac-zip       — zip the macOS .app
#   make mac-dmg       — create a distributable DMG
#   make sign          — code-sign the .app bundle
#   make notarize      — sign, create DMG, notarize, and staple
#
# Prerequisites:
#   macOS universal:  rustup target add aarch64-apple-darwin x86_64-apple-darwin
#   Windows cross:    brew install mingw-w64
#                     rustup target add x86_64-pc-windows-gnu
#   ICO generation:   brew install imagemagick
#   DMG:              pipx install dmgbuild
#
# Environment variables required for signing / notarization:
#   APPLE_DEVELOPER_CERTIFICATE_NAME   — e.g. "Developer ID Application: ..."
#   APPLE_DEVELOPER_TEAM_ID            — 10-char team ID
#   APPLE_ID                           — Apple ID email
#   APPLE_DEVELOPER_APP_PASSWORD       — app-specific password
# ============================================================

empty     :=
space     := $(empty) $(empty)

APP_NAME  := ConfUI
EXE_NAME  := confui
BUNDLE_ID := jp.emotiongraphics.confui
VERSION   := $(shell awk -F'"' '/^version *=/{print $$2; exit}' ../Cargo.toml)
MIN_MACOS := 11.0

# Schema: the demo test schema, co-located with this Makefile in demo/.
SCHEMA := $(abspath schema.toml)

# Cargo invocation referencing the parent confui/ package.
CARGO_MANIFEST := --manifest-path ../Cargo.toml

DIST      := dist
BUNDLE    := $(DIST)/$(APP_NAME).app
CONTENTS  := $(BUNDLE)/Contents
MACOS_DIR := $(CONTENTS)/MacOS
RES_DIR   := $(CONTENTS)/Resources

# Make-target–safe versions: spaces escaped as '\ ' for use in
# prerequisite lists and target definitions.
BUNDLE_T    := $(subst $(space),\ ,$(BUNDLE))
CONTENTS_T  := $(subst $(space),\ ,$(CONTENTS))
MACOS_DIR_T := $(subst $(space),\ ,$(MACOS_DIR))
RES_DIR_T   := $(subst $(space),\ ,$(RES_DIR))

ICON_SRC_MACOS := assets/appicon-macos.png
ICONSET        := $(DIST)/AppIcon.iconset
ICNS           := $(RES_DIR)/AppIcon.icns

CERT      := $(APPLE_DEVELOPER_CERTIFICATE_NAME)
TEAM_ID   := $(APPLE_DEVELOPER_TEAM_ID)
APPLE_ID_ := $(APPLE_ID)
APP_PASS  := $(APPLE_DEVELOPER_APP_PASSWORD)

APP_ZIP      := $(DIST)/$(APP_NAME)-v$(VERSION)-darwin-universal.zip
APP_DMG      := $(DIST)/$(APP_NAME)-v$(VERSION)-darwin-universal.dmg
DMG_SETTINGS := dmg_settings.py

WIN_TARGET     := x86_64-pc-windows-gnu
WIN_TARGET_DIR := /tmp/confui-demo-win
WIN_DIR        := $(DIST)/confui-windows
WIN_EXE        := $(WIN_DIR)/$(APP_NAME).exe
WIN_ZIP        := $(DIST)/$(APP_NAME)-v$(VERSION)-windows-x86_64.zip

LINUX_BIN := $(DIST)/$(EXE_NAME)

# Icon font — delegate to parent Makefile if not yet downloaded.
ICON_TTF := ../assets/icons.ttf

# Windows .ico — delegate to parent Makefile if not yet generated.
APPICON_ICO := ../assets/appicon.ico

# ── targets ─────────────────────────────────────────────────

.PHONY: all mac win linux win-zip mac-zip mac-dmg sign notarize clean help

## Default: build the macOS .app bundle (universal binary).
all: mac

# ── Icon font ────────────────────────────────────────────────
# Delegate to parent Makefile when the font file is absent.

../assets/icons.ttf:
	$(MAKE) -C .. assets/icons.ttf

# ── Windows .ico ─────────────────────────────────────────────
# Delegate to parent Makefile when the .ico has not been generated.

../assets/appicon.ico: ../assets/appicon.png
	$(MAKE) -C .. assets/appicon.ico

# ── macOS demo .app ──────────────────────────────────────────

## macOS universal .app bundle (demo).
##   Requires: rustup target add aarch64-apple-darwin x86_64-apple-darwin
mac: $(ICON_TTF) | $(MACOS_DIR_T) $(RES_DIR_T)
	@echo ">>> Building demo macOS .app  (schema: $(SCHEMA))"
	MACOSX_DEPLOYMENT_TARGET=$(MIN_MACOS) CONFUI_SCHEMA=$(SCHEMA) \
		cargo build $(CARGO_MANIFEST) --release --target aarch64-apple-darwin
	MACOSX_DEPLOYMENT_TARGET=$(MIN_MACOS) CONFUI_SCHEMA=$(SCHEMA) \
		cargo build $(CARGO_MANIFEST) --release --target x86_64-apple-darwin
	lipo -create -output "$(MACOS_DIR)/$(APP_NAME)" \
		../target/aarch64-apple-darwin/release/$(EXE_NAME) \
		../target/x86_64-apple-darwin/release/$(EXE_NAME)
	$(MAKE) _plist _icns_if_present
	@echo ">>> Done: $(BUNDLE)"
	@echo "    Run:  open $(BUNDLE)"

# ── Info.plist ───────────────────────────────────────────────
# Always re-generated so version changes in Cargo.toml propagate automatically.

.PHONY: _plist
_plist: | $(CONTENTS_T)
	@printf '%s\n' \
		'<?xml version="1.0" encoding="UTF-8"?>' \
		'<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">' \
		'<plist version="1.0">' \
		'<dict>' \
		'	<key>CFBundleName</key>              <string>$(APP_NAME)</string>' \
		'	<key>CFBundleIdentifier</key>        <string>$(BUNDLE_ID)</string>' \
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
		> "$(CONTENTS)/Info.plist"

# ── App icon ─────────────────────────────────────────────────
# Skip gracefully when assets/appicon-macos.png does not exist.

.PHONY: _icns_if_present
_icns_if_present: | $(RES_DIR_T)
	@if [ -f "$(ICON_SRC_MACOS)" ]; then \
		$(MAKE) _icns_build; \
	else \
		echo "Note: $(ICON_SRC_MACOS) not found — skipping icon generation."; \
	fi

.PHONY: _icns_build
_icns_build: | $(RES_DIR_T)
	mkdir -p "$(ICONSET)"
	sips -z 16    16    "$(ICON_SRC_MACOS)" --out "$(ICONSET)/icon_16x16.png"      >/dev/null
	sips -z 32    32    "$(ICON_SRC_MACOS)" --out "$(ICONSET)/icon_16x16@2x.png"   >/dev/null
	sips -z 32    32    "$(ICON_SRC_MACOS)" --out "$(ICONSET)/icon_32x32.png"      >/dev/null
	sips -z 64    64    "$(ICON_SRC_MACOS)" --out "$(ICONSET)/icon_32x32@2x.png"   >/dev/null
	sips -z 128   128   "$(ICON_SRC_MACOS)" --out "$(ICONSET)/icon_128x128.png"    >/dev/null
	sips -z 256   256   "$(ICON_SRC_MACOS)" --out "$(ICONSET)/icon_128x128@2x.png" >/dev/null
	sips -z 256   256   "$(ICON_SRC_MACOS)" --out "$(ICONSET)/icon_256x256.png"    >/dev/null
	sips -z 512   512   "$(ICON_SRC_MACOS)" --out "$(ICONSET)/icon_256x256@2x.png" >/dev/null
	sips -z 512   512   "$(ICON_SRC_MACOS)" --out "$(ICONSET)/icon_512x512.png"    >/dev/null
	sips -z 1024  1024  "$(ICON_SRC_MACOS)" --out "$(ICONSET)/icon_512x512@2x.png" >/dev/null
	iconutil -c icns "$(ICONSET)" -o "$(ICNS)"
	rm -rf "$(ICONSET)"

# ── Windows demo .exe ────────────────────────────────────────

## Windows .exe (cross-compiled from macOS via mingw-w64, demo).
##   Prerequisites: brew install mingw-w64
##                  rustup target add x86_64-pc-windows-gnu
win: $(ICON_TTF) $(APPICON_ICO)
	@echo ">>> Building demo Windows .exe  (schema: $(SCHEMA))"
	CONFUI_SCHEMA=$(SCHEMA) CARGO_TARGET_DIR="$(WIN_TARGET_DIR)" \
		cargo build $(CARGO_MANIFEST) --release --target $(WIN_TARGET)
	@mkdir -p "$(WIN_DIR)"
	@cp "$(WIN_TARGET_DIR)/$(WIN_TARGET)/release/confui.exe" "$(WIN_EXE)"
	@echo ">>> Output: $(WIN_EXE)"

## Package the Windows .exe into a distributable zip.
win-zip: win
	@cd "$(DIST)" && zip -j "$(notdir $(WIN_ZIP))" "confui-windows/$(APP_NAME).exe"
	@echo ">>> Package: $(WIN_ZIP)"

# ── Linux demo binary ────────────────────────────────────────

## Linux release binary (demo, run natively on a Linux host).
linux: $(ICON_TTF)
	@echo ">>> Building demo Linux binary  (schema: $(SCHEMA))"
	CONFUI_SCHEMA=$(SCHEMA) cargo build $(CARGO_MANIFEST) --release
	@mkdir -p "$(DIST)"
	@cp ../target/release/$(EXE_NAME) "$(LINUX_BIN)"
	@echo ">>> Output: $(LINUX_BIN)"

# ── Distribution (macOS) ─────────────────────────────────────

## Create a distributable zip of the macOS .app bundle.
mac-zip: mac
	ditto -c -k --keepParent "$(BUNDLE)" "$(APP_ZIP)"
	@echo ">>> Package: $(APP_ZIP)"

## Create a distributable DMG.
mac-dmg: mac
	@command -v dmgbuild >/dev/null 2>&1 || (echo "Error: dmgbuild not found. Run: pipx install dmgbuild" && exit 1)
	dmgbuild -s "$(DMG_SETTINGS)" -D app="$(BUNDLE)" "$(APP_NAME)" "$(APP_DMG)"
	@echo ">>> Package: $(APP_DMG)"

# ── Code signing ─────────────────────────────────────────────

## Code-sign the .app bundle with hardened runtime.
sign: mac
	@test -n "$(CERT)" || (echo "Error: APPLE_DEVELOPER_CERTIFICATE_NAME is not set" && exit 1)
	xattr -cr "$(BUNDLE)"
	codesign --deep --force --options runtime \
		--entitlements entitlements.plist \
		--sign "$(CERT)" \
		"$(BUNDLE)"
	@echo ">>> Signed: $(BUNDLE)"

# ── Notarization ─────────────────────────────────────────────

## Sign the .app, create a DMG, submit for notarization, and staple.
notarize: sign
	@test -n "$(TEAM_ID)"   || (echo "Error: APPLE_DEVELOPER_TEAM_ID is not set"      && exit 1)
	@test -n "$(APPLE_ID_)" || (echo "Error: APPLE_ID is not set"                     && exit 1)
	@test -n "$(APP_PASS)"  || (echo "Error: APPLE_DEVELOPER_APP_PASSWORD is not set" && exit 1)
	@command -v dmgbuild >/dev/null 2>&1 || (echo "Error: dmgbuild not found. Run: pipx install dmgbuild" && exit 1)
	dmgbuild -s "$(DMG_SETTINGS)" -D app="$(BUNDLE)" "$(APP_NAME)" "$(APP_DMG)"
	xcrun notarytool submit "$(APP_DMG)" \
		--apple-id  "$(APPLE_ID_)" \
		--password  "$(APP_PASS)" \
		--team-id   "$(TEAM_ID)" \
		--wait
	xcrun stapler staple "$(APP_DMG)"
	@echo ">>> Notarized and stapled: $(APP_DMG)"

# ── Directory scaffolding ────────────────────────────────────

$(DIST):
	mkdir -p "$(DIST)"

$(CONTENTS_T): | $(DIST)
	mkdir -p "$(CONTENTS)"

$(MACOS_DIR_T): | $(CONTENTS_T)
	mkdir -p "$(MACOS_DIR)"

$(RES_DIR_T): | $(CONTENTS_T)
	mkdir -p "$(RES_DIR)"

# ── Maintenance ──────────────────────────────────────────────

## Remove demo build artefacts (does not touch confui/target/).
clean:
	rm -rf $(DIST) $(WIN_TARGET_DIR)

## Show this help.
help:
	@grep -E '^##' Makefile | sed 's/^## //'
