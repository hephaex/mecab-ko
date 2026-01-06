# WASM 바인딩

MeCab-Ko는 WebAssembly(WASM)를 통해 브라우저와 Deno 환경에서 실행할 수 있습니다.

## 설치

### npm/yarn/pnpm

```bash
npm install @mecab-ko/wasm
# or
yarn add @mecab-ko/wasm
# or
pnpm add @mecab-ko/wasm
```

### CDN

```html
<script type="module">
  import init, { Tagger } from 'https://cdn.jsdelivr.net/npm/@mecab-ko/wasm/mecab_ko_wasm.js';

  await init();
  const tagger = new Tagger();
  console.log(tagger.parse('안녕하세요'));
</script>
```

## 빠른 시작

### 브라우저 (ES Modules)

```html
<!DOCTYPE html>
<html>
<head>
  <meta charset="utf-8">
  <title>MeCab-Ko WASM Demo</title>
</head>
<body>
  <textarea id="input" placeholder="분석할 텍스트 입력"></textarea>
  <button id="analyze">분석</button>
  <pre id="output"></pre>

  <script type="module">
    import init, { Tagger } from './node_modules/@mecab-ko/wasm/mecab_ko_wasm.js';

    // WASM 초기화
    await init();

    // Tagger 생성
    const tagger = new Tagger();

    // 분석 버튼 이벤트
    document.getElementById('analyze').addEventListener('click', () => {
      const text = document.getElementById('input').value;
      const result = tagger.parse(text);
      document.getElementById('output').textContent = result;
    });
  </script>
</body>
</html>
```

### Webpack/Vite

```javascript
import init, { Tagger } from '@mecab-ko/wasm';

// WASM 초기화
await init();

// Tagger 생성 및 사용
const tagger = new Tagger();
const result = tagger.parse('형태소 분석');
console.log(result);
```

### Deno

```typescript
import init, { Tagger } from 'https://deno.land/x/mecab_ko_wasm/mod.ts';

await init();

const tagger = new Tagger();
console.log(tagger.parse('안녕하세요'));
```

## API 레퍼런스

### `init()` 함수

WASM 모듈을 초기화합니다. 반드시 Tagger를 사용하기 전에 호출해야 합니다.

```typescript
/**
 * WASM 모듈을 초기화합니다.
 * @param module_or_path - WASM 모듈 또는 경로 (선택)
 * @returns Promise<void>
 */
async function init(module_or_path?: RequestInfo | URL | Response | BufferSource | WebAssembly.Module): Promise<void>;
```

### `Tagger` 클래스

```typescript
class Tagger {
  /**
   * 새로운 Tagger를 생성합니다.
   * @param config - Tagger 설정 (선택)
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
   * 리소스를 해제합니다.
   */
  free(): void;
}
```

### `TaggerConfig` 인터페이스

```typescript
interface TaggerConfig {
  /** 출력 포맷 ("mecab" | "wakati" | "json" | "csv") */
  outputFormat?: string;

  /** 띄어쓰기 패널티 (기본값: -1000) */
  spacePenalty?: number;

  /** 부분 처리 활성화 */
  partial?: boolean;

  /** 전부 출력 활성화 */
  allMorphs?: boolean;
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
}
```

## 사용 예제

### React

```typescript
import React, { useEffect, useState } from 'react';
import init, { Tagger } from '@mecab-ko/wasm';

function MecabAnalyzer() {
  const [tagger, setTagger] = useState<Tagger | null>(null);
  const [input, setInput] = useState('');
  const [result, setResult] = useState('');

  useEffect(() => {
    // WASM 초기화
    init().then(() => {
      setTagger(new Tagger());
    });

    // 클린업
    return () => {
      if (tagger) {
        tagger.free();
      }
    };
  }, []);

  const handleAnalyze = () => {
    if (tagger && input) {
      const parsed = tagger.parse(input);
      setResult(parsed);
    }
  };

  return (
    <div>
      <textarea
        value={input}
        onChange={(e) => setInput(e.target.value)}
        placeholder="분석할 텍스트 입력"
      />
      <button onClick={handleAnalyze} disabled={!tagger}>
        분석
      </button>
      <pre>{result}</pre>
    </div>
  );
}

export default MecabAnalyzer;
```

### Vue 3

