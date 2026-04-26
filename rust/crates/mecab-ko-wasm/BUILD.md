# Building mecab-ko-wasm

This guide explains how to build the WebAssembly bindings for MeCab-Ko.

## Prerequisites

### Required

- Rust 1.80+ with `wasm32-unknown-unknown` target
- wasm-bindgen-cli (optional but recommended)

```bash
# Install Rust target for WASM
rustup target add wasm32-unknown-unknown

# Install wasm-bindgen-cli (optional)
cargo install wasm-bindgen-cli
```

### Recommended

- wasm-pack (for production builds)

```bash
cargo install wasm-pack
```

## Build Methods

### Method 1: Using wasm-pack (Recommended)

wasm-pack is the easiest way to build and package the WASM module for npm.

#### For Web Browsers

```bash
wasm-pack build --target web
```

This creates a `pkg/` directory with:
- `mecab_ko_wasm.js` - JavaScript bindings
- `mecab_ko_wasm_bg.wasm` - WebAssembly binary
- `mecab_ko_wasm.d.ts` - TypeScript definitions
- `package.json` - npm package configuration

#### For Node.js

```bash
wasm-pack build --target nodejs
```

#### For Bundlers (webpack, rollup, etc.)

```bash
wasm-pack build --target bundler
```

#### With Optimizations

```bash
# Optimize for size
wasm-pack build --target web --release

# With additional optimizations
wasm-pack build --target web --release -- --features wee_alloc
```

### Method 2: Using cargo build

For development and testing without wasm-pack:

```bash
# Build the library
cargo build --target wasm32-unknown-unknown

# Build with release optimizations
cargo build --target wasm32-unknown-unknown --release
```

The WASM binary will be at:
- Debug: `target/wasm32-unknown-unknown/debug/mecab_ko_wasm.wasm`
- Release: `target/wasm32-unknown-unknown/release/mecab_ko_wasm.wasm`

### Method 3: Using wasm-bindgen (Manual)

For more control over the build process:

```bash
# Build the library
cargo build --target wasm32-unknown-unknown --release

# Generate JavaScript bindings
wasm-bindgen \
  --target web \
  --out-dir pkg \
  target/wasm32-unknown-unknown/release/mecab_ko_wasm.wasm
```

## Testing

### Run Tests in Node.js

```bash
wasm-pack test --node
```

### Run Tests in Browser

```bash
# Firefox
wasm-pack test --headless --firefox

# Chrome
wasm-pack test --headless --chrome
```

### Run Rust Tests

```bash
cargo test
```

## Publishing to npm

### Build for Production

```bash
# Build optimized package
wasm-pack build --target bundler --release

# Test the package locally
cd pkg
npm link

# In your test project
npm link mecab-ko-wasm
```

### Publish to npm

```bash
# Build the package
wasm-pack build --target bundler --release

# Publish (requires npm account)
wasm-pack publish
```

Or manually:

```bash
cd pkg
npm publish
```

## Optimization Tips

### 1. Enable LTO in Release Mode

Already configured in workspace `Cargo.toml`:

```toml
[profile.release]
lto = true
codegen-units = 1
```

### 2. Use wee_alloc

Smaller memory allocator for WASM (already optional in dependencies):

```bash
wasm-pack build --release -- --features wee_alloc
```

### 3. Optimize WASM Size

```bash
# Install wasm-opt (from binaryen)
# Ubuntu/Debian:
sudo apt install binaryen

# macOS:
brew install binaryen

# Optimize the WASM binary
wasm-opt -Oz -o output.wasm input.wasm
```

### 4. Strip Debug Info

```toml
[profile.release]
strip = true  # Already configured
```

## Troubleshooting

### "error: can't find crate for `std`"

Install the wasm32 target:

```bash
rustup target add wasm32-unknown-unknown
```

### Large WASM Binary Size

1. Build in release mode: `--release`
2. Enable LTO (already configured)
3. Use `wasm-opt` for additional optimization
4. Enable `wee_alloc` feature

### Import Errors in Browser

Make sure you're using the correct target:
- `--target web` for ES modules in browser
- `--target bundler` for webpack/rollup
- `--target nodejs` for Node.js

### TypeScript Errors

Regenerate bindings with wasm-pack to get updated `.d.ts` files.

## Development Workflow

```bash
# 1. Make changes to src/lib.rs

# 2. Test
cargo test
wasm-pack test --node

# 3. Build
wasm-pack build --target web --dev

# 4. Test in browser
# Create a simple HTML file and open it

# 5. Build for release
wasm-pack build --target bundler --release
```

## Size Reference

Typical build sizes (will vary based on dictionary size):

- Debug build: ~500KB - 2MB
- Release build: ~200KB - 1MB
- Release + wasm-opt: ~150KB - 800KB

Note: Actual size depends heavily on the dictionary data included.

## Further Reading

- [wasm-pack documentation](https://rustwasm.github.io/wasm-pack/)
- [wasm-bindgen guide](https://rustwasm.github.io/wasm-bindgen/)
- [Rust and WebAssembly book](https://rustwasm.github.io/book/)
