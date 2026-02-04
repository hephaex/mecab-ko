// ESM usage example for @mecab-ko/node
// Run with: node examples/esm-example.mjs

import { Mecab, getVersion } from '../index.js';

console.log('MeCab-Ko Node.js Bindings (ESM)');
console.log('Version:', getVersion());
console.log('');

try {
    const mecab = new Mecab();

    // Example text
    const text = '인공지능과 자연어 처리는 미래 기술입니다.';
    console.log(`Analyzing: "${text}"\n`);

    // Tokenize
    console.log('=== Tokenize ===');
    const tokens = mecab.tokenize(text);
    tokens.forEach(token => {
        console.log(`${token.surface} [${token.pos}]`);
    });

    // Extract morphemes
    console.log('\n=== Morphemes ===');
    const morphs = mecab.morphs(text);
    console.log(morphs.join(' | '));

    // Extract nouns
    console.log('\n=== Nouns ===');
    const nouns = mecab.nouns(text);
    console.log(nouns.join(', '));

    // POS tagging
    console.log('\n=== POS Tags ===');
    const pos = mecab.pos(text);
    pos.forEach(([surface, tag]) => {
        console.log(`${surface}/${tag}`);
    });

    // MeCab format output
    console.log('\n=== MeCab Format ===');
    const parsed = mecab.parse(text);
    console.log(parsed);

    // Batch processing with promises
    console.log('=== Async Batch Processing ===');
    const texts = [
        '첫 번째 문장입니다.',
        '두 번째 문장입니다.',
        '세 번째 문장입니다.'
    ];

    const results = await Promise.all(
        texts.map(async (t) => ({
            text: t,
            nouns: mecab.nouns(t),
            morphCount: mecab.morphs(t).length
        }))
    );

    results.forEach((result, idx) => {
        console.log(`${idx + 1}. ${result.text}`);
        console.log(`   Nouns: [${result.nouns.join(', ')}]`);
        console.log(`   Morphs: ${result.morphCount}`);
    });

} catch (error) {
    console.error('Error:', error.message);
    process.exit(1);
}
