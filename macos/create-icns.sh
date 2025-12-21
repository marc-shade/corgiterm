#!/bin/bash
# Convert SVG to macOS icns icon
# Usage: create-icns.sh input.svg output.icns

set -e

INPUT_SVG="$1"
OUTPUT_ICNS="$2"

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

# Generate all required icon sizes
SIZES=(16 32 64 128 256 512 1024)

for SIZE in "${SIZES[@]}"; do
    echo "  Generating ${SIZE}x${SIZE}..."

    if [ "$CONVERT_CMD" = "rsvg-convert" ]; then
        rsvg-convert -w $SIZE -h $SIZE "$INPUT_SVG" -o "$ICONSET_DIR/icon_${SIZE}x${SIZE}.png"
    elif [ "$CONVERT_CMD" = "convert" ]; then
        convert -background none -resize ${SIZE}x${SIZE} "$INPUT_SVG" "$ICONSET_DIR/icon_${SIZE}x${SIZE}.png"
    fi

    # Create @2x versions for Retina displays
    if [ $SIZE -le 512 ]; then
        DOUBLE=$((SIZE * 2))
        if [ "$CONVERT_CMD" = "rsvg-convert" ]; then
            rsvg-convert -w $DOUBLE -h $DOUBLE "$INPUT_SVG" -o "$ICONSET_DIR/icon_${SIZE}x${SIZE}@2x.png"
        elif [ "$CONVERT_CMD" = "convert" ]; then
            convert -background none -resize ${DOUBLE}x${DOUBLE} "$INPUT_SVG" "$ICONSET_DIR/icon_${SIZE}x${SIZE}@2x.png"
        fi
    fi
done

# Rename to match Apple's expected naming
cd "$ICONSET_DIR"
[ -f icon_1024x1024.png ] && mv icon_1024x1024.png icon_512x512@2x.png 2>/dev/null || true

echo "Creating icns file..."
iconutil -c icns "$ICONSET_DIR" -o "$OUTPUT_ICNS"

# Cleanup
rm -rf "$(dirname "$ICONSET_DIR")"

echo "Icon created: $OUTPUT_ICNS"
