import assert from 'node:assert';
import test from 'node:test';

test('mecab-ko-node can be imported', async () => {
  const mecab = await import('mecab-ko-node');
  assert.ok(mecab);
});
