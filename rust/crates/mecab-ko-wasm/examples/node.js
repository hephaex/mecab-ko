/**
 * MeCab-Ko WASM Node.js Example
 *
 * Usage:
 *   1. Build the WASM module: wasm-pack build --target nodejs --out-dir pkg-node
 *   2. Run this example: node examples/node.js
 */

const { Mecab } = require('../pkg-node/mecab_ko_wasm.js');

async function main() {
    console.log('='.repeat(60));
    console.log('MeCab-Ko WASM Node.js Demo');
    console.log('='.repeat(60));

    try {
        const mecab = new Mecab();

        const testTexts = [
            "안녕하세요. 형태소 분석기입니다.",
            "한국어 자연어 처리를 위한 형태소 분석",
            "Rust로 작성된 고성능 토크나이저",
            "2024년 신조어도 분석할 수 있습니다."
        ];

        for (const text of testTexts) {
            console.log('\n' + '-'.repeat(60));
            console.log('Input:', text);
            console.log('-'.repeat(60));

            // Tokenization
            console.log('\n[Tokenization]');
            const tokens = mecab.tokenize(text);
            tokens.forEach(token => {
                console.log(`  ${token.surface.padEnd(10)} ${token.pos.padEnd(8)} (${token.start}-${token.end})`);
            });

            // Morphemes
            console.log('\n[Morphemes]');
            const morphs = mecab.morphs(text);
            console.log('  ' + morphs.join(' | '));

            // Nouns
            console.log('\n[Nouns]');
            const nouns = mecab.nouns(text);
            console.log('  ' + (nouns.length > 0 ? nouns.join(', ') : '(none)'));

            // POS Tags
            console.log('\n[POS Tags]');
            const pos = mecab.pos(text);
            pos.forEach(([surface, tag]) => {
                console.log(`  ${surface} -> ${tag}`);
            });
        }

        // Performance test
        console.log('\n' + '='.repeat(60));
        console.log('Performance Test');
        console.log('='.repeat(60));

        const longText = testTexts.join(' ').repeat(100);
        const iterations = 100;

        const start = Date.now();
        for (let i = 0; i < iterations; i++) {
            mecab.tokenize(longText);
        }
        const elapsed = Date.now() - start;

        console.log(`\nProcessed ${iterations} iterations in ${elapsed}ms`);
        console.log(`Average: ${(elapsed / iterations).toFixed(2)}ms per tokenization`);
        console.log(`Text length: ${longText.length} characters`);

    } catch (error) {
        console.error('Error:', error);
        process.exit(1);
    }
}

main();
