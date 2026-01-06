# Node.js 바인딩

MeCab-Ko는 Neon을 사용하여 Node.js 바인딩을 제공합니다.

## 설치

```bash
npm install mecab-ko
# or
yarn add mecab-ko
# or
pnpm add mecab-ko
```

## 빠른 시작

```javascript
const { Tagger } = require('mecab-ko');

// Tagger 생성
const tagger = new Tagger();

// 문장 분석
const result = tagger.parse('안녕하세요');
console.log(result);

// 노드 단위 처리
const nodes = tagger.parseToNodes('형태소 분석');
nodes.forEach(node => {
  console.log(`${node.surface}\t${node.pos}\t${node.feature}`);
});
```

TypeScript:
```typescript
import { Tagger, Node, TaggerConfig } from 'mecab-ko';

const tagger = new Tagger();
const nodes: Node[] = tagger.parseToNodes('안녕하세요');
```

## API 레퍼런스

### `Tagger` 클래스

형태소 분석기의 메인 클래스입니다.

```typescript
class Tagger {
  /**
   * 새로운 Tagger를 생성합니다.
   * @param config - Tagger 설정
   */
  constructor(config?: TaggerConfig);

  /**
   * 텍스트를 분석하고 문자열로 반환합니다.
   * @param text - 분석할 텍스트
   * @returns 분석 결과 문자열
   */
  parse(text: string): string;

  /**
   * 텍스트를 분석하고 Node 배열로 반환합니다.
   * @param text - 분석할 텍스트
   * @returns Node 객체 배열
   */
  parseToNodes(text: string): Node[];

  /**
   * 텍스트를 분석하고 객체로 반환합니다.
   * @param text - 분석할 텍스트
   * @returns 분석 결과 객체
   */
  parseToObject(text: string): ParseResult;

  /**
   * N-best 결과를 반환합니다.
   * @param text - 분석할 텍스트
   * @param n - 반환할 후보 개수
   * @returns N개의 분석 결과 문자열 배열
   */
  parseNBest(text: string, n?: number): string[];

  /**
   * 비동기로 텍스트를 분석합니다.
   * @param text - 분석할 텍스트
   * @returns Promise<분석 결과 문자열>
   */
  parseAsync(text: string): Promise<string>;

  /**
   * 비동기로 Node 배열을 반환합니다.
   * @param text - 분석할 텍스트
   * @returns Promise<Node 객체 배열>
   */
  parseToNodesAsync(text: string): Promise<Node[]>;
}
```

### `TaggerConfig` 인터페이스

```typescript
interface TaggerConfig {
  /** 사전 디렉토리 경로 */
  dictDir?: string;

  /** 사용자 사전 파일 경로 */
  userDict?: string;

  /** 출력 포맷 ("mecab" | "wakati" | "json" | "csv") */
  outputFormat?: string;

  /** 띄어쓰기 패널티 (기본값: -1000) */
  spacePenalty?: number;

  /** 부분 처리 활성화 */
  partial?: boolean;

  /** 전부 출력 활성화 */
  allMorphs?: boolean;

  /** N-best 개수 */
  nbest?: number;

  /** theta 값 (N-best용) */
  theta?: number;
}
```

### `Node` 인터페이스

```typescript
interface Node {
  /** 표면형 (실제 텍스트) */
  surface: string;

  /** 품사 및 의미 정보 */
  feature: string;

  /** 품사 태그 */
  pos: string;

  /** 시작 위치 (바이트 단위) */
  start: number;

  /** 길이 (바이트 단위) */
  length: number;

  /** 비용 */
  cost: number;

  /** 기본형 (원형) */
  baseForm?: string;

  /** 읽기 정보 */
  reading?: string;
}
```

### `ParseResult` 인터페이스

```typescript
interface ParseResult {
  /** 원본 텍스트 */
  text: string;

  /** 분석된 노드 배열 */
  nodes: Node[];
}
```

## 사용 예제

### 기본 사용 (CommonJS)

```javascript
const { Tagger } = require('mecab-ko');

const tagger = new Tagger();
const text = '아버지가방에들어가신다';
const result = tagger.parse(text);

console.log(result);
```

### 기본 사용 (ES Modules)

```javascript
import { Tagger } from 'mecab-ko';

const tagger = new Tagger();
const result = tagger.parse('형태소 분석을 시작합니다');
console.log(result);
```

### TypeScript

