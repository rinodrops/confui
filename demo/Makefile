# ============================================================
# Settings — demo build targets
# ============================================================
# DEPRECATED: use Justfile instead (`just --list`).
# This file remains during the migration period.
# Standalone demo GUI (demo/schema.toml embedded). No checker.
#
# Usage:
#   make demo-arm64      — macOS .app → dist/darwin-arm64/
#   make demo-x86_64     — macOS .app (Intel)
#   make demo-win        — Windows .exe + config.toml
#   make demo-linux      — Linux binary + config.toml
#   make demo-zip-linux  — Linux zip (settings + config.toml)
#   make demo-dmg-arm64  — distributable DMG
#
# Prerequisites:
#   macOS:  rustup target add aarch64-apple-darwin x86_64-apple-darwin
#   Windows cross: brew install mingw-w64; rustup target add x86_64-pc-windows-gnu
#   ICO:    brew install imagemagick
#   DMG:    pipx install dmgbuild
# ============================================================

empty     :=
space     := $(empty) $(empty)

APP_NAME  := Settings
EXE_NAME  := settings
BUNDLE_ID := jp.emotiongraphics.settings
VERSION   := $(shell awk -F'"' '/^version *=/{print $$2; exit}' ../Cargo.toml)
MIN_MACOS := 11.0

ARCH_DARWIN_ARM64 := darwin-arm64
ARCH_DARWIN_X86   := darwin-x86_64
ARCH_WIN          := windows-x86_64
ARCH_LINUX        := linux-x86_64

RUST_TARGET_ARM64 := aarch64-apple-darwin
RUST_TARGET_X86   := x86_64-apple-darwin
WIN_TARGET        := x86_64-pc-windows-gnu
WIN_TARGET_DIR    := /tmp/settings-demo-win

SCHEMA := $(abspath schema.toml)
CONFIG := $(abspath config.toml)
CARGO_MANIFEST := --manifest-path ../Cargo.toml

DIST := dist

APP_ARM64 := $(DIST)/$(ARCH_DARWIN_ARM64)/$(APP_NAME).app
APP_X86   := $(DIST)/$(ARCH_DARWIN_X86)/$(APP_NAME).app

APP_ZIP_ARM64 := $(DIST)/$(APP_NAME)-v$(VERSION)-darwin-arm64.zip
APP_ZIP_X86   := $(DIST)/$(APP_NAME)-v$(VERSION)-darwin-x86_64.zip
APP_DMG_ARM64 := $(DIST)/$(APP_NAME)-v$(VERSION)-darwin-arm64.dmg
APP_DMG_X86   := $(DIST)/$(APP_NAME)-v$(VERSION)-darwin-x86_64.dmg

WIN_EXE   := $(DIST)/$(ARCH_WIN)/$(APP_NAME).exe
WIN_ZIP   := $(DIST)/$(APP_NAME)-v$(VERSION)-windows-x86_64.zip
LINUX_BIN := $(DIST)/$(ARCH_LINUX)/$(EXE_NAME)
APP_ZIP_LINUX := $(DIST)/$(APP_NAME)-v$(VERSION)-linux-x86_64.zip

DMG_SETTINGS := dmg_settings.py
ICON_SRC_MACOS := assets/appicon-macos.png
ICONSET        := $(DIST)/AppIcon.iconset

CERT      := $(APPLE_DEVELOPER_CERTIFICATE_NAME)
TEAM_ID   := $(APPLE_DEVELOPER_TEAM_ID)
APPLE_ID_ := $(APPLE_ID)
APP_PASS  := $(APPLE_DEVELOPER_APP_PASSWORD)

ICON_TTF    := ../assets/icons.ttf
APPICON_ICO := ../assets/appicon.ico

.PHONY: all demo-arm64 demo-x86_64 demo-win demo-linux \
	demo-zip-arm64 demo-zip-x86_64 demo-zip-win demo-zip-linux \
	demo-dmg-arm64 demo-dmg-x86_64 sign-arm64 sign-x86_64 \
	notarize-arm64 notarize-x86_64 clean help \
	_mac_demo_app _plist _icns_if_present _icns_build \
	mac win linux win-zip mac-zip mac-dmg sign notarize

## Default: demo macOS .app for the current host arch.
all: demo-arm64

../assets/icons.ttf:
	$(MAKE) -C .. icons

../assets/appicon.ico: ../assets/appicon.png
	$(MAKE) -C .. appicon-ico

# -----------------------------------------------------------------------
# Per-arch demo GUI → demo/dist/
# -----------------------------------------------------------------------

demo-arm64: $(ICON_TTF)
	$(MAKE) _mac_demo_app PM_ARCH_SLUG=$(ARCH_DARWIN_ARM64) \
		PM_RUST_TARGET=$(RUST_TARGET_ARM64) \
		PM_SETTINGS_BIN=../target/$(RUST_TARGET_ARM64)/release/$(EXE_NAME)
	@echo ">>> Done: $(APP_ARM64)"
	@echo "    Run:  open '$(APP_ARM64)'"

