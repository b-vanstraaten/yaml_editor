#!/bin/bash
# Patch Info.plist in the generated .app bundle to ensure the icon is set correctly

cargo bundle --release

APP_NAME="YAML Viewer"
BUNDLE_PATH="target/release/bundle/osx/$APP_NAME.app/Contents/Info.plist"

if [ -f "$BUNDLE_PATH" ]; then
  echo "Patching CFBundleIconFile in $BUNDLE_PATH ..."
  /usr/libexec/PlistBuddy -c "Set :CFBundleIconFile icon.icns" "$BUNDLE_PATH" || \
  /usr/libexec/PlistBuddy -c "Add :CFBundleIconFile string icon.icns" "$BUNDLE_PATH"
  echo "Done."
else
  echo "Info.plist not found at $BUNDLE_PATH. Did you run cargo bundle --release?"
fi

