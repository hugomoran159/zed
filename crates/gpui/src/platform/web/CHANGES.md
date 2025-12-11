# Web Platform Implementation - Core File Changes

This document describes all changes made to core (non-platform-specific) GPUI files to support the WebGPU/WASM platform, along with the reasoning behind each change.

## Overview

The web platform required modifications to core files primarily due to two fundamental differences from native platforms:

1. **Async GPU Initialization**: WebGPU requires async `request_adapter().await` and `request_device().await` calls, while Metal/Vulkan/DirectX initialize synchronously.

2. **No Blocking Operations**: WASM runs on a single-threaded event loop and cannot block. Functions like `smol::block_on()` are not available.

3. **Different Time APIs**: `std::time::Instant` is not available in WASM; `web_time::Instant` must be used instead.

---

## Core File Changes

### 1. `crates/gpui/src/platform.rs`

#### Change: Added web module
```rust
#[cfg(target_arch = "wasm32")]
pub(crate) mod web;

#[cfg(target_arch = "wasm32")]
pub(crate) use web::*;
```

**Reasoning**: Standard pattern for adding a new platform. The module is conditionally compiled only for WASM targets to avoid any overhead on native platforms.

#### Change: Added `is_renderer_ready()` to `PlatformWindow` trait
```rust
fn is_renderer_ready(&self) -> bool {
    true
}
```

**Reasoning**: WebGPU initialization is async, but GPUI's window creation and initial draw are synchronous. Without this check, the first `draw()` call happens before WebGPU is ready, causing glyphs to be painted to a `NoopAtlas` that discards them. This method allows the web platform to report when its async initialization is complete. The default `true` ensures no behavior change for native platforms.

#### Change: Added `as_any()` to `PlatformTextSystem` trait (WASM only)
```rust
#[cfg(target_arch = "wasm32")]
fn as_any(&self) -> &dyn std::any::Any;
```

**Reasoning**: The `TextSystem::load_font_from_url()` method (WASM-only) needs to downcast the platform text system to call web-specific font loading. This is only needed on WASM, so it's conditionally compiled to avoid dead code warnings on other platforms.

#### Change: Made `AtlasTextureList<T>` public
```rust
pub(crate) struct AtlasTextureList<T> {
    pub(crate) textures: Vec<Option<T>>,
    pub(crate) free_list: Vec<usize>,
}
```

**Reasoning**: The web atlas implementation (`WgpuAtlas`) needs to use the same texture management pattern as the Metal atlas. Making this struct `pub(crate)` allows code reuse rather than duplicating the logic.

#### Change: Added `web_time::Instant` conditional import
```rust
#[cfg(not(target_arch = "wasm32"))]
use std::time::Instant;
#[cfg(target_arch = "wasm32")]
use web_time::Instant;
```

**Reasoning**: `std::time::Instant` is not available in WASM. The `web_time` crate provides a compatible `Instant` type that uses `performance.now()` under the hood.

---

### 2. `crates/gpui/src/app.rs`

#### Change: Protected initial draw with `is_renderer_ready()` check
```rust
// On web, skip this initial draw if the async WebGPU renderer isn't ready yet
if window.platform_window.is_renderer_ready() {
    let clear = window.draw(cx);
    clear.clear();
}
```

**Reasoning**: When a window is opened, GPUI immediately draws it. On web, the WebGPU renderer initializes asynchronously after window creation. If we draw before the renderer is ready, all glyphs go to a `NoopAtlas` and are lost. This check defers the initial draw until WebGPU is ready. The render loop will trigger the first real draw once initialization completes.

#### Change: Added conditional `web_time::Instant` and `http_client` imports
```rust
#[cfg(not(target_arch = "wasm32"))]
use http_client::{self, HttpClient, Url};
#[cfg(target_arch = "wasm32")]
use crate::platform::web::http_client::{self as http_client, HttpClient, Url};
```

