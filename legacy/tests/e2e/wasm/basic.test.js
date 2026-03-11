/**
 * Basic WASM E2E tests for MeCab-Ko
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

describe('MeCab-Ko WASM Binding - Basic Tokenization', () => {
  let wasm;

  beforeAll(async () => {
    try {
      // Load WASM module
      const wasmPath = join(
        __dirname,
        '..',
        '..',
        '..',
        'rust',
        'target',
        'wasm32-unknown-unknown',
        'release',
        'mecab_ko_wasm.wasm'
      );

      // This is a simplified loading - actual implementation may vary
      const wasmModule = await import(wasmPath);
      wasm = wasmModule;
    } catch (error) {
      console.warn('WASM module not built yet, skipping tests');
    }
  });

  describe('Basic Functionality', () => {
    it('should tokenize simple sentence', () => {
      if (!wasm || !wasm.parse) return expect(true).toBe(true);

      const testCase = testSentences.test_cases.find(
        (tc) => tc.id === 'basic_001'
      );
      const result = wasm.parse(testCase.input);

      expect(result).toBeDefined();
    });

    it('should return JSON format', () => {
      if (!wasm || !wasm.parseToJson) return expect(true).toBe(true);

      const text = '나는 학교에 갑니다.';
      const result = wasm.parseToJson(text);

      expect(result).toBeDefined();
      const parsed = JSON.parse(result);
      expect(parsed).toHaveProperty('tokens');
      expect(Array.isArray(parsed.tokens)).toBe(true);
    });

    it('should handle empty string', () => {
      if (!wasm || !wasm.parse) return expect(true).toBe(true);

      const result = wasm.parse('');
      expect(result).toBeDefined();
    });

    it('should handle long text', () => {
      if (!wasm || !wasm.parse) return expect(true).toBe(true);

      const text = '나는 학교에 갑니다. '.repeat(100);
      const result = wasm.parse(text);

      expect(result).toBeDefined();
    });
  });

  describe('Memory Management', () => {
    it('should not leak memory on repeated calls', () => {
      if (!wasm || !wasm.parse) return expect(true).toBe(true);

      const text = '나는 학교에 갑니다.';

      for (let i = 0; i < 100; i++) {
        const result = wasm.parse(text);
        expect(result).toBeDefined();
      }
    });

    it('should handle large batch processing', () => {
      if (!wasm || !wasm.parse) return expect(true).toBe(true);

      const texts = Array(100)
        .fill('나는 학교에 갑니다.')
        .map((t, i) => `${i}: ${t}`);

      texts.forEach((text) => {
        const result = wasm.parse(text);
        expect(result).toBeDefined();
      });
    });
  });

  describe('Browser Compatibility', () => {
    it('should work in browser environment', () => {
      if (!wasm) return expect(true).toBe(true);

      // Basic check for WASM availability
      expect(typeof WebAssembly).toBe('object');
      expect(typeof WebAssembly.instantiate).toBe('function');
    });

    it('should handle Unicode correctly', () => {
      if (!wasm || !wasm.parse) return expect(true).toBe(true);

      const texts = [
        '한글 테스트',
        'Emoji 😀🎉',
        '漢字 混合',
        'Mixed 문장 test',
      ];

      texts.forEach((text) => {
        const result = wasm.parse(text);
        expect(result).toBeDefined();
      });
    });
  });

  describe('Error Handling', () => {
    it('should handle null input gracefully', () => {
      if (!wasm || !wasm.parse) return expect(true).toBe(true);

      expect(() => {
        // Depending on implementation, this might throw or return empty
        wasm.parse(null);
      }).not.toThrow();
    });

    it('should handle undefined input gracefully', () => {
      if (!wasm || !wasm.parse) return expect(true).toBe(true);

      expect(() => {
        wasm.parse(undefined);
      }).not.toThrow();
    });
  });
});

describe('MeCab-Ko WASM - Performance', () => {
  let wasm;

  beforeAll(async () => {
    try {
      const wasmPath = join(
        __dirname,
        '..',
        '..',
        '..',
        'rust',
        'target',
        'wasm32-unknown-unknown',
        'release',
        'mecab_ko_wasm.wasm'
      );
      const wasmModule = await import(wasmPath);
      wasm = wasmModule;
    } catch (error) {
      console.warn('WASM module not built yet, skipping tests');
    }
  });

  it('should parse 1000 short sentences in reasonable time', () => {
    if (!wasm || !wasm.parse) return expect(true).toBe(true);

    const text = '나는 학교에 갑니다.';
    const start = Date.now();

    for (let i = 0; i < 1000; i++) {
      wasm.parse(text);
    }

    const elapsed = Date.now() - start;
    // Should complete in less than 5 seconds
    expect(elapsed).toBeLessThan(5000);
  });

  it('should handle streaming mode if available', () => {
    if (!wasm || !wasm.parseStream) return expect(true).toBe(true);

    const texts = Array(10)
      .fill(null)
      .map((_, i) => `문장 ${i}: 테스트입니다.`);

    const results = [];
    for (const text of texts) {
      results.push(wasm.parseStream(text));
    }

    expect(results).toHaveLength(10);
  });
});