demo-x86_64: $(ICON_TTF)
	$(MAKE) _mac_demo_app PM_ARCH_SLUG=$(ARCH_DARWIN_X86) \
		PM_RUST_TARGET=$(RUST_TARGET_X86) \
		PM_SETTINGS_BIN=../target/$(RUST_TARGET_X86)/release/$(EXE_NAME)
	@echo ">>> Done: $(APP_X86)"

.PHONY: _mac_demo_app
_mac_demo_app:
	@test -n "$(PM_ARCH_SLUG)" && test -n "$(PM_RUST_TARGET)" && test -n "$(PM_SETTINGS_BIN)"
	@echo ">>> Building demo macOS .app  (arch: $(PM_ARCH_SLUG), schema: $(SCHEMA))"
	MACOSX_DEPLOYMENT_TARGET=$(MIN_MACOS) SETTINGS_SCHEMA="$(SCHEMA)" \
		cargo build $(CARGO_MANIFEST) --release -p settings --target $(PM_RUST_TARGET)
	@mkdir -p "$(DIST)/$(PM_ARCH_SLUG)/$(APP_NAME).app/Contents/MacOS"
	@mkdir -p "$(DIST)/$(PM_ARCH_SLUG)/$(APP_NAME).app/Contents/Resources"
	cp "$(PM_SETTINGS_BIN)" \
		"$(DIST)/$(PM_ARCH_SLUG)/$(APP_NAME).app/Contents/MacOS/$(APP_NAME)"
	cp "$(CONFIG)" \
		"$(DIST)/$(PM_ARCH_SLUG)/$(APP_NAME).app/Contents/Resources/config.toml"
	$(MAKE) _plist PLIST_CONTENTS="$(DIST)/$(PM_ARCH_SLUG)/$(APP_NAME).app/Contents"
	$(MAKE) _icns_if_present \
		ICNS_RES_DIR="$(DIST)/$(PM_ARCH_SLUG)/$(APP_NAME).app/Contents/Resources" \
		ICNS_OUT="$(DIST)/$(PM_ARCH_SLUG)/$(APP_NAME).app/Contents/Resources/AppIcon.icns"

demo-win: $(ICON_TTF) $(APPICON_ICO)
	@echo ">>> Building demo Windows .exe  (schema: $(SCHEMA))"
	SETTINGS_SCHEMA="$(SCHEMA)" CARGO_TARGET_DIR="$(WIN_TARGET_DIR)" \
		cargo build $(CARGO_MANIFEST) --release -p settings --target $(WIN_TARGET)
	@mkdir -p "$(DIST)/$(ARCH_WIN)"
	@cp "$(WIN_TARGET_DIR)/$(WIN_TARGET)/release/settings.exe" "$(WIN_EXE)"
	@cp "$(CONFIG)" "$(DIST)/$(ARCH_WIN)/config.toml"
	@echo ">>> Output: $(WIN_EXE)"

demo-linux: $(ICON_TTF)
	@echo ">>> Building demo Linux binary  (schema: $(SCHEMA))"
	SETTINGS_SCHEMA="$(SCHEMA)" cargo build $(CARGO_MANIFEST) --release -p settings
	@mkdir -p "$(DIST)/$(ARCH_LINUX)"
	@cp ../target/release/$(EXE_NAME) "$(LINUX_BIN)"
	@cp "$(CONFIG)" "$(DIST)/$(ARCH_LINUX)/config.toml"
	@echo ">>> Output: $(LINUX_BIN)"

# -----------------------------------------------------------------------
# Packages
# -----------------------------------------------------------------------

demo-zip-arm64: demo-arm64
	ditto -c -k --keepParent "$(APP_ARM64)" "$(APP_ZIP_ARM64)"
	@echo ">>> Package: $(APP_ZIP_ARM64)"

demo-zip-x86_64: demo-x86_64
	ditto -c -k --keepParent "$(APP_X86)" "$(APP_ZIP_X86)"
	@echo ">>> Package: $(APP_ZIP_X86)"

demo-zip-win: demo-win
	@rm -f "$(WIN_ZIP)"
	cd "$(DIST)/$(ARCH_WIN)" && \
		zip "../../$(notdir $(WIN_ZIP))" "$(APP_NAME).exe" config.toml
	@echo ">>> Package: $(WIN_ZIP)"

demo-zip-linux: demo-linux
	@rm -f "$(APP_ZIP_LINUX)"
	cd "$(DIST)/$(ARCH_LINUX)" && \
		zip "../../$(notdir $(APP_ZIP_LINUX))" "$(EXE_NAME)" config.toml
	@echo ">>> Package: $(APP_ZIP_LINUX)"

demo-dmg-arm64: demo-arm64
	@command -v dmgbuild >/dev/null 2>&1 || (echo "Error: dmgbuild not found. Run: pipx install dmgbuild" && exit 1)
	dmgbuild -s "$(DMG_SETTINGS)" -D app="$(APP_ARM64)" "$(APP_NAME)" "$(APP_DMG_ARM64)"
	@echo ">>> Package: $(APP_DMG_ARM64)"

