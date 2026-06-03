#!/usr/bin/env python3
"""Create a macOS .icns file from a square PNG using Pillow."""

import sys
from pathlib import Path

from PIL import Image


def main() -> int:
    if len(sys.argv) != 3:
        print("Usage: create-icns-pillow.py input.png output.icns", file=sys.stderr)
        return 1

    input_png = Path(sys.argv[1])
    output_icns = Path(sys.argv[2])

    if not input_png.is_file():
        print(f"Input file not found: {input_png}", file=sys.stderr)
        return 1

    image = Image.open(input_png).convert("RGBA")
    sizes = [(16, 16), (32, 32), (64, 64), (128, 128), (256, 256), (512, 512), (1024, 1024)]
    image.save(output_icns, format="ICNS", sizes=sizes)
    print(f"Icon created with Pillow fallback: {output_icns}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
