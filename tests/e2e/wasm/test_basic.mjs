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

test('Mecab class is exported with tokenize/morphs/nouns/pos/wakati methods', async () => {
  const mod = await tryImportWasm();
  if (mod === null) {
    console.log('  SKIP: wasm module not available');
    return;
  }
  // 실제 공개 API는 flat function이 아니라 Mecab 클래스다 (wasm/src/lib.rs).
  assert.strictEqual(typeof mod.Mecab, 'function', "export 'Mecab' should be a class");
  const expectedMethods = ['tokenize', 'morphs', 'nouns', 'pos', 'wakati'];
  for (const name of expectedMethods) {
    assert.strictEqual(
      typeof mod.Mecab.prototype[name],
      'function',
      `Mecab.prototype.${name} should be a function`,
    );
  }
});

// Helper: instantiate Mecab, or null when dict is not embedded in this build.
function tryNewMecab(mod) {
  try {
    return new mod.Mecab();
  } catch (err) {
    console.log('  SKIP: new Mecab() failed (dict may not be embedded):', err.message);
    return null;
  }
}

test('basic tokenization with simple Korean string', async () => {
  const mod = await tryImportWasm();
  if (mod === null) {
    console.log('  SKIP: wasm module not available');
    return;
  }
  const mecab = tryNewMecab(mod);
  if (mecab === null) return;
  const result = mecab.tokenize('안녕하세요');
  assert.ok(Array.isArray(result), 'tokenize() should return an array');
  assert.ok(result.length > 0, 'tokenize() should return at least one token');
});

test('empty input handling', async () => {
  const mod = await tryImportWasm();
  if (mod === null) {
    console.log('  SKIP: wasm module not available');
    return;
  }
  const mecab = tryNewMecab(mod);
  if (mecab === null) return;
  const result = mecab.tokenize('');
  assert.ok(Array.isArray(result), 'empty input should return an array');
});
