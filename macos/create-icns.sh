#!/bin/bash
# Convert SVG to macOS icns icon
# Usage: create-icns.sh input.svg output.icns

set -e

INPUT_SVG="$1"
OUTPUT_ICNS="$2"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

if [ -z "$INPUT_SVG" ] || [ -z "$OUTPUT_ICNS" ]; then
    echo "Usage: $0 input.svg output.icns"
    exit 1
fi

if [ ! -f "$INPUT_SVG" ]; then
    echo "Error: Input file not found: $INPUT_SVG"
    exit 1
fi

# Create temporary iconset directory
ICONSET_DIR=$(mktemp -d)/AppIcon.iconset
mkdir -p "$ICONSET_DIR"

echo "Converting SVG to PNG at multiple sizes..."

# Check for available tools
if command -v rsvg-convert &> /dev/null; then
    # Use librsvg (best quality)
    CONVERT_CMD="rsvg-convert"
elif command -v convert &> /dev/null; then
    # Use ImageMagick
    CONVERT_CMD="convert"
elif command -v qlmanage &> /dev/null; then
    # Use macOS Quick Look (fallback)
    CONVERT_CMD="qlmanage"
else
    echo "Error: No SVG converter found. Install librsvg or imagemagick:"
    echo "  brew install librsvg"
    exit 1
fi

generate_png() {
    local pixels="$1"
    local output="$2"

    echo "  Generating ${output##*/} (${pixels}x${pixels})..."

    if [ "$CONVERT_CMD" = "rsvg-convert" ]; then
        rsvg-convert -w "$pixels" -h "$pixels" "$INPUT_SVG" -o "$output"
    elif [ "$CONVERT_CMD" = "convert" ]; then
        convert -background none -resize "${pixels}x${pixels}" "$INPUT_SVG" "$output"
    fi
}

# Generate the exact iconset file names required by iconutil.
generate_png 16 "$ICONSET_DIR/icon_16x16.png"
generate_png 32 "$ICONSET_DIR/icon_16x16@2x.png"
generate_png 32 "$ICONSET_DIR/icon_32x32.png"
generate_png 64 "$ICONSET_DIR/icon_32x32@2x.png"
generate_png 128 "$ICONSET_DIR/icon_128x128.png"
generate_png 256 "$ICONSET_DIR/icon_128x128@2x.png"
generate_png 256 "$ICONSET_DIR/icon_256x256.png"
generate_png 512 "$ICONSET_DIR/icon_256x256@2x.png"
generate_png 512 "$ICONSET_DIR/icon_512x512.png"
generate_png 1024 "$ICONSET_DIR/icon_512x512@2x.png"

echo "Creating icns file..."
if ! iconutil -c icns "$ICONSET_DIR" -o "$OUTPUT_ICNS"; then
    echo "iconutil rejected the iconset; trying Pillow fallback..."

    BASE_PNG="$(mktemp "${TMPDIR:-/tmp}/corgiterm-icon-1024.XXXXXX.png")"
    if [ "$CONVERT_CMD" = "rsvg-convert" ]; then
        rsvg-convert -w 1024 -h 1024 "$INPUT_SVG" -o "$BASE_PNG"
    elif [ "$CONVERT_CMD" = "convert" ]; then
        convert -background none -resize 1024x1024 "$INPUT_SVG" "$BASE_PNG"
    fi

    python3 "$SCRIPT_DIR/create-icns-pillow.py" "$BASE_PNG" "$OUTPUT_ICNS"
    rm -f "$BASE_PNG"
fi

# Cleanup
rm -rf "$(dirname "$ICONSET_DIR")"

echo "Icon created: $OUTPUT_ICNS"
