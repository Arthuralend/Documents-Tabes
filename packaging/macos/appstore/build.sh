#!/usr/bin/env bash
# Build Write (FS Office Write) for the Mac App Store: a sandboxed, universal Write.app signed
# with an Apple Distribution certificate and its Mac App Store provisioning profile, wrapped in a
# .pkg signed with a Mac Installer Distribution certificate, ready for Transporter or altool.
#
#   packaging/macos/appstore/build.sh [--skip-build] [--upload]
#
# Needs a Mac with Xcode's command-line tools, Rust with the aarch64-apple-darwin and
# x86_64-apple-darwin targets, and (env):
#   APPSTORE_BUNDLE_ID        the App ID registered for Write (default com.fantasticsoftware.write)
#   APPSTORE_APP_IDENTITY     "Apple Distribution: <name> (<TEAMID>)"
#   APPSTORE_INSTALLER_IDENTITY "3rd Party Mac Developer Installer: <name> (<TEAMID>)"
#                             (Xcode calls it "Mac Installer Distribution")
#   APPSTORE_PROFILE          path to the .provisionprofile (Mac App Store distribution)
#   APPSTORE_VERSION          the version shown in the App Store (default: the release version)
# --upload also needs an App Store Connect API key:
#   APPSTORE_API_KEY_ID, APPSTORE_API_ISSUER (the key's .p8 in ~/.appstoreconnect/private_keys/)
#
# Output: $DIST/FS-Office-Write-<version>-appstore.pkg
# See docs/appstore/README.md for the whole path to the App Store.
set -euo pipefail
# shellcheck source=../../env.sh
. "$(dirname "${BASH_SOURCE[0]}")/../../env.sh"
HERE="$ROOT/packaging/macos"

SKIP_BUILD=0
UPLOAD=0
while [ $# -gt 0 ]; do
  case "$1" in
    --skip-build) SKIP_BUILD=1; shift ;;
    --upload) UPLOAD=1; shift ;;
    -h | --help) sed -n '2,20p' "$0"; exit 0 ;;
    *) echo "unknown argument: $1" >&2; exit 2 ;;
  esac
done

BUNDLE_ID="${APPSTORE_BUNDLE_ID:-com.fantasticsoftware.write}"
: "${APPSTORE_APP_IDENTITY:?set APPSTORE_APP_IDENTITY (Apple Distribution certificate)}"
: "${APPSTORE_INSTALLER_IDENTITY:?set APPSTORE_INSTALLER_IDENTITY (Mac Installer Distribution certificate)}"
: "${APPSTORE_PROFILE:?set APPSTORE_PROFILE (Mac App Store provisioning profile)}"
[ -f "$APPSTORE_PROFILE" ] || { echo "error: no provisioning profile at $APPSTORE_PROFILE" >&2; exit 1; }

# Keep in sync with LSMinimumSystemVersion in Info.plist.in.
export MACOSX_DEPLOYMENT_TARGET=11.0
# The version the App Store shows (APPSTORE_VERSION, e.g. 1.0); the release version otherwise.
SHORT_VERSION="${APPSTORE_VERSION:-${VERSION%%-*}}"
# The App Store wants a build number that grows with every upload of the same version.
BUILD_NUMBER="${APPSTORE_BUILD_NUMBER:-$(date -u +%Y%m%d%H%M)}"
WORK="$CARGO_TARGET_DIR/appstore-package"
APP="$WORK/Write.app"
PKG="$DIST/FS-Office-Write-$VERSION-appstore.pkg"
TARGETS=(aarch64-apple-darwin x86_64-apple-darwin)

echo "==> FS Office Write $SHORT_VERSION ($BUILD_NUMBER) for the Mac App Store, bundle id $BUNDLE_ID"

# ---- build -------------------------------------------------------------------------------------
if [ "$SKIP_BUILD" = 0 ]; then
  args=()
  for t in "${TARGETS[@]}"; do args+=(--target "$t"); done
  (cd "$ROOT" && cargo build --release --locked -p wordcraft "${args[@]}")
