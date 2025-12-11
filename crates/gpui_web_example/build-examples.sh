#!/bin/bash
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$SCRIPT_DIR/../.."
WWW_DIR="$SCRIPT_DIR/www"
PKG_DIR="$WWW_DIR/pkg"
EXAMPLES_DIR="$PKG_DIR/examples"

# All GPUI examples
EXAMPLES=(
    "hello_world"
    "painting"
    "gradient"
    "animation"
    "text"
    "shadow"
    "opacity"
    "input"
    "scrollable"
    "data_table"
    "drag_drop"
    "focus_visible"
    "gif_viewer"
    "grid_layout"
    "image"
    "image_gallery"
    "image_loading"
    "on_window_close_quit"
    "ownership_post"
    "paths_bench"
    "pattern"
    "set_menus"
    "svg"
    "tab_stop"
    "text_layout"
    "text_wrapper"
    "tree"
    "uniform_list"
    "window"
    "window_positioning"
    "window_shadow"
)

echo "Building GPUI web examples..."
echo "Output directory: $WWW_DIR"

# Create directories
mkdir -p "$EXAMPLES_DIR"
mkdir -p "$WWW_DIR/assets"

# Copy example assets to www directory for serving
echo "Copying example assets..."
cp -r "$REPO_ROOT/crates/gpui/examples/image" "$WWW_DIR/assets/"
cp -r "$REPO_ROOT/crates/gpui/examples/svg" "$WWW_DIR/assets/" 2>/dev/null || true

# Change to main workspace root for cargo commands
cd "$REPO_ROOT"

# Build each example
for example in "${EXAMPLES[@]}"; do
    echo ""
    echo "=== Building $example ==="

    # Build the example for wasm32
    cargo build -p gpui --example "$example" --target wasm32-unknown-unknown --release --features web

    # Run wasm-bindgen
    mkdir -p "$EXAMPLES_DIR/$example"
    ~/.cargo/bin/wasm-bindgen \
        --target web \
        --out-dir "$EXAMPLES_DIR/$example" \
        "$REPO_ROOT/target/wasm32-unknown-unknown/release/examples/$example.wasm"

    echo "Built $example -> $EXAMPLES_DIR/$example"
done

echo ""
echo "=== Build complete ==="
echo "Start the server with: python3 -m http.server 8080 --directory $WWW_DIR"
echo "Then visit: http://localhost:8080/examples.html"
