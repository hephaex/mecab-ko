// TypeScript usage example for @mecab-ko/node
// Compile with: tsc examples/typescript-example.ts
// Run with: node examples/typescript-example.js

import { Mecab, Token, getVersion } from '../index.js';

interface AnalysisResult {
    text: string;
    tokens: Token[];
    morphs: string[];
    nouns: string[];
    posTagged: string[][];
}

async function analyzeText(mecab: Mecab, text: string): Promise<AnalysisResult> {
    return {
        text,
        tokens: mecab.tokenize(text),
        morphs: mecab.morphs(text),
        nouns: mecab.nouns(text),
        posTagged: mecab.pos(text)
    };
}

async function main(): Promise<void> {
    console.log(`MeCab-Ko Node.js v${getVersion()}\n`);

    const mecab = new Mecab();

    // Single text analysis
    const text = '자연어 처리는 인공지능의 핵심 기술입니다.';
    console.log(`Analyzing: "${text}"\n`);

    const result = await analyzeText(mecab, text);

    console.log('Tokens:');
    result.tokens.forEach((token: Token) => {
        console.log(`  ${token.surface} [${token.pos}] @${token.start}-${token.end}`);
        if (token.reading) {
            console.log(`    Reading: ${token.reading}`);
        }
        if (token.lemma) {
            console.log(`    Lemma: ${token.lemma}`);
        }
    });

    console.log(`\nMorphs: ${result.morphs.join(' / ')}`);
    console.log(`Nouns: ${result.nouns.join(', ')}`);

    // Batch processing
    const texts: string[] = [
        '서울은 대한민국의 수도입니다.',
        '기계 학습은 AI의 한 분야입니다.',
        '형태소 분석은 중요한 전처리 과정입니다.'
    ];

    console.log('\n=== Batch Processing ===\n');

    const results = await Promise.all(
        texts.map(t => analyzeText(mecab, t))
    );

    results.forEach((r, idx) => {
        console.log(`${idx + 1}. "${r.text}"`);
        console.log(`   Nouns: [${r.nouns.join(', ')}]`);
    });

    // Extract all unique nouns
    const allNouns = new Set<string>();
    results.forEach(r => r.nouns.forEach(n => allNouns.add(n)));
    console.log(`\nUnique nouns: ${Array.from(allNouns).join(', ')}`);

    // POS statistics
    const posStats = new Map<string, number>();
    results.forEach(r => {
        r.tokens.forEach(t => {
            posStats.set(t.pos, (posStats.get(t.pos) || 0) + 1);
        });
    });

    console.log('\nPOS Tag Statistics:');
    Array.from(posStats.entries())
        .sort((a, b) => b[1] - a[1])
        .forEach(([pos, count]) => {
            console.log(`  ${pos}: ${count}`);
        });
}

main().catch((error: Error) => {
    console.error('Error:', error.message);
    process.exit(1);
});
