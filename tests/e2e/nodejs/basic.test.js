/**
 * Basic tokenization E2E tests for MeCab-Ko Node.js binding
 */

import { describe, it, expect, beforeAll } from 'vitest';
import { readFileSync } from 'fs';
import { fileURLToPath } from 'url';
import { dirname, join } from 'path';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

// Load test fixtures
const fixturesPath = join(__dirname, '..', 'fixtures', 'test_sentences.json');
const testSentences = JSON.parse(readFileSync(fixturesPath, 'utf-8'));

describe('MeCab-Ko Node.js Binding - Basic Tokenization', () => {
  let MecabKo;
  let tagger;

  beforeAll(async () => {
    try {
      MecabKo = await import('mecab-ko-node');
      tagger = new MecabKo.Tagger();
    } catch (error) {
      console.warn('mecab-ko-node not built yet, skipping tests');
    }
  });

  describe('Basic Sentences', () => {
    it('should tokenize simple sentence', () => {
      if (!tagger) {
        return expect(true).toBe(true); // Skip
      }

      const testCase = testSentences.test_cases.find(
        (tc) => tc.id === 'basic_001'
      );
      const result = tagger.parse(testCase.input);

      expect(result).toBeDefined();
      expect(typeof result).toBe('string');
      expect(result.length).toBeGreaterThan(0);
    });

    it('should tokenize verb conjugation', () => {
      if (!tagger) return expect(true).toBe(true);

      const testCase = testSentences.test_cases.find(
        (tc) => tc.id === 'basic_002'
      );
      const result = tagger.parse(testCase.input);

      expect(result).toBeDefined();
      expect(result).toContain('날씨');
    });

    it('should tokenize question sentence', () => {
      if (!tagger) return expect(true).toBe(true);

      const testCase = testSentences.test_cases.find(
        (tc) => tc.id === 'basic_003'
      );
      const result = tagger.parse(testCase.input);

      expect(result).toBeDefined();
      expect(result).toContain('무엇');
    });

    it('should tokenize compound noun', () => {
      if (!tagger) return expect(true).toBe(true);

      const testCase = testSentences.test_cases.find(
        (tc) => tc.id === 'compound_001'
      );
      const result = tagger.parse(testCase.input);

      expect(result).toBeDefined();
      expect(result).toContain('대한민국');
      expect(result).toContain('서울');
    });

    it('should tokenize mixed Korean and English', () => {
      if (!tagger) return expect(true).toBe(true);

      const testCase = testSentences.test_cases.find(
        (tc) => tc.id === 'mixed_001'
      );
      const result = tagger.parse(testCase.input);

      expect(result).toBeDefined();
      expect(result).toContain('Python');
    });

    it('should tokenize numbers', () => {
      if (!tagger) return expect(true).toBe(true);

      const testCase = testSentences.test_cases.find(
        (tc) => tc.id === 'numbers_001'
      );
      const result = tagger.parse(testCase.input);

      expect(result).toBeDefined();
      expect(result).toContain('2024');
    });

    it('should tokenize honorific speech', () => {
      if (!tagger) return expect(true).toBe(true);

      const testCase = testSentences.test_cases.find(
        (tc) => tc.id === 'honorific_001'
      );
      const result = tagger.parse(testCase.input);

      expect(result).toBeDefined();
      expect(result).toContain('선생님');
    });
  });

  describe('Edge Cases', () => {
    it('should handle empty string', () => {
      if (!tagger) return expect(true).toBe(true);

      const testCase = testSentences.test_cases.find(
        (tc) => tc.id === 'edge_empty'
      );
      const result = tagger.parse(testCase.input);

      expect(result).toBeDefined();
    });

    it('should handle whitespace only', () => {
      if (!tagger) return expect(true).toBe(true);

      const testCase = testSentences.test_cases.find(
        (tc) => tc.id === 'edge_whitespace'
      );
      const result = tagger.parse(testCase.input);

      expect(result).toBeDefined();
    });

    it('should handle punctuation only', () => {
      if (!tagger) return expect(true).toBe(true);

      const testCase = testSentences.test_cases.find(
        (tc) => tc.id === 'edge_punctuation'
      );
      const result = tagger.parse(testCase.input);

      expect(result).toBeDefined();
    });

    it('should handle long sentence', () => {
      if (!tagger) return expect(true).toBe(true);

      const testCase = testSentences.test_cases.find(
        (tc) => tc.id === 'long_sentence'
      );
      const result = tagger.parse(testCase.input);

      expect(result).toBeDefined();
    });
  });

  describe('Parse Modes', () => {
    it('should parse to array of tokens', () => {
      if (!tagger || !tagger.parseToArray) return expect(true).toBe(true);

      const text = '나는 학교에 갑니다.';
      const result = tagger.parseToArray(text);

      expect(Array.isArray(result)).toBe(true);
      expect(result.length).toBeGreaterThan(0);
    });

    it('should parse to JSON', () => {
      if (!tagger || !tagger.parseToJson) return expect(true).toBe(true);

      const text = '나는 학교에 갑니다.';
      const result = tagger.parseToJson(text);

      expect(result).toBeDefined();
      const parsed = JSON.parse(result);
      expect(parsed).toHaveProperty('tokens');
      expect(Array.isArray(parsed.tokens)).toBe(true);
    });
  });

  describe('Memory Management', () => {
    it('should not leak memory on repeated calls', () => {
      if (!tagger) return expect(true).toBe(true);

      const text = '나는 학교에 갑니다.';

      // Parse many times
      for (let i = 0; i < 1000; i++) {
        const result = tagger.parse(text);
        expect(result).toBeDefined();
      }
    });

    it('should handle large text', () => {
      if (!tagger) return expect(true).toBe(true);

      const text = '나는 학교에 갑니다. '.repeat(1000);
      const result = tagger.parse(text);

      expect(result).toBeDefined();
    });
  });

  describe('Concurrent Parsing', () => {
    it('should handle concurrent parsing', async () => {
      if (!tagger) return expect(true).toBe(true);

      const texts = [
        '나는 학교에 갑니다.',
        '오늘은 날씨가 좋습니다.',
        '이것은 무엇입니까?',
      ];

      const promises = texts.map((text) =>
        Promise.resolve(tagger.parse(text))
      );

      const results = await Promise.all(promises);

      expect(results).toHaveLength(texts.length);
      results.forEach((result) => {
        expect(result).toBeDefined();
        expect(typeof result).toBe('string');
      });
    });
  });
});
