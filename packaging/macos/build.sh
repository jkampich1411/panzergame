#!/usr/bin/env bash
# Baut alle macOS-Downloads nach dist/, jeweils als Universal-Binary (Intel + Apple Silicon):
#   Panzer-macos-universal.dmg          Launcher als Panzer.app
#   panzer-game-macos-universal.zip     Spielpaket, das der Launcher lädt
#   panzer-server-macos-universal.zip   dedizierter Server
# Die Dateinamen enthalten keine Version, sonst brechen die Links auf der Website.
set -euo pipefail
cd "$(dirname "$0")/../.."

VERSION=$(sed -n 's/^version = "\(.*\)"$/\1/p' Cargo.toml | head -n1)
DIST=$PWD/dist
WORK=$PWD/target/packaging/macos
export MACOSX_DEPLOYMENT_TARGET=11.0

for target in aarch64-apple-darwin x86_64-apple-darwin; do
    rustup target add "$target"
    cargo build --release --locked --target "$target" -p panzer -p panzer-launcher -p panzer-server
done

rm -rf "$WORK"
mkdir -p "$DIST" "$WORK"

for bin in panzer panzer-launcher panzer-server; do
    lipo -create -output "$WORK/$bin" "target/aarch64-apple-darwin/release/$bin" "target/x86_64-apple-darwin/release/$bin"
    # Apple Silicon startet nichts ohne Signatur. Das ist nur ad hoc, kein Apple-Developer-Zertifikat.
    codesign --force --sign - "$WORK/$bin"
done

rm -f "$DIST/panzer-game-macos-universal.zip" "$DIST/panzer-server-macos-universal.zip"
zip -q -j "$DIST/panzer-game-macos-universal.zip" "$WORK/panzer"
zip -q -j "$DIST/panzer-server-macos-universal.zip" "$WORK/panzer-server"

APP=$WORK/dmg/Panzer.app
mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources"
cp "$WORK/panzer-launcher" "$APP/Contents/MacOS/panzer-launcher"
cp assets/icon/panzer.icns "$APP/Contents/Resources/panzer.icns"
sed "s/__VERSION__/$VERSION/g" packaging/macos/Info.plist >"$APP/Contents/Info.plist"
codesign --force --deep --sign - "$APP"
ln -s /Applications "$WORK/dmg/Applications"

# hdiutil scheitert auf CI-Rechnern gelegentlich mit "Resource busy"
for attempt in 1 2 3 4 5; do
    if hdiutil create -volname Panzer -srcfolder "$WORK/dmg" -ov -format UDZO "$DIST/Panzer-macos-universal.dmg"; then
        break
    fi
    [ "$attempt" = 5 ] && exit 1
    sleep 10
done

ls -l "$DIST"
