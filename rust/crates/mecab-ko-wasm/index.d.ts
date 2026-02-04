/**
 * MeCab-Ko WebAssembly TypeScript Definitions
 * Korean morphological analyzer for browser and Node.js
 */

/**
 * A token representing a morpheme in Korean text
 */
export interface Token {
  /** The surface form (original text) */
  readonly surface: string;
  /** Part-of-speech tag */
  readonly pos: string;
  /** Start byte offset in the input text */
  readonly start: number;
  /** End byte offset in the input text */
  readonly end: number;
  /** Reading form (if available) */
  readonly reading?: string;
  /** Lemma/dictionary form (if available) */
  readonly lemma?: string;
  /** Convert token to JSON string */
  toJSON(): string;
}

/**
 * MeCab-Ko tokenizer class
 *
 * @example
 * ```typescript
 * import init, { Mecab } from 'mecab-ko-wasm';
 *
 * await init();
 * const mecab = new Mecab();
 *
 * const tokens = mecab.tokenize('안녕하세요');
 * console.log(tokens);
 *
 * const morphs = mecab.morphs('형태소 분석');
 * console.log(morphs); // ['형태소', '분석']
 *
 * const nouns = mecab.nouns('한국어 형태소 분석기');
 * console.log(nouns); // ['한국어', '형태소', '분석기']
 * ```
 */
export class Mecab {
  /** Create a new MeCab tokenizer instance */
  constructor();

  /**
   * Tokenize Korean text into morphemes
   * @param text - Input Korean text
   * @returns Array of Token objects
   */
  tokenize(text: string): Token[];

  /**
   * Extract morpheme surface forms
   * @param text - Input Korean text
   * @returns Array of surface form strings
   */
  morphs(text: string): string[];

  /**
   * Extract part-of-speech tagged pairs
   * @param text - Input Korean text
   * @returns Array of [surface, pos] tuples
   */
  pos(text: string): [string, string][];

  /**
   * Extract only nouns from text
   * @param text - Input Korean text
   * @returns Array of noun strings
   */
  nouns(text: string): string[];

  /**
   * Split text into space-separated morphemes (wakati mode)
   * @param text - Input Korean text
   * @returns Array of morpheme strings
   */
  wakati(text: string): string[];
}

/**
 * Initialize the WebAssembly module
 * Must be called before creating Mecab instances
 */
export default function init(): Promise<void>;
