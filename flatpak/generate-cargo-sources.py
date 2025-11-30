#!/usr/bin/env python3
"""
Generate cargo-sources.json for Flatpak from Cargo.lock

This script parses Cargo.lock and generates the JSON file needed
for offline Flatpak builds.

Usage:
    cd corgiterm
    python3 flatpak/generate-cargo-sources.py > flatpak/cargo-sources.json
"""

import json
import re
import sys
from pathlib import Path
from urllib.parse import quote


def parse_cargo_lock(lock_path: Path) -> list:
    """Parse Cargo.lock and return list of sources for Flatpak."""
    content = lock_path.read_text()
    sources = []

    # Split into packages
    packages = content.split("[[package]]")[1:]  # Skip header

    for pkg_text in packages:
        lines = pkg_text.strip().split("\n")
        pkg = {}
        for line in lines:
            if "=" in line:
                key, value = line.split("=", 1)
                key = key.strip()
                value = value.strip().strip('"')
                pkg[key] = value

        name = pkg.get("name", "")
        version = pkg.get("version", "")
        source = pkg.get("source", "")
        checksum = pkg.get("checksum", "")

        # Skip path dependencies (local crates)
        if not source or source.startswith("git+"):
            continue

        # Only process crates.io packages
        if "registry+https://github.com/rust-lang/crates.io-index" in source:
            crate_url = f"https://static.crates.io/crates/{name}/{name}-{version}.crate"

            source_entry = {
                "type": "file",
                "url": crate_url,
                "sha256": checksum,
                "dest": "cargo/vendor",
                "dest-filename": f"{name}-{version}.crate"
            }
            sources.append(source_entry)

    return sources


def main():
    # Find Cargo.lock
    script_dir = Path(__file__).parent
    project_root = script_dir.parent
    lock_path = project_root / "Cargo.lock"

    if not lock_path.exists():
        print(f"Error: Cargo.lock not found at {lock_path}", file=sys.stderr)
        sys.exit(1)

    sources = parse_cargo_lock(lock_path)

    # Add cargo config
    cargo_config = {
        "type": "inline",
        "contents": """[source.crates-io]
replace-with = "vendored-sources"

[source.vendored-sources]
directory = "cargo/vendor"
""",
        "dest": ".cargo",
        "dest-filename": "config.toml"
    }

    all_sources = [cargo_config] + sources

    print(json.dumps(all_sources, indent=2))


if __name__ == "__main__":
    main()