fi
rm -rf "$WORK"
mkdir -p "$WORK/bin" "$DIST"
inputs=()
for t in "${TARGETS[@]}"; do inputs+=("$CARGO_TARGET_DIR/$t/release/wordcraft"); done
lipo -create -output "$WORK/bin/wordcraft" "${inputs[@]}"

# ---- the bundle --------------------------------------------------------------------------------
mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources"
cp "$WORK/bin/wordcraft" "$APP/Contents/MacOS/Write"
cp "$ROOT/assets/app-icon/wordcraft.icns" "$APP/Contents/Resources/Write.icns"
cp "$ROOT/LICENSE-MIT" "$ROOT/LICENSE-APACHE" "$ROOT/NOTICE" "$APP/Contents/Resources/"
for loc in en ru hy; do
  mkdir -p "$APP/Contents/Resources/$loc.lproj"
  printf 'CFBundleDisplayName = "Write";\nCFBundleName = "Write";\n' > "$APP/Contents/Resources/$loc.lproj/InfoPlist.strings"
done
printf 'NSMicrophoneUsageDescription = "Write слушает микрофон, пока вы диктуете текст.";\nNSSpeechRecognitionUsageDescription = "Write превращает вашу речь в текст во время диктовки.";\n' \
  >> "$APP/Contents/Resources/ru.lproj/InfoPlist.strings"
sed -e "s/@VERSION@/$VERSION/g" -e "s/@SHORT_VERSION@/$SHORT_VERSION/g" \
  -e "s/@BUILD_SHA@/${WORDCRAFT_BUILD_SHA:-unknown}/g" "$HERE/Info.plist.in" >"$APP/Contents/Info.plist"
PL="$APP/Contents/Info.plist"
/usr/libexec/PlistBuddy -c "Set :CFBundleIdentifier $BUNDLE_ID" "$PL"
/usr/libexec/PlistBuddy -c "Set :CFBundleVersion $BUILD_NUMBER" "$PL"
# Export compliance: Write uses only the system's standard cryptography (opening password-protected
# Word documents), which is exempt; App Store Connect still asks once, see docs/appstore/README.md.
/usr/libexec/PlistBuddy -c "Add :ITSAppUsesNonExemptEncryption bool false" "$PL"
plutil -lint "$PL"
printf 'APPL????' >"$APP/Contents/PkgInfo"
cp "$APPSTORE_PROFILE" "$APP/Contents/embedded.provisionprofile"

# ---- sign and package --------------------------------------------------------------------------
xattr -cr "$APP"
codesign --force --timestamp --options runtime --entitlements "$HERE/appstore/entitlements.plist" \
  --sign "$APPSTORE_APP_IDENTITY" "$APP/Contents/MacOS/Write"
codesign --force --timestamp --options runtime --entitlements "$HERE/appstore/entitlements.plist" \
  --sign "$APPSTORE_APP_IDENTITY" "$APP"
codesign --verify --strict --deep --verbose=2 "$APP"
codesign -d --entitlements :- "$APP"
productbuild --component "$APP" /Applications --sign "$APPSTORE_INSTALLER_IDENTITY" "$PKG"
pkgutil --check-signature "$PKG"
echo "==> $PKG"

if [ "$UPLOAD" = 1 ]; then
  : "${APPSTORE_API_KEY_ID:?set APPSTORE_API_KEY_ID}"
  : "${APPSTORE_API_ISSUER:?set APPSTORE_API_ISSUER}"
  xcrun altool --validate-app -f "$PKG" -t macos --apiKey "$APPSTORE_API_KEY_ID" --apiIssuer "$APPSTORE_API_ISSUER"
  xcrun altool --upload-app -f "$PKG" -t macos --apiKey "$APPSTORE_API_KEY_ID" --apiIssuer "$APPSTORE_API_ISSUER"
fi
