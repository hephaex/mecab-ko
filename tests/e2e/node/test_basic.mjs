import assert from 'node:assert';
import { describe, it, before, test } from 'node:test';

// Helper: try to create a Mecab instance, returns null if module unavailable or no dict
async function tryCreateMecab() {
  try {
    const mod = await import('mecab-ko-node');
    const mecab = new mod.Mecab();
    return { mod, mecab };
  } catch {
    return null;
  }
}

// -- Import tests (no dict needed) --

test('mecab-ko-node module can be imported', async (t) => {
  try {
    const mecab = await import('mecab-ko-node');
    assert.ok(mecab);
  } catch (e) {
    t.skip('mecab-ko-node not available: ' + e.message);
  }
});

test('getVersion returns semver string', async (t) => {
  try {
    const mod = await import('mecab-ko-node');
    const version = mod.getVersion();
    assert.ok(typeof version === 'string');
    assert.match(version, /^\d+\.\d+/);
  } catch (e) {
    t.skip('mecab-ko-node not available: ' + e.message);
  }
});

test('Mecab.withDict throws on invalid path', async (t) => {
  try {
    const mod = await import('mecab-ko-node');
    assert.throws(() => {
      mod.Mecab.withDict('/nonexistent_dict_xyz');
    });
  } catch (e) {
    t.skip('mecab-ko-node not available: ' + e.message);
  }
});

// -- Dict-dependent tests --

describe('with dictionary', async () => {
  let mod, mecab;

  before(async () => {
    const result = await tryCreateMecab();
    if (!result) {
      return;
    }
    mod = result.mod;
    mecab = result.mecab;
  });

  it('Mecab constructor succeeds', (t) => {
    if (!mecab) { t.skip('no dictionary'); return; }
    assert.ok(mecab);
  });

  it('tokenize returns array of tokens', (t) => {
    if (!mecab) { t.skip('no dictionary'); return; }
    const tokens = mecab.tokenize('안녕하세요');
    assert.ok(Array.isArray(tokens));
    assert.ok(tokens.length > 0);
  });

  it('token has expected shape', (t) => {
    if (!mecab) { t.skip('no dictionary'); return; }
    const tokens = mecab.tokenize('안녕하세요');
    const token = tokens[0];
    assert.ok(typeof token.surface === 'string');
  });

  it('morphs returns string array', (t) => {
    if (!mecab) { t.skip('no dictionary'); return; }
    const result = mecab.morphs('안녕하세요');
    assert.ok(Array.isArray(result));
    assert.ok(result.length > 0);
    assert.ok(result.every(m => typeof m === 'string'));
  });

  it('nouns returns string array', (t) => {
    if (!mecab) { t.skip('no dictionary'); return; }
    const result = mecab.nouns('대한민국의 수도');
    assert.ok(Array.isArray(result));
    assert.ok(result.every(n => typeof n === 'string'));
  });

  it('pos returns nested arrays', (t) => {
    if (!mecab) { t.skip('no dictionary'); return; }
    const result = mecab.pos('안녕하세요');
    assert.ok(Array.isArray(result));
    assert.ok(result.length > 0);
    for (const pair of result) {
      assert.ok(Array.isArray(pair));
      assert.strictEqual(pair.length, 2);
    }
  });

  it('parse contains EOS', (t) => {
    if (!mecab) { t.skip('no dictionary'); return; }
    const result = mecab.parse('안녕하세요');
    assert.ok(typeof result === 'string');
    assert.ok(result.includes('EOS'));
  });

  it('parse has tab-separated fields', (t) => {
    if (!mecab) { t.skip('no dictionary'); return; }
    const result = mecab.parse('안녕하세요');
    const lines = result.trim().split('\n').filter(l => l && l !== 'EOS');
    assert.ok(lines.length > 0);
    assert.ok(lines.every(l => l.includes('\t')));
  });

  it('empty input does not crash', (t) => {
    if (!mecab) { t.skip('no dictionary'); return; }
    const morphs = mecab.morphs('');
    assert.ok(Array.isArray(morphs));
    const parsed = mecab.parse('');
    assert.ok(typeof parsed === 'string');
  });
});
