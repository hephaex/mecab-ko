# MeCab-Ko

[![CI](https://github.com/hephaex/mecab-ko/actions/workflows/ci.yml/badge.svg)](https://github.com/hephaex/mecab-ko/actions/workflows/ci.yml)
[![Security](https://github.com/hephaex/mecab-ko/actions/workflows/security.yml/badge.svg)](https://github.com/hephaex/mecab-ko/actions/workflows/security.yml)
[![Rust](https://img.shields.io/badge/rust-1.83+-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE)
[![crates.io](https://img.shields.io/crates/v/mecab-ko.svg)](https://crates.io/crates/mecab-ko)
[![npm](https://img.shields.io/npm/v/mecab-ko-wasm.svg)](https://www.npmjs.com/package/mecab-ko-wasm)

**순수 Rust로 작성된 고성능 한국어 형태소 분석기**

MeCab-Ko는 [은전한닢 프로젝트](https://bitbucket.org/eunjeon/mecab-ko)의 MeCab-Ko 한국어 형태소 분석기를 Rust로 재구현한 프로젝트입니다. 

메모리 안전성, 크로스 플랫폼 지원(WebAssembly 포함), Python/Node.js/브라우저 바인딩을 제공합니다.

## 주요 성과

| 지표 | v0.5.0 | v0.6.0 | v0.7.2 |
|------|--------|--------|--------|
| Token Accuracy (자체셋) | 100.0% | 100.0% | **100.0%** |
| 외부 골드셋 F1 (세종 정규화) | — | — | **0.703** (UD-GSD 0.710 / UD-Kaist 0.705 / KLUE-DP 0.694) |
| 테스트 | 500문장 | 1,100문장 | **1,149 pass / 18 ignored** |
| 처리 속도 | ~238K tok/sec | ~680K tok/sec | **~680K tok/sec** |
| 메모리 사용량 | ~150MB | ~150MB | **~34MB (-77%)** |

> 자체셋 100%는 자체 사전 기반 자체 테스트셋의 토큰 일치율입니다. 외부 골드셋 F1은
> `scripts/review/measure_f1.py`로 실측한 값이며(silver 변환 특성상 상한이 1.0 미만),
> 상세는 [`docs/research/competitor-analysis/`](docs/research/competitor-analysis/) 참조.

### 설치 방법

```bash
# Rust
cargo add mecab-ko

# Python
pip install mecab-ko-python

# npm (WebAssembly)
npm install mecab-ko-wasm

# Docker
docker pull ghcr.io/hephaex/mecab-ko:latest
```

## 주요 특징

- **순수 Rust 구현** - `#![deny(unsafe_code)]`로 메모리 안전성 보장 (FFI 바인딩 크레이트 제외)
- **고성능** - SIMD 가속 Viterbi 알고리즘, ~680K tokens/sec
- **크로스 플랫폼** - Linux, macOS, Windows, WebAssembly 지원
- **다양한 바인딩** - Python (PyO3), Node.js (N-API), WebAssembly
- **한국어 최적화** - 띄어쓰기 보정, 자모 처리, 종성 기반 분석
- **메모리 효율** - LazyEntries로 ~34MB 메모리 사용 (기존 대비 -77%)
- **사용자 사전** - 사용자 정의 단어 추가 지원
- **KoNLPy 호환** - KoNLPy의 Mecab API 대체 가능
- **세종 코퍼스 호환** - 복합 태그 분리 모드 지원
- **검색엔진 통합** - Elasticsearch 8.x / OpenSearch 3.x 플러그인

## 빠른 시작

### Rust

`Cargo.toml`에 추가:

```toml
[dependencies]
mecab-ko = "0.7"
```

```rust
use mecab_ko::Tokenizer;

fn main() -> Result<(), mecab_ko::Error> {
    let mut tokenizer = Tokenizer::new()?;

    // 한국어 텍스트 토큰화
    let tokens = tokenizer.tokenize("안녕하세요, 형태소 분석기입니다.");
    for token in tokens {
        println!("{}\t{}", token.surface, token.pos);
    }
    // 출력:
    // 안녕    NNG
    // 하      XSV
    // 세요    EF
    // ...

    // 분리 모드 (형태소 공백 구분)
    let words = tokenizer.wakati("한국어 형태소 분석");
    println!("{}", words.join(" "));

    // 명사만 추출
    let nouns = tokenizer.nouns("오늘 날씨가 좋습니다");
    println!("{:?}", nouns);  // ["오늘", "날씨"]

    Ok(())
}
```

### Python

```bash
pip install mecab-ko-python
```

```python
from mecab_ko import Mecab

mecab = Mecab()

# 형태소 추출
print(mecab.morphs("안녕하세요"))
# ['안녕', '하', '세요']

# 명사 추출
print(mecab.nouns("아버지가방에들어가신다"))
# ['아버지', '가방']

# 품사 태깅
print(mecab.pos("나는 학생입니다"))
# [('나', 'NP'), ('는', 'JX'), ('학생', 'NNG'), ('이', 'VCP'), ('ㅂ니다', 'EF')]

# MeCab 형식 출력
print(mecab.parse("형태소"))
# 형태소  NNG,*,*,*,*,*,*,*
# EOS
```

### Node.js / 브라우저 (WebAssembly)

```bash
npm install mecab-ko-wasm
```

```javascript
import init, { Mecab } from 'mecab-ko-wasm';

async function analyze() {
    // WASM 모듈 초기화
    await init();
    const mecab = new Mecab();

    // 형태소 추출
    const morphs = mecab.morphs("안녕하세요");
    console.log(morphs);  // ["안녕", "하", "세요"]

    // 명사 추출
    const nouns = mecab.nouns("형태소 분석기입니다");
    console.log(nouns);  // ["형태소", "분석기"]

    // 상세 토큰 정보
    const tokens = mecab.tokenize("한국어 분석기");
    tokens.forEach(token => {
        console.log(`${token.surface}: ${token.pos}`);
    });
}

analyze();
```

## 기능

### 세종 코퍼스 호환 모드

MeCab-Ko의 복합 태그(VV+EF 등)를 세종 코퍼스 형식으로 변환합니다:

```rust
use mecab_ko::sejong::{SejongConverter, SejongToken};

let converter = SejongConverter::new();

// "갔다/VV+EF" → ["갔/VV", "다/EF"]
let tokens = tokenizer.tokenize("갔다");
let sejong_tokens = converter.convert_tokens(&tokens);

// 세종 형식 출력
println!("{}", converter.format_sejong(&sejong_tokens));
// 갔/VV 다/EF
```

### N-best 경로 탐색

```rust
use mecab_ko::ImprovedNbestSearcher;

let lattice = tokenizer.tokenize_to_lattice("한국어");
let searcher = ImprovedNbestSearcher::new(&matrix);
let results = searcher.search(&lattice, 5);  // 상위 5개 경로

for (rank, path) in results.iter().enumerate() {
    println!("#{}: cost={}", rank + 1, path.total_cost);
}
```

### 분석 모드

```rust
use mecab_ko::{extract_nouns, extract_verbs, extract_lemmas};

// 명사만 추출
let nouns = extract_nouns(&mut tokenizer, "오늘 서울 날씨가 좋습니다");
// ["오늘", "서울", "날씨"]

// 동사만 추출
let verbs = extract_verbs(&mut tokenizer, "나는 학교에 갔다");
// ["가다"]

// 원형 복원
let lemmas = extract_lemmas(&mut tokenizer, "아버지가 방에 들어가셨다");
```

### 토큰화 캐싱

```rust
use mecab_ko::{TokenCache, CacheConfig};

let cache = TokenCache::new(CacheConfig::default());
let key = cache.make_key("반복되는 입력");

let tokens = cache.get_or_insert(key, || {
    tokenizer.tokenize("반복되는 입력")
});

println!("캐시 히트율: {:.1}%", cache.stats().hit_rate() * 100.0);
```

## 저장소 구조

```
mecab-ko/
├── rust/               # Rust 소스 코드 (메인)
│   └── crates/         # Rust 크레이트들
├── data/               # 사전 데이터
├── docs/               # 문서
│   ├── examples/       # 예제 코드
│   ├── research/       # 연구 자료
│   └── project/        # 프로젝트 계획/진행
├── search-plugins/     # Elasticsearch/OpenSearch 플러그인
├── docker/             # Docker 설정
├── scripts/            # 빌드/유틸리티 스크립트
├── legacy/             # 레거시 C++ 코드
└── .github/            # CI/CD 설정
```

## 크레이트 구조

| 크레이트 | 설명 | 버전 |
|----------|------|------|
| `mecab-ko` | 통합 API (모든 public API re-export) | [![crates.io](https://img.shields.io/crates/v/mecab-ko.svg)](https://crates.io/crates/mecab-ko) |
| `mecab-ko-core` | 핵심 엔진 (Lattice, Viterbi, Tokenizer) | [![crates.io](https://img.shields.io/crates/v/mecab-ko-core.svg)](https://crates.io/crates/mecab-ko-core) |
| `mecab-ko-dict` | 사전 관리 (Trie, Matrix, 사용자 사전) | [![crates.io](https://img.shields.io/crates/v/mecab-ko-dict.svg)](https://crates.io/crates/mecab-ko-dict) |
| `mecab-ko-hangul` | 한글 유틸리티 (자모 분리/결합) | [![crates.io](https://img.shields.io/crates/v/mecab-ko-hangul.svg)](https://crates.io/crates/mecab-ko-hangul) |
| `mecab-ko-dict-builder` | 사전 컴파일 도구 | [![crates.io](https://img.shields.io/crates/v/mecab-ko-dict-builder.svg)](https://crates.io/crates/mecab-ko-dict-builder) |
| `mecab-ko-cli` | 명령줄 인터페이스 | - |
| `mecab-ko-wasm` | WebAssembly 바인딩 | [![npm](https://img.shields.io/npm/v/mecab-ko-wasm.svg)](https://www.npmjs.com/package/mecab-ko-wasm) |
| `mecab-ko-python` | Python 바인딩 (PyO3) | - |
| `mecab-ko-node` | Node.js 바인딩 (N-API) | - |
| `mecab-ko-elasticsearch` | Elasticsearch/OpenSearch 호환 (JNI) | - |
| `mecab-ko-dict-validator` | 사전 검증 도구 | - |
| `mecab-ko-dict-sync` | 사전 동기화 | - |
| `mecab-ko-profiler` | 성능/메모리 프로파일러 | - |

## CLI 사용법

```bash
# 기본 토큰화
echo "안녕하세요" | mecab

# 분리 모드
mecab --wakati "한국어 형태소 분석"

# 사용자 사전
mecab --user-dict custom.csv "신조어 테스트"

# 도메인 사전 (관보 기관명)
mecab --user-dict data/domain-dic/government/agencies.csv "국토교통부가 환경부와 협의했다"

# REPL 모드
mecab --repl

# 정확도 평가
mecab evaluate --input test.tsv --dicdir ./dict-output

# 세종 호환 모드로 평가
mecab evaluate --input test.tsv --dicdir ./dict-output --sejong
```

## 성능

### KPI (v0.7.2 기준)

| 지표 | 목표 | 현재 |
|------|------|------|
| Token Accuracy (자체셋) | 95%+ | **100.0%** (1,100문장) |
| 외부 골드셋 F1 (실측) | — | **0.703** (세종 정규화, `measure_f1.py`) |
| 토큰화 속도 | ~150K 단어/초 | **~680K 단어/초** |
| 콜드 스타트 | < 200ms | **0.13ms** |
| 메모리 사용량 | < 100MB | **~34MB** (LazyEntries) |
| Clippy 경고 | 0 | **0 warnings** |
| 보안 취약점 | 0 | **0 vulnerabilities** |

## 품사 태그 (세종 코퍼스)

| 태그 | 한국어 | 영어 |
|-----|--------|---------|
| NNG | 일반 명사 | General noun |
| NNP | 고유 명사 | Proper noun |
| NNB | 의존 명사 | Dependent noun |
| NP | 대명사 | Pronoun |
| VV | 동사 | Verb |
| VA | 형용사 | Adjective |
| VX | 보조 용언 | Auxiliary verb |
| MAG | 일반 부사 | General adverb |
| JKS | 주격 조사 | Subject particle |
| JKO | 목적격 조사 | Object particle |
| JX | 보조사 | Auxiliary particle |
| EF | 종결 어미 | Final ending |
| EC | 연결 어미 | Connective ending |
| ETM | 관형형 어미 | Adnominal ending |

전체 품사 태그 목록은 [품사 태그 매핑](docs/pos-tag-mapping.md)을 참조하세요.

## 빌드 방법

### 필수 요구사항

- Rust 1.75 이상
- Python 바인딩: Python 3.9+ 및 maturin
- Node.js 바인딩: Node.js 18+ 및 npm
- WASM: wasm-pack

### 소스에서 빌드

```bash
# 저장소 클론
git clone https://github.com/hephaex/mecab-ko.git
cd mecab-ko/rust

# 전체 빌드
cargo build --release

# 테스트 실행
cargo test

# 린터 실행
cargo clippy

# 코드 포맷팅
cargo fmt

# 문서 빌드
cargo doc --no-deps --open
```

### 사전 빌드

```bash
# mecab-ko-dic 다운로드
cd data
curl -LO https://bitbucket.org/eunjeon/mecab-ko-dic/downloads/mecab-ko-dic-2.1.1-20180720.tar.gz
tar xzf mecab-ko-dic-2.1.1-20180720.tar.gz

# 바이너리 사전 빌드
cargo run -p mecab-ko-dict-builder -- build \
    --input mecab-ko-dic-2.1.1-20180720 \
    --output dict-output
```

## 로드맵

### v0.6.0
- ✅ SIMD 가속 Viterbi 알고리즘 (283% 성능 향상)
- ✅ 1,100문장 테스트셋, 100% Token Accuracy
- ✅ 멀티플랫폼 배포 (crates.io, PyPI, npm, Docker)
- ✅ 세종 코퍼스 호환 모드, N-best 경로 탐색

### v0.7.2 (현재 릴리스)
- ✅ LazyEntries 메모리 최적화 (150MB → 34MB, -77%)
- ✅ Streaming API (SentenceReader, StreamingTokenizer) — 버퍼 상한 16MB, 다중바이트 안전
- ✅ sejong.rs 9,700줄 → 11개 서브모듈 분할, 36개 smoke test 추가
- ✅ CI 워크플로우 통합 (22→20), Actions 11개 최신 버전 업그레이드
- ✅ Clippy zero warnings, 보안 취약점 제로
- ✅ 도메인 사전 자동 수집 파이프라인
  - 신조어: 국립국어원 우리말샘 API 연동, 주간/월간 자동 수집 → PR
  - 뉴스 NNP: 뉴스 고유명사(인명/지명/단체명) 주간 자동 갱신, LLM 검증
  - 관보 기관명: 대한민국 관보 12.8만건에서 503건 정부기관 NNP 추출
  - IT 용어: 283K건 도메인 사전 운영 중
  - CI 검증 게이트: CSV 포맷·품사·중복·인코딩 자동 검증
- 📋 사용자 사전 확장 (도메인 오버레이, 핫리로드)
- 📋 Elasticsearch 8.x / OpenSearch 3.x 플러그인

## 기여하기

기여를 환영합니다! [CONTRIBUTING.md](docs/CONTRIBUTING.md)를 참조해 주세요.

```bash
# 개발 워크플로우
cd rust
cargo build          # 빌드
cargo test           # 테스트
cargo clippy         # 린트
cargo fmt            # 포맷팅
```

### 코딩 규칙

- `unsafe` 코드 사용 금지 (`#![deny(unsafe_code)]`) — FFI 크레이트(python/wasm/node/elasticsearch)는 플랫폼 제약으로 예외
- 라이브러리 코드에서 `unwrap()`, `expect()` 사용 금지
- 모든 public API에 rustdoc 문서 필수

## 라이선스

이 프로젝트는 다음 라이선스 중 하나를 선택하여 사용할 수 있습니다:

- [MIT License](LICENSE-MIT)
- [Apache License 2.0](LICENSE-APACHE)

사전 데이터는 [Apache License 2.0](https://www.apache.org/licenses/LICENSE-2.0)을 따릅니다.

## 감사의 글

- [MeCab](https://taku910.github.io/mecab/) - Taku Kudo
- [은전한닢 프로젝트](https://bitbucket.org/eunjeon/mecab-ko) - 원본 mecab-ko
- [Lindera](https://github.com/lindera/lindera) - Rust 형태소 분석기 참조
- [Kiwi](https://github.com/bab2min/Kiwi) - 한국어 형태소 분석기 참조
- [KoNLPy](https://konlpy.org/) - Python 한국어 NLP 라이브러리
- [국립국어원](https://www.korean.go.kr/) - 세종 코퍼스 및 품사 태그셋

## 연락처

- **개발자**: hephaex (hephaex@gmail.com)
- **이슈**: [GitHub Issues](https://github.com/hephaex/mecab-ko/issues)
- **토론**: [GitHub Discussions](https://github.com/hephaex/mecab-ko/discussions)