```vue
<template>
  <div>
    <textarea v-model="input" placeholder="분석할 텍스트 입력"></textarea>
    <button @click="analyze" :disabled="!tagger">분석</button>
    <pre>{{ result }}</pre>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue';
import init, { Tagger } from '@mecab-ko/wasm';

const tagger = ref<Tagger | null>(null);
const input = ref('');
const result = ref('');

onMounted(async () => {
  await init();
  tagger.value = new Tagger();
});

onUnmounted(() => {
  if (tagger.value) {
    tagger.value.free();
  }
});

const analyze = () => {
  if (tagger.value && input.value) {
    result.value = tagger.value.parse(input.value);
  }
};
</script>
```

### Svelte

```svelte
<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import init, { Tagger } from '@mecab-ko/wasm';

  let tagger: Tagger | null = null;
  let input = '';
  let result = '';

  onMount(async () => {
    await init();
    tagger = new Tagger();
  });

  onDestroy(() => {
    if (tagger) {
      tagger.free();
    }
  });

  function analyze() {
    if (tagger && input) {
      result = tagger.parse(input);
    }
  }
</script>

<textarea bind:value={input} placeholder="분석할 텍스트 입력" />
<button on:click={analyze} disabled={!tagger}>분석</button>
<pre>{result}</pre>
```

### Web Worker

```javascript
// worker.js
import init, { Tagger } from '@mecab-ko/wasm';

let tagger = null;

self.onmessage = async (event) => {
  const { type, text } = event.data;

  if (type === 'init') {
    await init();
    tagger = new Tagger();
    self.postMessage({ type: 'ready' });
  } else if (type === 'parse') {
    if (tagger) {
      const result = tagger.parse(text);
      self.postMessage({ type: 'result', result });
    }
  }
};
```

```javascript
// main.js
const worker = new Worker('worker.js', { type: 'module' });

worker.onmessage = (event) => {
  const { type, result } = event.data;

  if (type === 'ready') {
    console.log('Worker ready');
    worker.postMessage({ type: 'parse', text: '안녕하세요' });
  } else if (type === 'result') {
    console.log('결과:', result);
  }
};

worker.postMessage({ type: 'init' });
```

### Node 단위 처리

```typescript
import init, { Tagger } from '@mecab-ko/wasm';

await init();

const tagger = new Tagger();
const nodes = tagger.parseToNodes('형태소 분석을 시작합니다');

nodes.forEach(node => {
  console.log(`표면형: ${node.surface}`);
  console.log(`품사: ${node.pos}`);
  console.log(`특성: ${node.feature}`);
  console.log('---');
});

tagger.free();
```

### JSON 출력

```typescript
import init, { Tagger } from '@mecab-ko/wasm';

await init();

const tagger = new Tagger({ outputFormat: 'json' });
const result = tagger.parseToObject('JSON 형식으로 출력');

console.log('원본 텍스트:', result.text);
console.log('노드 개수:', result.nodes.length);

result.nodes.forEach((node, index) => {
  console.log(`${index + 1}. ${node.surface} (${node.pos})`);
});

tagger.free();
```

### 명사 추출

```typescript
import init, { Tagger } from '@mecab-ko/wasm';

await init();

const tagger = new Tagger();
const text = '저는 오늘 학교에 가서 공부를 했습니다.';
const nodes = tagger.parseToNodes(text);

const nouns = nodes
  .filter(node => ['NNG', 'NNP'].includes(node.pos))
  .map(node => node.surface);

console.log('명사:', nouns);
// 출력: ['오늘', '학교', '공부']

tagger.free();
```

### 실시간 입력 분석

```typescript
import init, { Tagger } from '@mecab-ko/wasm';

await init();

const tagger = new Tagger();
const input = document.getElementById('input') as HTMLTextAreaElement;
const output = document.getElementById('output') as HTMLPreElement;

let debounceTimer: number;

input.addEventListener('input', () => {
  clearTimeout(debounceTimer);
  debounceTimer = setTimeout(() => {
    const text = input.value;
    if (text) {
      const result = tagger.parse(text);
      output.textContent = result;
    } else {
      output.textContent = '';
    }
  }, 300);
});

// 페이지 언로드 시 정리
window.addEventListener('beforeunload', () => {
  tagger.free();
});
```

