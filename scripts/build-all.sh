#!/bin/bash
set -e

echo "CrossCrypt Build Script"
echo "======================="

# Get script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

cd "$PROJECT_DIR"

# Parse arguments
RELEASE=false
TARGETS=()

while [[ $# -gt 0 ]]; do
    case $1 in
        --release)
            RELEASE=true
            shift
            ;;
        --target)
            TARGETS+=("$2")
            shift 2
            ;;
        --all)
            TARGETS=("x86_64-pc-windows-gnu" "x86_64-apple-darwin" "x86_64-unknown-linux-gnu")
            shift
            ;;
        *)
            echo "Unknown option: $1"
            echo "Usage: $0 [--release] [--target <target>] [--all]"
            exit 1
            ;;
    esac
done

# Default to current platform if no targets specified
if [ ${#TARGETS[@]} -eq 0 ]; then
    TARGETS=("native")
fi

# Build flags
if [ "$RELEASE" = true ]; then
    BUILD_FLAGS="--release"
    BUILD_DIR="release"
else
    BUILD_FLAGS=""
    BUILD_DIR="debug"
fi

# Create dist directory
mkdir -p "$PROJECT_DIR/dist"

# Build for each target
for target in "${TARGETS[@]}"; do
    echo ""
    echo "Building for: $target"
    echo "------------------------"
    
    if [ "$target" = "native" ]; then
        cargo build $BUILD_FLAGS
        
        # Copy binary
        if [[ "$OSTYPE" == "msys" || "$OSTYPE" == "win32" ]]; then
            cp "$PROJECT_DIR/target/$BUILD_DIR/crosscrypt.exe" "$PROJECT_DIR/dist/"
        else
            cp "$PROJECT_DIR/target/$BUILD_DIR/crosscrypt" "$PROJECT_DIR/dist/"
        fi
    else
        # Cross compilation
        rustup target add "$target" 2>/dev/null || true
        cargo build $BUILD_FLAGS --target "$target"
        
        # Copy binary with target name
        if [[ "$target" == *"windows"* ]]; then
            cp "$PROJECT_DIR/target/$target/$BUILD_DIR/crosscrypt.exe" "$PROJECT_DIR/dist/crosscrypt-$target.exe"
        else
            cp "$PROJECT_DIR/target/$target/$BUILD_DIR/crosscrypt" "$PROJECT_DIR/dist/crosscrypt-$target"
        fi
    fi
done

echo ""
echo "Build complete! Binaries in: $PROJECT_DIR/dist/"
ls -la "$PROJECT_DIR/dist/"
