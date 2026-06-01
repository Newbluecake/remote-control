#!/bin/bash
set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
BINARY="$SCRIPT_DIR/remote-control"

if [ ! -f "$BINARY" ]; then
    echo "Error: remote-control not found in $SCRIPT_DIR"
    exit 1
fi

codesign --force --sign - "$BINARY"
echo "Signed successfully. You can now grant Accessibility permission to:"
echo "  $BINARY"
echo ""
echo "Go to: System Settings -> Privacy & Security -> Accessibility"
echo "  then add remote-control to the list."