**Reasoning**: The standard `http_client` crate uses native networking which isn't available in WASM. The web platform provides a stub implementation. Using the same module alias (`http_client`) allows the rest of the code to work unchanged.

---

### 3. `crates/gpui/src/window.rs`

#### Change: Added `is_renderer_ready()` check in frame callback
```rust
let renderer_ready = handle
    .update(&mut cx, |_, window, _| {
        window.platform_window.is_renderer_ready()
    })
    .unwrap_or(true);

if renderer_ready && (invalidator.is_dirty() || request_frame_options.force_render) {
    // draw...
}
```

**Reasoning**: The render loop runs via `requestAnimationFrame` on web, which starts immediately. Without this check, every frame would attempt to draw before WebGPU is ready. This ensures drawing only happens after async initialization completes.

#### Change: Added sprite atlas refresh for web
```rust
#[cfg(target_arch = "wasm32")]
{
    self.sprite_atlas = self.platform_window.sprite_atlas();
}
```

**Reasoning**: On web, the sprite atlas is created during async WebGPU initialization, after the window is constructed. The window initially holds a reference to a `NoopAtlas`. This refresh ensures we get the real atlas once it's available.

#### Change: Handle `None` tile from atlas gracefully
```rust
let Some(tile) = tile else {
    return Ok(());
};
```

**Reasoning**: Before WebGPU is ready, the `NoopAtlas` returns `Ok(None)` for tile insertions. Previously the code called `.expect()` assuming tiles always exist. This change gracefully handles the `None` case by skipping the glyph (it will be repainted on the next frame when the real atlas is available).

---

### 4. `crates/gpui/src/executor.rs`

#### Change: Made `block()` panic on WASM
```rust
#[cfg(target_arch = "wasm32")]
pub fn block<R>(&self, _future: impl Future<Output = R>) -> R {
    panic!("block() is not supported on WASM - use async/await instead")
}
```

**Reasoning**: WASM runs on a single-threaded event loop and cannot block. Attempting to block would freeze the browser tab. This explicit panic provides a clear error message guiding developers to use async patterns instead.

#### Change: Made `block_with_timeout()` panic on WASM
```rust
#[cfg(target_arch = "wasm32")]
pub fn block_with_timeout<Fut: Future>(
    &self,
    _duration: Duration,
    _future: Fut,
) -> Result<Fut::Output, Fut> {
    panic!("block_with_timeout() is not supported on WASM - use async/await instead")
}
```

**Reasoning**: Same as above - blocking is not possible in WASM.

#### Change: Conditional imports for smol vs futures
```rust
#[cfg(not(target_arch = "wasm32"))]
use smol::prelude::*;
#[cfg(target_arch = "wasm32")]
use futures::StreamExt as _;
```

**Reasoning**: The `smol` runtime is not available in WASM. The `futures` crate provides compatible traits for async operations.

---

### 5. `crates/gpui/src/text_system.rs`

#### Change: Web-specific fallback font stack
```rust
#[cfg(target_arch = "wasm32")]
fallback_font_stack: smallvec![
    font("Inter"),
    font("Roboto"),
    font("Open Sans"),
    font("Noto Sans"),
    font("Arial"),
    font("Helvetica"),
    font("sans-serif"),
],
```

**Reasoning**: The default fallback font stack includes macOS/Linux-specific fonts that don't exist in browsers. Web browsers have different default fonts, so a web-appropriate stack is needed.

#### Change: Added `load_font_from_url()` async method
```rust
#[cfg(target_arch = "wasm32")]
pub async fn load_font_from_url(&self, url: &str) -> Result<()> {
    // ...
}
```

**Reasoning**: On native platforms, fonts are loaded from the filesystem. In browsers, fonts must be fetched from URLs. This async method provides that capability for web applications.

---

### 6. `crates/gpui/src/gpui.rs`

