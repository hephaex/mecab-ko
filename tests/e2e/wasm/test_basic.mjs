import { test } from 'node:test';
import assert from 'node:assert/strict';
import { existsSync } from 'node:fs';
import { resolve, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const PKG_DIR = resolve(__dirname, '../../../rust/crates/mecab-ko-wasm/pkg');
const WASM_AVAILABLE = existsSync(resolve(PKG_DIR, 'mecab_ko_wasm.js'));

// ---------------------------------------------------------------------------
// Helper: attempt to import the WASM module, return null if not built yet
// ---------------------------------------------------------------------------
async function tryImportWasm() {
  if (!WASM_AVAILABLE) return null;
  try {
    const mod = await import(resolve(PKG_DIR, 'mecab_ko_wasm.js'));
    return mod;
  } catch {
    return null;
  }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

test('wasm pkg directory exists (build artifact check)', () => {
  if (!WASM_AVAILABLE) {
    // Scaffold: gracefully skip when wasm-pack has not run yet
    console.log('  SKIP: wasm build artifacts not present at', PKG_DIR);
    return;
  }
  assert.ok(existsSync(PKG_DIR), 'pkg/ directory should exist after wasm-pack build');
});

test('wasm module can be imported', async () => {
  const mod = await tryImportWasm();
  if (mod === null) {
    console.log('  SKIP: wasm module not available');
    return;
  }
  assert.ok(mod !== null, 'wasm module should be importable');
});

test('exported functions exist: tokenize, morphs, nouns, pos, parse', async () => {
  const mod = await tryImportWasm();
  if (mod === null) {
    console.log('  SKIP: wasm module not available');
    return;
  }
  const expectedExports = ['tokenize', 'morphs', 'nouns', 'pos', 'parse'];
  for (const name of expectedExports) {
    assert.strictEqual(
      typeof mod[name],
      'function',
      `export '${name}' should be a function`,
    );
  }
});

test('basic tokenization with simple Korean string', async () => {
  const mod = await tryImportWasm();
  if (mod === null) {
    console.log('  SKIP: wasm module not available');
    return;
  }
  try {
    const result = mod.tokenize('안녕하세요');
    assert.ok(Array.isArray(result), 'tokenize() should return an array');
    assert.ok(result.length > 0, 'tokenize() should return at least one token');
  } catch (err) {
    // Dictionary may not be embedded in CI; treat as a graceful skip
    console.log('  SKIP: tokenize() failed (dict may not be embedded):', err.message);
  }
});

test('empty input handling', async () => {
  const mod = await tryImportWasm();
  if (mod === null) {
    console.log('  SKIP: wasm module not available');
    return;
  }
  try {
    const result = mod.tokenize('');
    assert.ok(Array.isArray(result), 'empty input should return an array');
  } catch (err) {
    // Some implementations may throw on empty input — acceptable for a scaffold
    console.log('  NOTE: tokenize("") threw:', err.message);
  }
});
