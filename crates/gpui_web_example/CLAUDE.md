# GPUI Web Example - Development Guide

## Building WASM

### Standard Build

From the `crates/gpui_web_example` directory:

```bash
wasm-pack build --target web --out-dir www/pkg
```

This builds directly to `www/pkg/` which is served by the HTTP server.

### Serving

```bash
cd www && python3 -m http.server 8081
```

Then open http://localhost:8081 in a WebGPU-capable browser (Chrome/Edge).

## Build Cache Issues

WASM builds can appear to succeed while using stale cached artifacts. This causes confusing behavior where code changes don't appear in the browser.

### Symptoms of Cache Issues

- Build completes in < 1 second (should take 8-30+ seconds for real compilation)
- Console output shows "Finished ... in 0.13s" instead of "Compiling gpui ..."
- Code changes don't appear in browser even after rebuild
- Debug logs you added don't show up

### Solution: Force Fresh Build

When you suspect caching issues, perform a clean rebuild:

```bash
# From crates/gpui_web_example directory

# 1. Delete WASM output directories
rm -rf www/pkg pkg

# 2. Delete WASM target cache (optional but thorough)
rm -rf ../../target/wasm32-unknown-unknown

# 3. Rebuild
wasm-pack build --target web --out-dir www/pkg
```

A fresh build should show:
- "Compiling gpui v0.2.2" in the output
- Build time of 8-30+ seconds (depending on your machine)

### Alternative: Touch Source Files

If you only changed files in the web platform module:

```bash
touch ../../gpui/src/platform/web/window.rs
wasm-pack build --target web --out-dir www/pkg
```

### Browser Cache

The browser may also cache the WASM file. To force a fresh load:

1. Open DevTools (F12)
2. Right-click the refresh button
3. Select "Empty Cache and Hard Reload"

Or add a cache-busting query parameter: `http://localhost:8081?v=2`

## Directory Structure

```
crates/gpui_web_example/
  ├── src/
  │   └── lib.rs          # WASM entry point
  ├── www/
  │   ├── index.html      # HTML shell
  │   └── pkg/            # wasm-pack output (gitignored)
  ├── pkg/                # Alternative output location (gitignored)
  └── Cargo.toml
```

**Important:** Always build with `--out-dir www/pkg` to ensure the HTTP server serves the latest build. If you build without `--out-dir`, output goes to `pkg/` which is NOT served.

## Testing with Playwright

```bash
# Navigate to the page
await page.goto('http://localhost:8081');

# Check for successful initialization
# Look for "WebGPU renderer initialized" in console logs

# Inspect DOM
await page.evaluate(() => document.body.innerHTML);
```

## Debugging Tips

1. **Check console logs** - The renderer logs key events like "WebGPU renderer initialized"
2. **Verify WASM file timestamp** - `ls -la www/pkg/gpui_web_example_bg.wasm`
3. **Check file size** - A complete build should be ~4MB; a corrupted/partial build may be smaller
4. **Use `cargo clean`** - For stubborn cache issues: `cargo clean && wasm-pack build --target web --out-dir www/pkg`