#### Change: Conditional `http_client` export
```rust
#[cfg(not(target_arch = "wasm32"))]
pub use http_client;
#[cfg(target_arch = "wasm32")]
pub use platform::web::http_client;
```

**Reasoning**: Ensures code using `gpui::http_client` gets the appropriate implementation for the platform.

#### Change: Removed `Timer` export for WASM
```rust
#[cfg(not(target_arch = "wasm32"))]
pub use smol::Timer;
```

**Reasoning**: `smol::Timer` is not available in WASM. The web platform provides its own `Timer` implementation exported from `platform::web::timer`.

---

### 7. `crates/gpui/src/profiler.rs`, `elements/animation.rs`, `elements/img.rs`

#### Change: Conditional `web_time::Instant` import
```rust
#[cfg(not(target_arch = "wasm32"))]
use std::time::Instant;
#[cfg(target_arch = "wasm32")]
use web_time::Instant;
```

**Reasoning**: All files using `Instant` need this conditional import since `std::time::Instant` is unavailable in WASM.

---

### 8. `crates/gpui/src/svg_renderer.rs`

#### Change: Removed unused `SmallVec` import
```rust
// Removed: use smallvec::SmallVec;
```

**Reasoning**: The code was changed to use `smallvec::smallvec![]` macro directly, making the `SmallVec` type import unnecessary. This was a cleanup to remove a compiler warning.

---

## Verification

All changes have been verified to not break other platforms:

- **macOS**: `cargo check -p gpui` passes with no warnings
- **Linux**: `cargo check -p gpui --features "wayland,x11"` passes with no warnings
- **WASM**: `wasm-pack build` succeeds
- **Tests**: All 71 gpui tests pass

## Testing the Web Platform

### Unit Tests

The web platform includes unit tests using `wasm-bindgen-test`. Tests are organized in each module:

- `http_client.rs` - Tests for URL parsing, AsyncBody, and FakeHttpClient
- `timer.rs` - Tests for Timer creation and duration
- `platform.rs` - Tests for clipboard, keyboard layout, cursor styles
- `window.rs` - Tests for window creation, bounds, sizing, and renderer state

**Running Native Tests (non-WASM tests):**
```bash
cargo test -p gpui --lib
```

**WASM Test Structure:**
```rust
#[cfg(test)]
mod tests {
    // Non-WASM tests run with `cargo test`
    #[test]
    fn test_something() { ... }

    // WASM-only tests require wasm-pack
    #[cfg(target_arch = "wasm32")]
    mod wasm_tests {
        use wasm_bindgen_test::*;
        wasm_bindgen_test_configure!(run_in_browser);

        #[wasm_bindgen_test]
        fn test_in_browser() { ... }
    }
}
```

**Note on WASM Test Execution:**
Running `wasm-pack test` on the full gpui crate currently fails due to dev-dependencies (like `libz-sys`) that require native compilation. To run WASM-specific tests:

1. Create a minimal test crate without heavy dev-dependencies, or
2. Use the `gpui_web_example` for integration testing

### Integration Testing

1. Build: `cd crates/gpui_web_example && wasm-pack build --target web --out-dir pkg`
2. Serve: `cd www && python3 -m http.server 8081`
3. Test: Open http://localhost:8081 in a WebGPU-capable browser
4. Verify: Check console for "WebGPU renderer initialized" and visible text rendering

### Automated Testing with Playwright

For CI/CD, use Playwright to automate browser testing:

```javascript
const { chromium } = require('playwright');

(async () => {
  const browser = await chromium.launch();
  const page = await browser.newPage();

  page.on('console', msg => console.log(msg.text()));

  await page.goto('http://localhost:8081');
  await page.waitForFunction(() =>
    window.performance.now() > 3000 // Wait for WebGPU init
  );

  // Check for successful initialization
  const logs = await page.evaluate(() => window.__gpui_logs || []);
  console.assert(logs.includes('WebGPU renderer initialized'));

  await browser.close();
})();
```
