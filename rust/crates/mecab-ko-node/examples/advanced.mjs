// Advanced usage patterns for @mecab-ko/node
// Run with: node examples/advanced.mjs

import { Mecab, getVersion } from '../index.js';
import { performance } from 'perf_hooks';

console.log(`MeCab-Ko Node.js v${getVersion()}`);
console.log('Advanced Usage Examples\n');

const mecab = new Mecab();

// 1. Text normalization and analysis
console.log('=== 1. Text Normalization ===');
const texts = [
    '   공백이    많은     텍스트   ',
    '특수문자!!@#$포함%%텍스트',
    'Mixed한글English123숫자'
];

texts.forEach(text => {
    const nouns = mecab.nouns(text.trim());
    console.log(`"${text}" -> Nouns: [${nouns.join(', ')}]`);
});
console.log('');

// 2. Batch processing with chunking
console.log('=== 2. Batch Processing ===');
async function batchAnalyze(texts, chunkSize = 10) {
    const results = [];
    for (let i = 0; i < texts.length; i += chunkSize) {
        const chunk = texts.slice(i, i + chunkSize);
        const chunkResults = chunk.map(text => ({
            text,
            tokens: mecab.tokenize(text),
            nouns: mecab.nouns(text)
        }));
        results.push(...chunkResults);
        await Promise.resolve(); // Yield to event loop
    }
    return results;
}

const largeBatch = Array(50).fill(null).map((_, i) =>
    `문장 번호 ${i + 1}입니다.`
);

const start = performance.now();
const batchResults = await batchAnalyze(largeBatch);
const elapsed = performance.now() - start;

console.log(`Processed ${batchResults.length} texts in ${elapsed.toFixed(2)}ms`);
console.log(`Average: ${(elapsed / batchResults.length).toFixed(2)}ms per text`);
console.log('');

// 3. POS filtering
console.log('=== 3. POS Filtering ===');
const sentence = '빠른 갈색 여우가 게으른 개를 뛰어넘습니다.';
const tokens = mecab.tokenize(sentence);

// Filter by POS categories
const nouns = tokens.filter(t => t.pos.startsWith('NN'));
const verbs = tokens.filter(t => t.pos.startsWith('VV') || t.pos.startsWith('VA'));
const particles = tokens.filter(t => t.pos.startsWith('J'));

console.log(`Text: ${sentence}`);
console.log(`Nouns: [${nouns.map(t => t.surface).join(', ')}]`);
console.log(`Verbs: [${verbs.map(t => t.surface).join(', ')}]`);
console.log(`Particles: [${particles.map(t => t.surface).join(', ')}]`);
console.log('');

// 4. Custom tokenization pipeline
console.log('=== 4. Custom Pipeline ===');
class KoreanAnalyzer {
    constructor() {
        this.mecab = new Mecab();
        this.stopwords = new Set(['은', '는', '이', '가', '을', '를']);
    }

    analyze(text) {
        const tokens = this.mecab.tokenize(text);

        return {
            original: text,
            tokens: tokens,
            contentWords: tokens.filter(t =>
                !this.stopwords.has(t.surface) &&
                (t.pos.startsWith('NN') || t.pos.startsWith('VV'))
            ),
            entities: tokens.filter(t => t.pos === 'NNP'),
            statistics: {
                totalTokens: tokens.length,
                uniqueTokens: new Set(tokens.map(t => t.surface)).size,
                avgTokenLength: tokens.reduce((sum, t) => sum + t.surface.length, 0) / tokens.length
            }
        };
    }
}

const analyzer = new KoreanAnalyzer();
const analysis = analyzer.analyze('서울은 대한민국의 수도이며 경제 중심지입니다.');

console.log('Original:', analysis.original);
console.log('Content words:', analysis.contentWords.map(t => t.surface).join(', '));
console.log('Entities:', analysis.entities.map(t => t.surface).join(', '));
console.log('Statistics:', analysis.statistics);
console.log('');

// 5. Stream-like processing
console.log('=== 5. Stream Processing ===');
async function* textStream(texts) {
    for (const text of texts) {
        await new Promise(resolve => setTimeout(resolve, 10)); // Simulate async source
        yield text;
    }
}

async function processStream(stream) {
    const results = [];
    for await (const text of stream) {
        const nouns = mecab.nouns(text);
        results.push({ text, nouns });
    }
    return results;
}

const streamTexts = [
    '첫 번째 스트림 데이터',
    '두 번째 스트림 데이터',
    '세 번째 스트림 데이터'
];

const streamResults = await processStream(textStream(streamTexts));
streamResults.forEach((result, idx) => {
    console.log(`${idx + 1}. ${result.text} -> [${result.nouns.join(', ')}]`);
});
console.log('');

// 6. Performance comparison
console.log('=== 6. Performance Comparison ===');
const perfText = '한국어 형태소 분석은 자연어 처리의 기본 단계입니다.';
const iterations = 1000;

// Tokenize performance
let start1 = performance.now();
for (let i = 0; i < iterations; i++) {
    mecab.tokenize(perfText);
}
let elapsed1 = performance.now() - start1;

// Morphs performance
let start2 = performance.now();
for (let i = 0; i < iterations; i++) {
    mecab.morphs(perfText);
}
let elapsed2 = performance.now() - start2;

// Nouns performance
let start3 = performance.now();
for (let i = 0; i < iterations; i++) {
    mecab.nouns(perfText);
}
let elapsed3 = performance.now() - start3;

console.log(`Iterations: ${iterations}`);
console.log(`tokenize(): ${elapsed1.toFixed(2)}ms (${(elapsed1/iterations).toFixed(3)}ms avg)`);
console.log(`morphs():   ${elapsed2.toFixed(2)}ms (${(elapsed2/iterations).toFixed(3)}ms avg)`);
console.log(`nouns():    ${elapsed3.toFixed(2)}ms (${(elapsed3/iterations).toFixed(3)}ms avg)`);
console.log('');

// 7. MeCab format parsing
console.log('=== 7. MeCab Format Parsing ===');
const mecabOutput = mecab.parse('형태소 분석 결과');
console.log('Raw MeCab output:');
console.log(mecabOutput);

// Parse the output back
const lines = mecabOutput.trim().split('\n');
const parsedTokens = lines
    .filter(line => line !== 'EOS')
    .map(line => {
        const [surface, features] = line.split('\t');
        const featureList = features.split(',');
        return {
            surface,
            pos: featureList[0],
            features: featureList
        };
    });

console.log('Parsed tokens:');
parsedTokens.forEach(token => {
    console.log(`  ${token.surface} (${token.pos})`);
});
console.log('');

console.log('All examples completed successfully! ✓');
