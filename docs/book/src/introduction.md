# MeCab-Ko 소개

**MeCab-Ko**는 한국어 형태소 분석을 위한 고성능 오픈소스 라이브러리입니다. 기존 C/C++ 기반의 은전한닢(mecab-ko)을 순수 Rust로 재구현하여, 메모리 안전성과 현대적인 개발 환경을 제공합니다.

> **최신 버전: v0.4.0** (2026-03-05) 🎉 **crates.io 정식 배포!**
>
> - **6개 크레이트 crates.io 정식 배포**
> - 세종 코퍼스 호환 모드 강화 (후처리 파이프라인 완성)
> - 고유명사 ~200개 추가 (도시, 국가, 브랜드, 유명인)
> - 신조어 사전 v3.0 (511개)
> - 평가 데이터셋 확장 (160 → 300문장)
> - CI 자동 정확도 측정 (회귀 탐지)

## 왜 MeCab-Ko인가?

**한국어 자연어 처리(NLP)**의 첫 단계는 형태소 분석입니다. MeCab-Ko는 다음과 같은 장점을 제공합니다:

- **높은 정확도**: mecab-ko-dic 기반의 검증된 사전
- **빠른 속도**: Rust의 성능 + Zero-copy 사전 로딩
- **다양한 플랫폼**: Python, Node.js, WebAssembly, Rust
- **Elasticsearch 호환**: Nori 분석기와 호환되는 API

## 형태소 분석이란?

형태소 분석(Morphological Analysis)은 문장을 가장 작은 의미 단위인 형태소로 분리하고, 각 형태소의 품사를 태깅하는 자연어 처리의 기본 과정입니다.

예를 들어, "아버지가방에들어가신다"라는 문장을 분석하면:

```
아버지    NNG,가족,*,*,*,*,아버지,아버지
가        JKS,*,*,*,*,*,가,가
방        NNG,장소,*,*,*,*,방,방
에        JKB,*,*,*,*,*,에,에
들어가    VV,*,*,*,*,*,들어가,들어가
시        EP,*,*,*,*,*,시,시
ㄴ다      EF,*,*,*,*,*,ㄴ다,ㄴ다
EOS
```

## 주요 특징

### 순수 Rust 구현

- `unsafe` 코드 없이 메모리 안전성 보장
- 크로스 플랫폼 지원 (Linux, macOS, Windows)
- WASM 지원으로 브라우저에서도 실행 가능

### 한국어 최적화

- 띄어쓰기 패널티를 통한 한국어 특화 분석
- 한글 자모 분리/결합 유틸리티 내장
- 세종 품사 태그 체계 기반
- Unknown 단어 패턴 감지 및 처리 (v0.2.0)

### 고성능

- Zero-copy 사전 로딩
- 효율적인 Double-Array Trie 검색
- Viterbi 알고리즘 최적 구현
- 처리량: ~238K morphemes/sec

### 유연성

- 사용자 사전 지원
- 다양한 출력 포맷 (MeCab, Wakati, JSON, CSV 등)
- 라이브러리 및 CLI 도구 제공
- Elasticsearch Nori 호환 분석기

## 성능 지표

| 지표 | 목표 | 측정값 | 상태 |
|------|------|--------|------|
| Throughput | 150K ops/sec | 238K | PASS |
| Cold Start | < 200ms | 132ms | PASS |
| Memory | < 150MB | 145MB | PASS |

## 프로젝트 구조

```
mecab-ko/
├── rust/crates/
│   ├── mecab-ko/               # 통합 라이브러리
│   ├── mecab-ko-core/          # 핵심 분석 엔진
│   ├── mecab-ko-dict/          # 사전 관리
│   ├── mecab-ko-dict-builder/  # 사전 빌드 도구
│   ├── mecab-ko-dict-validator/# 사전 검증 도구
│   ├── mecab-ko-hangul/        # 한글 유틸리티
│   ├── mecab-ko-cli/           # CLI 도구
│   ├── mecab-ko-python/        # Python 바인딩
│   ├── mecab-ko-wasm/          # WASM 바인딩
│   ├── mecab-ko-node/          # Node.js 바인딩
│   └── mecab-ko-elasticsearch/ # ES/Nori 호환
└── docs/
    └── book/                   # 이 가이드북
```

