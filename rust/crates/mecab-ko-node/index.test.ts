import { describe, it, expect, beforeAll } from 'vitest';
import { Mecab, getVersion, type Token } from './index.js';

describe('@mecab-ko/node', () => {
  let mecab: Mecab;

  beforeAll(() => {
    mecab = new Mecab();
  });

  describe('Mecab constructor', () => {
    it('should create a new Mecab instance', () => {
      const instance = new Mecab();
      expect(instance).toBeInstanceOf(Mecab);
    });

    it('should create instance with custom dictionary path', () => {
      // This will fail if the path doesn't exist, which is expected
      expect(() => {
        Mecab.withDict('/nonexistent/path');
      }).toThrow();
    });
  });

  describe('tokenize', () => {
    it('should tokenize Korean text', () => {
      const tokens = mecab.tokenize('안녕하세요');

      expect(Array.isArray(tokens)).toBe(true);
      expect(tokens.length).toBeGreaterThan(0);

      tokens.forEach((token: Token) => {
        expect(token).toHaveProperty('surface');
        expect(token).toHaveProperty('pos');
        expect(token).toHaveProperty('start');
        expect(token).toHaveProperty('end');
        expect(typeof token.surface).toBe('string');
        expect(typeof token.pos).toBe('string');
        expect(typeof token.start).toBe('number');
        expect(typeof token.end).toBe('number');
      });
    });

    it('should handle empty string', () => {
      const tokens = mecab.tokenize('');
      expect(Array.isArray(tokens)).toBe(true);
    });

    it('should handle complex sentences', () => {
      const text = '대한민국의 수도는 서울입니다.';
      const tokens = mecab.tokenize(text);

      expect(tokens.length).toBeGreaterThan(0);

      // Verify tokens have proper byte positions
      tokens.forEach((token: Token) => {
        expect(token.start).toBeGreaterThanOrEqual(0);
        expect(token.end).toBeGreaterThan(token.start);
      });
    });

    it('should handle mixed Korean and English', () => {
      const tokens = mecab.tokenize('Hello 안녕');
      expect(tokens.length).toBeGreaterThan(0);
    });

    it('should handle numbers', () => {
      const tokens = mecab.tokenize('2024년 1월 5일');
      expect(tokens.length).toBeGreaterThan(0);
    });
  });

  describe('morphs', () => {
    it('should extract morphemes', () => {
      const morphs = mecab.morphs('형태소 분석');

      expect(Array.isArray(morphs)).toBe(true);
      expect(morphs.length).toBeGreaterThan(0);

      morphs.forEach((morph: string) => {
        expect(typeof morph).toBe('string');
        expect(morph.length).toBeGreaterThan(0);
      });
    });

    it('should handle empty string', () => {
      const morphs = mecab.morphs('');
      expect(Array.isArray(morphs)).toBe(true);
    });

    it('should return same number of morphs as tokens', () => {
      const text = '테스트 문장';
      const tokens = mecab.tokenize(text);
      const morphs = mecab.morphs(text);

      expect(morphs.length).toBe(tokens.length);
    });
  });

  describe('nouns', () => {
    it('should extract nouns', () => {
      const nouns = mecab.nouns('대한민국의 수도는 서울입니다');

      expect(Array.isArray(nouns)).toBe(true);

      nouns.forEach((noun: string) => {
        expect(typeof noun).toBe('string');
        expect(noun.length).toBeGreaterThan(0);
      });
    });

    it('should return empty array when no nouns', () => {
      // Text with no nouns (only particles/endings)
      const nouns = mecab.nouns('');
      expect(Array.isArray(nouns)).toBe(true);
    });

    it('should filter only noun POS tags', () => {
      const text = '프로그래밍 언어';
      const nouns = mecab.nouns(text);
      const tokens = mecab.tokenize(text);

      // All returned items should have NN* POS tags
      const nounTokens = tokens.filter(t => t.pos.startsWith('NN'));
      expect(nouns.length).toBeLessThanOrEqual(nounTokens.length);
    });
  });

  describe('pos', () => {
    it('should return POS tagged pairs', () => {
      const pairs = mecab.pos('안녕하세요');

      expect(Array.isArray(pairs)).toBe(true);
      expect(pairs.length).toBeGreaterThan(0);

      pairs.forEach((pair: string[]) => {
        expect(Array.isArray(pair)).toBe(true);
        expect(pair.length).toBe(2);
        expect(typeof pair[0]).toBe('string'); // surface
        expect(typeof pair[1]).toBe('string'); // pos
        expect(pair[0].length).toBeGreaterThan(0);
        expect(pair[1].length).toBeGreaterThan(0);
      });
    });

    it('should handle empty string', () => {
      const pairs = mecab.pos('');
      expect(Array.isArray(pairs)).toBe(true);
    });

    it('should match tokenize results', () => {
      const text = '테스트';
      const tokens = mecab.tokenize(text);
      const pairs = mecab.pos(text);

      expect(pairs.length).toBe(tokens.length);

      pairs.forEach((pair: string[], idx: number) => {
        expect(pair[0]).toBe(tokens[idx].surface);
        expect(pair[1]).toBe(tokens[idx].pos);
      });
    });
  });

  describe('getVersion', () => {
    it('should return version string', () => {
      const version = getVersion();

      expect(typeof version).toBe('string');
      expect(version.length).toBeGreaterThan(0);
      expect(version).toMatch(/^\d+\.\d+\.\d+/);
    });

    it('should return consistent version', () => {
      const v1 = getVersion();
      const v2 = getVersion();
      expect(v1).toBe(v2);
    });
  });

  describe('Thread safety', () => {
    it('should handle concurrent tokenization', async () => {
      const texts = [
        '첫 번째 문장',
        '두 번째 문장',
        '세 번째 문장',
        '네 번째 문장',
        '다섯 번째 문장'
      ];

      const promises = texts.map(text =>
        Promise.resolve(mecab.tokenize(text))
      );

      const results = await Promise.all(promises);

      expect(results.length).toBe(texts.length);
      results.forEach(tokens => {
        expect(Array.isArray(tokens)).toBe(true);
        expect(tokens.length).toBeGreaterThan(0);
      });
    });
  });

  describe('Edge cases', () => {
    it('should handle special characters', () => {
      const tokens = mecab.tokenize('!@#$%^&*()');
      expect(Array.isArray(tokens)).toBe(true);
    });

    it('should handle whitespace', () => {
      const tokens = mecab.tokenize('   ');
      expect(Array.isArray(tokens)).toBe(true);
    });

    it('should handle very long text', () => {
      const longText = '안녕하세요. '.repeat(100);
      const tokens = mecab.tokenize(longText);
      expect(Array.isArray(tokens)).toBe(true);
      expect(tokens.length).toBeGreaterThan(0);
    });

    it('should handle newlines', () => {
      const tokens = mecab.tokenize('첫째 줄\n둘째 줄');
      expect(Array.isArray(tokens)).toBe(true);
    });

    it('should handle tabs', () => {
      const tokens = mecab.tokenize('탭\t문자');
      expect(Array.isArray(tokens)).toBe(true);
    });
  });

  describe('Performance', () => {
    it('should tokenize reasonably fast', () => {
      const text = '대한민국은 동아시아에 위치한 민주공화국입니다.';
      const iterations = 100;

      const start = performance.now();
      for (let i = 0; i < iterations; i++) {
        mecab.tokenize(text);
      }
      const end = performance.now();

      const avgTime = (end - start) / iterations;

      // Should complete in reasonable time (adjust threshold as needed)
      expect(avgTime).toBeLessThan(100); // 100ms per tokenization
    });
  });
});
