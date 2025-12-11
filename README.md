# GPUI Web Platform (WebAssembly/WebGPU)

> **Note**: This implementation was developed with assistance from [Claude Code](https://claude.ai/code).

This fork adds web platform support to [GPUI](https://www.gpui.rs/), the GPU-accelerated UI framework from [Zed](https://zed.dev).

**[Live Demo →](https://hugomoran159.github.io/gpui-web-demo/examples.html)**

## Overview

GPUI now runs in the browser via WebAssembly with support for both **WebGPU** and **WebGL2** backends (via wgpu's automatic fallback). All 31 GPUI examples work in modern browsers.

### What Works

- Text rendering (via cosmic-text)
- Images, GIFs, and SVGs
- Animations and transitions
- Mouse input (click, hover, drag & drop)
- Keyboard input and text editing
- Scrolling and uniform lists
- Gradients, shadows, patterns, and paths
- Flexbox and grid layouts

### What's Not Implemented

- Clipboard (read/write)
- File dialogs
- Native menus
- Multi-window support
- Drag & drop to/from OS

## Technical Approach

The implementation adds a new platform module (`crates/gpui/src/platform/web/`) alongside the existing macOS, Linux, and Windows implementations. Key components:

| Component | Implementation |
|-----------|----------------|
| Rendering | WebGPU or WebGL2 via `wgpu` (auto-detected) |
| Text | `cosmic-text` (same as Linux) |
| Event loop | `requestAnimationFrame` via `wasm-bindgen` |
| Async runtime | `futures` crate (no `smol` in WASM) |
| Time | `web_time::Instant` (no `std::time::Instant`) |

### Core Changes

Changes to existing GPUI code are minimal and `cfg`-gated:

1. **Async GPU Init**: WebGPU requires async `request_adapter().await` - added `is_renderer_ready()` check to defer first draw
2. **No Blocking**: WASM can't block - `executor.block()` panics with helpful message
3. **Time APIs**: `std::time::Instant` unavailable - conditional `web_time` import
4. **HTTP Client**: Web fetch API stub replaces native `http_client`

All native platforms remain unchanged. See [`crates/gpui/src/platform/web/CHANGES.md`](crates/gpui/src/platform/web/CHANGES.md) for detailed documentation.

## Building for Web

### Prerequisites

```bash
rustup target add wasm32-unknown-unknown
cargo install wasm-pack
```

### Build an Example

```bash
cd crates/gpui_web_example
wasm-pack build --target web --out-dir www/pkg
```

### Serve Locally

```bash
cd www && python3 -m http.server 8081
```

Open http://localhost:8081 in Chrome or Edge (WebGPU required).

## Project Structure

```
crates/
├── gpui/
│   └── src/
│       └── platform/
│           ├── mac/          # macOS (Metal)
│           ├── linux/        # Linux (Vulkan)
│           ├── windows/      # Windows (DirectX)
│           └── web/          # NEW: Web (WebGPU)
│               ├── platform.rs
│               ├── window.rs
│               ├── renderer.rs
│               ├── atlas.rs
│               ├── text_system.rs
│               ├── timer.rs
│               ├── http_client.rs
│               └── CHANGES.md
└── gpui_web_example/         # WASM build harness
    ├── src/lib.rs
    └── www/
        └── index.html
```

## Stats

- **~5,300 lines** of new web platform code
- **7 new files** in `platform/web/`
- **~150 lines** of changes to core GPUI (all `cfg`-gated)
- **0 changes** to native platform behavior

## Related Links

- [Zed Web Tracking Issue](https://github.com/zed-industries/zed/issues/5396)
- [GPUI Documentation](https://www.gpui.rs/)
- [Zed Repository](https://github.com/zed-industries/zed)

## License

Same as Zed - see [LICENSE](LICENSE).