| Crate | 설명 | 상태 |
|-------|------|------|
| `mecab-ko` | 사용자를 위한 통합 인터페이스 | **v0.4.0** (crates.io) |
| `mecab-ko-core` | Lattice, Viterbi, 미등록어 처리, 세종 호환 | **v0.4.0** (crates.io) |
| `mecab-ko-dict` | 사전 로딩, Trie, 연접 비용 매트릭스 | **v0.4.0** (crates.io) |
| `mecab-ko-hangul` | 자모 분리/결합, 문자 분류 | **v0.4.0** (crates.io) |
| `mecab-ko-dict-builder` | 사전 빌드 도구 | **v0.4.0** (crates.io) |
| `mecab-ko-dict-validator` | 사전 검증 도구 | **v0.4.0** (crates.io) |
| `mecab-ko-cli` | `mecab-ko` 명령줄 도구 | v0.4.0 |
| `mecab-ko-elasticsearch` | Nori 호환 분석기 | v0.4.0 |
| `mecab-ko-wasm` | WebAssembly 바인딩 | v0.3.1 (npm) |

## 다른 프로젝트와의 비교

| 프로젝트 | 언어 | Throughput | Memory | 특징 |
|---------|------|------------|--------|------|
| **mecab-ko (원본)** | C++ | 18 MB/s | ~80 MB | 원조, 유지보수 중단 |
| **Kiwi** | C++ | 22 MB/s | ~150 MB | 독자 모델, 높은 정확도 |
| **Lindera** | Rust | 12 MB/s | ~180 MB | 일본어 중심 |
| **MeCab-Ko** | Rust | 15 MB/s | ~145 MB | mecab-ko 호환, 순수 Rust |

## v0.4.0 주요 변경사항

### crates.io 정식 배포 🎉
- 6개 크레이트 crates.io 정식 배포 완료
- `cargo add mecab-ko`로 손쉬운 설치
- MIT/Apache 2.0 듀얼 라이선스

### 세종 코퍼스 호환 모드 강화
- 후처리 파이프라인 완성 (4단계)
  1. `apply_decomposition_corrections()`: 오분석 패턴 보정
  2. `apply_token_merges()`: 잘못 분해된 토큰 병합
  3. `apply_lexicon_overrides()`: 고빈도 어휘 매핑
  4. `apply_context_corrections()`: 컨텍스트 기반 품사 보정
- EP/EC/ETM/ETN 패턴 확장

### 고유명사 및 신조어 확장
- 고유명사 ~200개 추가 (도시, 국가, 브랜드, 대학, 유명인)
- 신조어 사전 v3.0 (511개): AI/ML, 소셜미디어, MZ세대 용어

### 평가 인프라 강화
- 평가 데이터셋: 160 → 300문장 확장
- CI 자동 정확도 측정 (회귀 탐지)
- 기준선: Token Accuracy 23.8% (세종 모드)

### 이전 버전 주요 기능
- K-best Viterbi 경로 탐색 (v0.3.0)
- LRU 캐싱 토크나이저 (v0.3.0)
- 스트리밍 API (v0.3.0)
- npm WASM 배포 (v0.3.1)

자세한 내용은 [변경 이력](changelog.md)을 참조하세요.

## 라이선스

Apache 2.0 또는 MIT 라이선스 중 선택하여 사용할 수 있습니다.

## 시작하기

다음 장에서 설치 방법과 빠른 시작 가이드를 확인하세요:

- [설치](installation.md)
- [빠른 시작](quick-start.md)
- [기본 사용법 튜토리얼](tutorials/basic-usage.md)
