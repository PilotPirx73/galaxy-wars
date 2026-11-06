#!/bin/sh
# Baut "Galaxy Wars.app" (per Doppelklick startbar).
# Ist das Intel-Ziel installiert (rustup target add x86_64-apple-darwin),
# entsteht eine Universal-App für Apple-Silicon- und Intel-Macs.
set -e
cd "$(dirname "$0")"

APP="Galaxy Wars.app"
if rustup target list --installed | grep -q x86_64-apple-darwin; then
  cargo build --release --target aarch64-apple-darwin
  cargo build --release --target x86_64-apple-darwin
  BIN_ARM=target/aarch64-apple-darwin/release/galaxy-wars
  BIN_X86=target/x86_64-apple-darwin/release/galaxy-wars
  UNIVERSAL=1
else
  cargo build --release
  BIN=target/release/galaxy-wars
  UNIVERSAL=0
fi

rm -rf "$APP"
mkdir -p "$APP/Contents/MacOS"
if [ "$UNIVERSAL" = 1 ]; then
  lipo -create -output "$APP/Contents/MacOS/galaxy-wars" "$BIN_ARM" "$BIN_X86"
else
  cp "$BIN" "$APP/Contents/MacOS/galaxy-wars"
fi

cat > "$APP/Contents/Info.plist" <<'EOF'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleName</key><string>Galaxy Wars 2026</string>
  <key>CFBundleDisplayName</key><string>Galaxy Wars 2026</string>
  <key>CFBundleIdentifier</key><string>local.galaxy-wars</string>
  <key>CFBundleVersion</key><string>0.1.0</string>
  <key>CFBundleShortVersionString</key><string>0.1.0</string>
  <key>CFBundleExecutable</key><string>galaxy-wars</string>
  <key>CFBundlePackageType</key><string>APPL</string>
  <key>NSHighResolutionCapable</key><true/>
  <key>LSMinimumSystemVersion</key><string>11.0</string>
</dict>
</plist>
EOF

# Ad-hoc-Signatur, damit macOS die App lokal ohne Rückfrage startet
codesign --force --deep --sign - "$APP" >/dev/null 2>&1 || true
echo "Fertig: $(pwd)/$APP"