### IndexedDB에 결과 저장

```typescript
import init, { Tagger } from '@mecab-ko/wasm';

await init();

const tagger = new Tagger();

// IndexedDB 열기
const dbPromise = indexedDB.open('MecabDB', 1);

dbPromise.onupgradeneeded = (event) => {
  const db = (event.target as IDBOpenDBRequest).result;
  if (!db.objectStoreNames.contains('analyses')) {
    db.createObjectStore('analyses', { keyPath: 'id', autoIncrement: true });
  }
};

dbPromise.onsuccess = (event) => {
  const db = (event.target as IDBOpenDBRequest).result;

  function saveAnalysis(text: string) {
    const result = tagger.parseToObject(text);

    const transaction = db.transaction(['analyses'], 'readwrite');
    const store = transaction.objectStore('analyses');

    store.add({
      text: result.text,
      nodes: result.nodes,
      timestamp: Date.now(),
    });
  }

  saveAnalysis('저장할 텍스트');
};
```

### Service Worker 캐싱

```javascript
// service-worker.js
import init, { Tagger } from '@mecab-ko/wasm';

const CACHE_NAME = 'mecab-cache-v1';
let tagger = null;

self.addEventListener('install', async (event) => {
  event.waitUntil(
    (async () => {
      await init();
      tagger = new Tagger();
    })()
  );
});

self.addEventListener('message', (event) => {
  const { type, text, id } = event.data;

  if (type === 'parse' && tagger) {
    const result = tagger.parse(text);
    event.ports[0].postMessage({ id, result });
  }
});
```

## 번들 크기 최적화

### Tree Shaking

```javascript
// 필요한 것만 import
import init, { Tagger } from '@mecab-ko/wasm';

// Tagger만 사용
await init();
const tagger = new Tagger();
```

### 코드 스플리팅 (Webpack)

```javascript
// 동적 import로 lazy loading
const loadMecab = async () => {
  const { default: init, Tagger } = await import('@mecab-ko/wasm');
  await init();
  return new Tagger();
};

// 필요할 때만 로드
button.addEventListener('click', async () => {
  const tagger = await loadMecab();
  const result = tagger.parse(input.value);
  console.log(result);
});
```

### Vite 최적화

```javascript
// vite.config.js
export default {
  optimizeDeps: {
    exclude: ['@mecab-ko/wasm'],
  },
  build: {
    target: 'esnext',
  },
};
```

## 메모리 관리

WASM 메모리는 자동으로 관리되지만, 명시적으로 해제할 수 있습니다:

```typescript
const tagger = new Tagger();

// 사용
const result = tagger.parse('텍스트');

// 사용 완료 후 메모리 해제
tagger.free();
```

React/Vue 등에서 클린업:

```typescript
useEffect(() => {
  let tagger: Tagger | null = null;

  init().then(() => {
    tagger = new Tagger();
  });

  return () => {
    if (tagger) {
      tagger.free();
    }
  };
}, []);
```

## 성능 고려사항

### 초기화 비용

WASM 초기화는 비용이 높으므로 앱 시작 시 한 번만 수행:

```typescript
// Good - 앱 시작 시 한 번
const tagger = await initTagger();

// Bad - 사용할 때마다
button.addEventListener('click', async () => {
  const tagger = await initTagger(); // 비효율적
});
```

### Tagger 재사용

```typescript
// Good - 싱글톤 패턴
let globalTagger: Tagger | null = null;

async function getTagger() {
  if (!globalTagger) {
    await init();
    globalTagger = new Tagger();
  }
  return globalTagger;
}

// Bad - 매번 생성
async function analyze(text: string) {
  const tagger = new Tagger(); // 비효율적
  return tagger.parse(text);
}
```

### Web Worker 활용

무거운 작업은 Web Worker에서 처리:

```typescript
// 메인 스레드를 블록하지 않음
const worker = new Worker('mecab-worker.js', { type: 'module' });
worker.postMessage({ text: largeText });
```

## 브라우저 호환성

- Chrome 57+
- Firefox 52+
- Safari 11+
- Edge 16+

## 관련 문서

- [Rust API](./rust.md)
- [Python API](./python.md)
- [Node.js API](./nodejs.md)
- [CLI 사용법](../cli-usage.md)
