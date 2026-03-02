# MeCab-Ko 소개

**MeCab-Ko**는 한국어 형태소 분석을 위한 고성능 도구입니다. 기존 C/C++ 기반의 은전한닢(mecab-ko)을 순수 Rust로 재구현하여, 메모리 안전성과 현대적인 개발 환경을 제공합니다.

> **최신 버전: v0.2.0** (2026-03-02)
>
> - 정확도 측정 인프라 구축
> - Unknown 단어 처리 개선
> - 복합명사 분해 개선
> - 사전 품질 검증 도구

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
| `mecab-ko` | 사용자를 위한 통합 인터페이스 | v0.2.0 |
| `mecab-ko-core` | Lattice, Viterbi, 미등록어 처리 | v0.2.0 |
| `mecab-ko-dict` | 사전 로딩, Trie, 연접 비용 매트릭스 | v0.2.0 |
| `mecab-ko-hangul` | 자모 분리/결합, 문자 분류 | v0.1.1 |
| `mecab-ko-cli` | `mecab-ko` 명령줄 도구 | v0.2.0 |
| `mecab-ko-elasticsearch` | Nori 호환 분석기 | v0.2.0 |

## 다른 프로젝트와의 비교

| 프로젝트 | 언어 | Throughput | Memory | 특징 |
|---------|------|------------|--------|------|
| **mecab-ko (원본)** | C++ | 18 MB/s | ~80 MB | 원조, 유지보수 중단 |
| **Kiwi** | C++ | 22 MB/s | ~150 MB | 독자 모델, 높은 정확도 |
| **Lindera** | Rust | 12 MB/s | ~180 MB | 일본어 중심 |
| **MeCab-Ko** | Rust | 15 MB/s | ~145 MB | mecab-ko 호환, 순수 Rust |

## v0.2.0 주요 변경사항

### 정확도 측정 인프라
- `mecab evaluate` CLI 서브커맨드
- Token/Sentence/POS Accuracy 측정
- Precision/Recall/F1 계산

### Unknown 단어 처리 개선
- 패턴 감지: CamelCase, ProperNoun, HangulAlphaMix 등
- 패턴별 비용 조정
- 품사 태그 추정 개선

### 복합명사 분해 개선
- 종성 패턴 분석 알고리즘
- 접미사/접두사 자동 감지
- Character offset 정확도 개선

자세한 내용은 [변경 이력](changelog.md)을 참조하세요.

## 라이선스

Apache 2.0 또는 MIT 라이선스 중 선택하여 사용할 수 있습니다.

## 시작하기

다음 장에서 설치 방법과 빠른 시작 가이드를 확인하세요:

- [설치](installation.md)
- [빠른 시작](quick-start.md)
- [기본 사용법 튜토리얼](tutorials/basic-usage.md)