demo-dmg-x86_64: demo-x86_64
	@command -v dmgbuild >/dev/null 2>&1 || (echo "Error: dmgbuild not found. Run: pipx install dmgbuild" && exit 1)
	dmgbuild -s "$(DMG_SETTINGS)" -D app="$(APP_X86)" "$(APP_NAME)" "$(APP_DMG_X86)"
	@echo ">>> Package: $(APP_DMG_X86)"

# -----------------------------------------------------------------------
# Code signing / notarization
# -----------------------------------------------------------------------

sign-arm64: demo-arm64
	@test -n "$(CERT)" || (echo "Error: APPLE_DEVELOPER_CERTIFICATE_NAME is not set" && exit 1)
	xattr -cr "$(APP_ARM64)"
	codesign --deep --force --options runtime \
		--entitlements entitlements.plist \
		--sign "$(CERT)" \
		"$(APP_ARM64)"
	@echo ">>> Signed: $(APP_ARM64)"

sign-x86_64: demo-x86_64
	@test -n "$(CERT)" || (echo "Error: APPLE_DEVELOPER_CERTIFICATE_NAME is not set" && exit 1)
	xattr -cr "$(APP_X86)"
	codesign --deep --force --options runtime \
		--entitlements entitlements.plist \
		--sign "$(CERT)" \
		"$(APP_X86)"
	@echo ">>> Signed: $(APP_X86)"

notarize-arm64: sign-arm64
	@test -n "$(TEAM_ID)"   || (echo "Error: APPLE_DEVELOPER_TEAM_ID is not set"      && exit 1)
	@test -n "$(APPLE_ID_)" || (echo "Error: APPLE_ID is not set"                     && exit 1)
	@test -n "$(APP_PASS)"  || (echo "Error: APPLE_DEVELOPER_APP_PASSWORD is not set" && exit 1)
	@command -v dmgbuild >/dev/null 2>&1 || (echo "Error: dmgbuild not found. Run: pipx install dmgbuild" && exit 1)
	dmgbuild -s "$(DMG_SETTINGS)" -D app="$(APP_ARM64)" "$(APP_NAME)" "$(APP_DMG_ARM64)"
	xcrun notarytool submit "$(APP_DMG_ARM64)" \
		--apple-id  "$(APPLE_ID_)" \
		--password  "$(APP_PASS)" \
		--team-id   "$(TEAM_ID)" \
		--wait
	xcrun stapler staple "$(APP_DMG_ARM64)"
	@echo ">>> Notarized and stapled: $(APP_DMG_ARM64)"

notarize-x86_64: sign-x86_64
	@test -n "$(TEAM_ID)"   || (echo "Error: APPLE_DEVELOPER_TEAM_ID is not set"      && exit 1)
	@test -n "$(APPLE_ID_)" || (echo "Error: APPLE_ID is not set"                     && exit 1)
	@test -n "$(APP_PASS)"  || (echo "Error: APPLE_DEVELOPER_APP_PASSWORD is not set" && exit 1)
	@command -v dmgbuild >/dev/null 2>&1 || (echo "Error: dmgbuild not found. Run: pipx install dmgbuild" && exit 1)
	dmgbuild -s "$(DMG_SETTINGS)" -D app="$(APP_X86)" "$(APP_NAME)" "$(APP_DMG_X86)"
	xcrun notarytool submit "$(APP_DMG_X86)" \
		--apple-id  "$(APPLE_ID_)" \
		--password  "$(APP_PASS)" \
		--team-id   "$(TEAM_ID)" \
		--wait
	xcrun stapler staple "$(APP_DMG_X86)"
	@echo ">>> Notarized and stapled: $(APP_DMG_X86)"

# Backward-compatible aliases (deprecated).
mac: demo-arm64
win: demo-win
linux: demo-linux
win-zip: demo-zip-win
linux-zip: demo-zip-linux
mac-zip: demo-zip-arm64
mac-dmg: demo-dmg-arm64
sign: sign-arm64
notarize: notarize-arm64

# -----------------------------------------------------------------------
# Info.plist / ICNS
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
		> "$(PLIST_CONTENTS)/Info.plist"

ICNS_RES_DIR ?=
ICNS_OUT       ?=

.PHONY: _icns_if_present
_icns_if_present:
	@test -n "$(ICNS_RES_DIR)" && test -n "$(ICNS_OUT)"
	@if [ -f "$(ICON_SRC_MACOS)" ]; then \
		$(MAKE) _icns_build ICNS_RES_DIR="$(ICNS_RES_DIR)" ICNS_OUT="$(ICNS_OUT)"; \
	else \
		echo "Note: $(ICON_SRC_MACOS) not found — skipping icon generation."; \
	fi

.PHONY: _icns_build
_icns_build:
	@test -n "$(ICNS_RES_DIR)" && test -n "$(ICNS_OUT)"
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
	iconutil -c icns "$(ICONSET)" -o "$(ICNS_OUT)"
	rm -rf "$(ICONSET)"

## Remove demo build artefacts (does not touch parent target/).
clean:
	rm -rf $(DIST) $(WIN_TARGET_DIR)

## Show this help.
help:
	@grep -E '^##' Makefile | sed 's/^## //'
