#!/bin/bash
# CorgiTerm build script

set -e

echo "🐕 Building CorgiTerm..."

# Check for required dependencies
check_deps() {
    local missing=()

    if ! pkg-config --exists gtk4; then
        missing+=("gtk4-devel")
    fi

    if ! pkg-config --exists libadwaita-1; then
        missing+=("libadwaita-devel")
    fi

    if [ ${#missing[@]} -ne 0 ]; then
        echo "Missing dependencies: ${missing[*]}"
        echo "Install with: sudo dnf install ${missing[*]}"
        exit 1
    fi
}

# Build mode
MODE="${1:-debug}"

case "$MODE" in
    debug)
        echo "Building in debug mode..."
        cargo build
        echo "Binary: target/debug/corgiterm"
        ;;
    release)
        echo "Building in release mode..."
        cargo build --release
        echo "Binary: target/release/corgiterm"
        ;;
    test)
        echo "Running tests..."
        cargo test --workspace
        ;;
    check)
        echo "Running checks..."
        cargo fmt --all -- --check
        cargo clippy --workspace -- -D warnings
        cargo test --workspace
        ;;
    *)
        echo "Usage: $0 [debug|release|test|check]"
        exit 1
        ;;
esac

echo "✅ Build complete!"
