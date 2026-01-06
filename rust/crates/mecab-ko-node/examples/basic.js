// Basic usage example for @mecab-ko/node
// Run with: node examples/basic.js

const { Mecab, getVersion } = require('../index.js');

console.log('MeCab-Ko Node.js Bindings');
console.log('Version:', getVersion());
console.log('');

try {
    // Create a new Mecab instance
    const mecab = new Mecab();

    // Test texts
    const texts = [
        '안녕하세요',
        '대한민국의 수도는 서울입니다',
        '형태소 분석기',
        '아버지가 방에 들어가신다'
    ];

    console.log('=== Tokenization Examples ===\n');

    texts.forEach(text => {
        console.log(`Text: "${text}"`);

        // Tokenize
        console.log('Tokens:');
        const tokens = mecab.tokenize(text);
        tokens.forEach(token => {
            console.log(`  - ${token.surface} [${token.pos}] (${token.start}:${token.end})`);
        });

        // Morphs
        const morphs = mecab.morphs(text);
        console.log(`Morphs: [${morphs.join(', ')}]`);

        // Nouns
        const nouns = mecab.nouns(text);
        console.log(`Nouns: [${nouns.join(', ')}]`);

        // POS
        const pos = mecab.pos(text);
        console.log('POS pairs:');
        pos.forEach(([surface, tag]) => {
            console.log(`  - ${surface}/${tag}`);
        });

        console.log('');
    });

    console.log('=== Performance Test ===\n');

    const testText = '한국어 형태소 분석은 자연어 처리의 기본입니다.';
    const iterations = 1000;

    console.time('Tokenization');
    for (let i = 0; i < iterations; i++) {
        mecab.tokenize(testText);
    }
    console.timeEnd('Tokenization');

    console.log(`Processed ${iterations} texts`);

} catch (error) {
    console.error('Error:', error.message);
    process.exit(1);
}