```typescript
import { Tagger, Node, TaggerConfig } from 'mecab-ko';

const config: TaggerConfig = {
  outputFormat: 'json',
  spacePenalty: -1000,
};

const tagger = new Tagger(config);
const nodes: Node[] = tagger.parseToNodes('안녕하세요');

nodes.forEach((node: Node) => {
  console.log(`표면형: ${node.surface}`);
  console.log(`품사: ${node.pos}`);
  if (node.baseForm) {
    console.log(`기본형: ${node.baseForm}`);
  }
  console.log('---');
});
```

### Node 단위 처리

```javascript
const { Tagger } = require('mecab-ko');

const tagger = new Tagger();
const nodes = tagger.parseToNodes('형태소 분석을 시작합니다');

for (const node of nodes) {
  console.log(`표면형: ${node.surface}`);
  console.log(`품사: ${node.pos}`);
  console.log(`특성: ${node.feature}`);
  console.log('---');
}
```

### 객체 형식 출력

```javascript
const { Tagger } = require('mecab-ko');

const tagger = new Tagger();
const result = tagger.parseToObject('안녕하세요');

console.log('원본 텍스트:', result.text);
console.log('노드 개수:', result.nodes.length);

result.nodes.forEach((node, index) => {
  console.log(`${index + 1}. ${node.surface} (${node.pos})`);
});
```

### 사용자 사전 사용

```javascript
const { Tagger } = require('mecab-ko');

const tagger = new Tagger({
  userDict: './user.csv',
});

const result = tagger.parse('딥러닝과 머신러닝');
console.log(result);
```

user.csv:
```csv
딥러닝,NNG,-1000,딥러닝
머신러닝,NNG,-1000,머신러닝
```

### Wakati 출력 (형태소만)

```javascript
const { Tagger } = require('mecab-ko');

const tagger = new Tagger({ outputFormat: 'wakati' });
const result = tagger.parse('형태소만 추출합니다');
console.log(result); // "형태소 만 추출 하 ㅂ니다"
```

### JSON 출력

```javascript
const { Tagger } = require('mecab-ko');

const tagger = new Tagger({ outputFormat: 'json' });
const result = tagger.parse('JSON 형식으로 출력');
const data = JSON.parse(result);
console.log(JSON.stringify(data, null, 2));
```

### N-best 분석

```javascript
const { Tagger } = require('mecab-ko');

const tagger = new Tagger();
const candidates = tagger.parseNBest('아버지가방에들어가신다', 3);

candidates.forEach((candidate, index) => {
  console.log(`=== 후보 ${index + 1} ===`);
  console.log(candidate);
});
```

### 비동기 처리

```javascript
const { Tagger } = require('mecab-ko');

const tagger = new Tagger();

async function analyze() {
  try {
    const result = await tagger.parseAsync('비동기 처리 예제');
    console.log(result);

    const nodes = await tagger.parseToNodesAsync('또 다른 문장');
    console.log(nodes);
  } catch (error) {
    console.error('분석 실패:', error);
  }
}

analyze();
```

### Promise 체이닝

```javascript
const { Tagger } = require('mecab-ko');

const tagger = new Tagger();

tagger.parseAsync('첫 번째 문장')
  .then(result => {
    console.log('결과 1:', result);
    return tagger.parseAsync('두 번째 문장');
  })
  .then(result => {
    console.log('결과 2:', result);
  })
  .catch(error => {
    console.error('오류:', error);
  });
```

### 병렬 처리

```javascript
const { Tagger } = require('mecab-ko');

const tagger = new Tagger();
const texts = [
  '첫 번째 문장',
  '두 번째 문장',
  '세 번째 문장',
];

Promise.all(texts.map(text => tagger.parseAsync(text)))
  .then(results => {
    results.forEach((result, index) => {
      console.log(`결과 ${index + 1}:`, result);
    });
  })
  .catch(error => {
    console.error('오류:', error);
  });
```

### Express 서버

```javascript
const express = require('express');
const { Tagger } = require('mecab-ko');

const app = express();
const tagger = new Tagger();

app.use(express.json());

app.post('/analyze', async (req, res) => {
  try {
    const { text } = req.body;

    if (!text) {
      return res.status(400).json({ error: 'No text provided' });
    }

    const result = await tagger.parseToNodesAsync(text);
    res.json({ text, nodes: result });
  } catch (error) {
    res.status(500).json({ error: error.message });
  }
});

app.listen(3000, () => {
  console.log('Server running on port 3000');
});
```

