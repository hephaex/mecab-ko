# Build Guide for mecab-ko-node

This guide explains how to build and test the Node.js bindings for MeCab-Ko.

## Prerequisites

### System Requirements

- **Rust**: 1.80 or higher
- **Node.js**: 16 or higher
- **npm** or **yarn** or **pnpm**

### Install Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup update
```

### Install Node.js

Use [nvm](https://github.com/nvm-sh/nvm) (recommended):

```bash
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.0/install.sh | bash
nvm install 20
nvm use 20
```

Or download from [nodejs.org](https://nodejs.org/).

## Building

### 1. Install Dependencies

```bash
cd rust/crates/mecab-ko-node
npm install
```

This will install:
- `@napi-rs/cli` - Build tooling
- `typescript` - TypeScript compiler
- `vitest` - Test framework
- Other dev dependencies

### 2. Build the Native Module

#### Debug Build

```bash
npm run build:debug
```

This creates a debug build at `target/debug/libmecab_ko_node.{so,dylib,dll}`.

#### Release Build

```bash
npm run build
```

This creates an optimized release build with:
- Link-time optimization (LTO)
- Single codegen unit
- Stripped symbols
- Maximum optimization level

The output will be in `target/release/`.

### 3. Run Tests

#### Rust Tests

```bash
cargo test
```

Runs the Rust unit tests in `src/lib.rs`.

#### Node.js Tests

First, build the module:

```bash
npm run build:debug
```

Then run tests:

```bash
npm test
```

Or in watch mode:

```bash
npm run test:watch
```

## Development Workflow

### Quick Iteration

For fast iteration during development:

1. Make changes to `src/lib.rs`
2. Run `npm run build:debug`
3. Run `npm test`

### Check Code Quality

```bash
# Format code
cargo fmt

# Run linter
cd ../.. # Go to workspace root
cargo clippy -p mecab-ko-node

# Check compilation
cargo check -p mecab-ko-node
```

### Testing TypeScript Types

```bash
# Compile TypeScript example
npx tsc examples/typescript-example.ts

# Run it
node examples/typescript-example.js
```

## Cross-Platform Builds

### Build for Current Platform

```bash
npm run build
```

### Build for All Platforms (CI)

The napi-rs CLI supports building for multiple platforms:

```bash
# Build for specific target
npm run build -- --target x86_64-apple-darwin

# Build universal binary for macOS
npm run build -- --target universal-apple-darwin
```

Supported targets:
- `x86_64-apple-darwin` (macOS Intel)
- `aarch64-apple-darwin` (macOS Apple Silicon)
- `x86_64-pc-windows-msvc` (Windows x64)
- `x86_64-unknown-linux-gnu` (Linux x64 glibc)
- `aarch64-unknown-linux-gnu` (Linux ARM64 glibc)
- `x86_64-unknown-linux-musl` (Linux x64 musl)
- `aarch64-unknown-linux-musl` (Linux ARM64 musl)

## Publishing

### Pre-publish Checklist

1. Update version in `Cargo.toml` and `package.json`
2. Update `CHANGELOG.md`
3. Run all tests
4. Build release version
5. Test the package locally

### Prepare for Publishing

```bash
npm run prepublishOnly
```

This runs `napi prepublish` which:
- Validates the package
- Generates platform-specific packages
- Prepares artifacts

### Publish to npm

```bash
npm publish
```

Or for first-time publish:

```bash
npm publish --access public
```

## Troubleshooting

### Build Fails with "napi-build not found"

Make sure you have run `npm install` first.

### Rust Version Too Old

```bash
rustup update
rustc --version  # Should be 1.80 or higher
```

### Node Module Won't Load

1. Rebuild the module:
   ```bash
   npm rebuild
   ```

2. Check that the binary exists:
   ```bash
   ls -la *.node
   ```

3. Check Node.js architecture matches the build:
   ```bash
   node -p process.arch
   ```

### Tests Fail

1. Make sure you built the module first:
   ```bash
   npm run build:debug
   ```

2. Check that dependencies are installed:
   ```bash
   npm install
   ```

3. Run tests with verbose output:
   ```bash
   npm test -- --reporter=verbose
   ```

### Clippy Errors in Dependencies

The `mecab-ko-node` crate has relaxed lints for FFI code. To check only this crate:

```bash
cd rust/crates/mecab-ko-node
cargo clippy --lib
```

## Performance Tips

### Optimization Levels

The release profile is already optimized, but you can experiment:

```toml
[profile.release]
opt-level = 3        # Maximum optimization
lto = "fat"          # Full LTO
codegen-units = 1    # Single codegen unit
```

### Profiling

Use `cargo flamegraph`:

```bash
cargo install flamegraph
cargo flamegraph --bench tokenizer_bench
```

Or use Node.js profiler:

```bash
node --prof examples/basic.js
node --prof-process isolate-*.log > processed.txt
```

## CI/CD Integration

### GitHub Actions Example

```yaml
name: Build

on: [push, pull_request]

jobs:
  build:
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v3
      - uses: actions/setup-node@v3
        with:
          node-version: '20'
      - uses: dtolnay/rust-toolchain@stable
      - run: cd rust/crates/mecab-ko-node && npm install
      - run: cd rust/crates/mecab-ko-node && npm run build
      - run: cd rust/crates/mecab-ko-node && npm test
```

## Additional Resources

- [N-API Documentation](https://nodejs.org/api/n-api.html)
- [napi-rs Documentation](https://napi.rs/)
- [MeCab-Ko Main Repository](https://github.com/hephaex/mecab-ko)
- [Rust Book](https://doc.rust-lang.org/book/)

## Getting Help

- Open an issue: [GitHub Issues](https://github.com/hephaex/mecab-ko/issues)
- Discussions: [GitHub Discussions](https://github.com/hephaex/mecab-ko/discussions)