사용 예:
```bash
curl -X POST http://localhost:3000/analyze \
  -H "Content-Type: application/json" \
  -d '{"text": "안녕하세요"}'
```

### 파일 처리

```javascript
const fs = require('fs');
const readline = require('readline');
const { Tagger } = require('mecab-ko');

const tagger = new Tagger();

async function processFile(filename) {
  const fileStream = fs.createReadStream(filename);
  const rl = readline.createInterface({
    input: fileStream,
    crlfDelay: Infinity,
  });

  for await (const line of rl) {
    if (line.trim()) {
      const result = tagger.parse(line);
      console.log(result);
    }
  }
}

processFile('large_file.txt')
  .catch(console.error);
```

### 스트림 처리

```javascript
const { Tagger } = require('mecab-ko');
const { Transform } = require('stream');

const tagger = new Tagger();

class MecabTransform extends Transform {
  constructor(options) {
    super(options);
  }

  _transform(chunk, encoding, callback) {
    try {
      const text = chunk.toString();
      const result = tagger.parse(text);
      this.push(result + '\n');
      callback();
    } catch (error) {
      callback(error);
    }
  }
}

// 사용
process.stdin
  .pipe(new MecabTransform())
  .pipe(process.stdout);
```

### 명사/동사 추출

```javascript
const { Tagger } = require('mecab-ko');

const tagger = new Tagger();
const text = '저는 오늘 학교에 가서 공부를 했습니다.';
const nodes = tagger.parseToNodes(text);

// 명사 추출
const nouns = nodes
  .filter(node => ['NNG', 'NNP'].includes(node.pos))
  .map(node => node.surface);

console.log('명사:', nouns);
// 출력: ['오늘', '학교', '공부']

// 동사 추출
const verbs = nodes
  .filter(node => ['VV', 'VA'].includes(node.pos))
  .map(node => node.surface);

console.log('동사:', verbs);
// 출력: ['가', '하']
```

### Next.js API Route

```typescript
// pages/api/analyze.ts
import type { NextApiRequest, NextApiResponse } from 'next';
import { Tagger } from 'mecab-ko';

const tagger = new Tagger();

export default async function handler(
  req: NextApiRequest,
  res: NextApiResponse
) {
  if (req.method !== 'POST') {
    return res.status(405).json({ error: 'Method not allowed' });
  }

  const { text } = req.body;

  if (!text) {
    return res.status(400).json({ error: 'No text provided' });
  }

  try {
    const nodes = tagger.parseToNodes(text);
    res.status(200).json({ text, nodes });
  } catch (error) {
    res.status(500).json({ error: 'Analysis failed' });
  }
}
```

## 성능 최적화

### Tagger 재사용

```javascript
// Bad - 매번 생성
for (const text of texts) {
  const tagger = new Tagger(); // 비효율적
  const result = tagger.parse(text);
}

// Good - 재사용
const tagger = new Tagger(); // 한 번만 생성
for (const text of texts) {
  const result = tagger.parse(text);
}
```

### 비동기 배치 처리

```javascript
const { Tagger } = require('mecab-ko');

async function batchAnalyze(texts, batchSize = 100) {
  const tagger = new Tagger();
  const results = [];

  for (let i = 0; i < texts.length; i += batchSize) {
    const batch = texts.slice(i, i + batchSize);
    const batchResults = await Promise.all(
      batch.map(text => tagger.parseAsync(text))
    );
    results.push(...batchResults);
  }

  return results;
}
```

## 에러 처리

```javascript
const { Tagger } = require('mecab-ko');

try {
  const tagger = new Tagger({
    dictDir: '/invalid/path',
  });
} catch (error) {
  console.error('Tagger 생성 실패:', error.message);
}

const tagger = new Tagger();

try {
  const result = tagger.parse('분석할 텍스트');
  console.log(result);
} catch (error) {
  console.error('분석 실패:', error.message);
}
```

## 타입 정의 파일

TypeScript 사용 시 타입 정의가 자동으로 제공됩니다:

```typescript
import { Tagger, Node, TaggerConfig, ParseResult } from 'mecab-ko';
```

## 관련 문서

- [Rust API](./rust.md)
- [Python API](./python.md)
- [CLI 사용법](../cli-usage.md)
- [사용자 사전](../user-dictionary.md)
