# 한국어 형태소 분석기 현대화 프로젝트 (Korean Morphological Analyzer Modernization)

> **Project**: MeCab-Ko - Korean Morphological Analyzer in Rust  
> **Author**: hephaex (hephaex@gmail.com)  
> **Repository**: https://github.com/hephaex/mecab-ko

## 프로젝트 코드명: **MeCab-Ko** (메캅-러스트-코)

---

## 📋 프로젝트 개요

### 배경 및 동기

한국어 형태소 분석 생태계의 현재 상황:

| 프로젝트 | 언어 | 최종 업데이트 | 현황 |
|---------|------|-------------|------|
| **mecab-ko** | C/C++ | ~2018 | 사실상 유지보수 중단 |
| **mecab-ko-dic** | CSV/데이터 | 2018-07-20 (v2.1.1) | 6년+ 미갱신 |
| **Nori** (Lucene) | Java | 활발 | mecab-ko-dic 의존 |
| **Kiwi** | C++ | 2024 활발 | 독자 모델, 86.7% 정확도 |
| **Lindera** | Rust | 2024 활발 | ko-dic 지원, 기반 활용 가능 |

**핵심 문제점:**
1. **사전(Dictionary)의 노후화**: 2018년 이후 신조어, 외래어, 전문용어 미반영
2. **코드베이스 레거시화**: C/C++ 기반으로 메모리 안전성, 현대적 빌드시스템 부재
3. **생태계 파편화**: 여러 fork가 난립하여 통합적 개선 어려움

### 프로젝트 목표

```
┌─────────────────────────────────────────────────────────────────┐
│  Phase 1: 사전 현대화 (mecab-ko-dic 2.x → 3.0)                 │
│  Phase 2: Rust 리팩토링 (mecab-ko → mecab-ko)               │
│  Phase 3: Nori/Kiwi 호환 레이어 및 생태계 통합                 │
│  Phase 4: Elasticsearch/Lucene 플러그인 배포                   │
└─────────────────────────────────────────────────────────────────┘
```

---

## 🏗️ 아키텍처 설계

### 전체 구조

```
┌─────────────────────────────────────────────────────────────────────────┐
│                    MeCab-Ko 아키텍처                                 │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  ┌─────────────────┐   ┌─────────────────┐   ┌─────────────────┐       │
│  │   CLI Tool      │   │  Python Binding │   │  WASM Module    │       │
│  │  (mecab-ko)  │   │  (PyO3/maturin) │   │   (Browser)     │       │
│  └────────┬────────┘   └────────┬────────┘   └────────┬────────┘       │
│           │                     │                     │                │
│  ┌────────┴─────────────────────┴─────────────────────┴────────┐       │
│  │                     Rust Core Library                        │       │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐       │       │
│  │  │   Tokenizer  │  │   Lattice    │  │   Viterbi    │       │       │
│  │  │   Module     │  │   Builder    │  │   Search     │       │       │
│  │  └──────────────┘  └──────────────┘  └──────────────┘       │       │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐       │       │
│  │  │  Dictionary  │  │    Cost      │  │   Unknown    │       │       │
│  │  │   Manager    │  │   Matrix     │  │  Word Handler│       │       │
│  │  └──────────────┘  └──────────────┘  └──────────────┘       │       │
│  └──────────────────────────────────────────────────────────────┘       │
│                              │                                          │
│  ┌──────────────────────────┴──────────────────────────────────┐       │
│  │                Dictionary Layer (v3.0)                       │       │
│  │  ┌────────────┐  ┌────────────┐  ┌────────────┐             │       │
│  │  │  Core Dic  │  │  User Dic  │  │ Domain Dic │             │       │
│  │  │ (800K+ 엔트리)│  │ (사용자 정의)│  │ (IT/의료/법률)│             │       │
│  │  └────────────┘  └────────────┘  └────────────┘             │       │
│  └──────────────────────────────────────────────────────────────┘       │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

### 기술 스택

| 영역 | 기술 | 선택 이유 |
|-----|------|---------|
| 코어 | **Rust** | 메모리 안전성, 성능, WASM 지원 |
| 사전 빌더 | **Rust (FST/Double-Array)** | 압축률, 조회 성능 |
| Python 바인딩 | **PyO3 + maturin** | 생태계 호환성 |
| WASM | **wasm-pack** | 브라우저 지원 |
| 사전 포맷 | **Custom Binary + LZMA** | Lindera 호환, 최소 용량 |
| CI/CD | **GitHub Actions** | 멀티플랫폼 빌드 |

---

## 📊 이슈 리스트 및 분류

### Epic 1: 사전 현대화 (Dictionary Modernization)

| ID | 이슈 | 우선순위 | 복잡도 | 의존성 |
|----|-----|---------|-------|--------|
| DIC-001 | mecab-ko-dic 소스 분석 및 포맷 문서화 | P0 | M | - |
| DIC-002 | 세종 말뭉치 v2.0 품사 태그 체계 검토 | P0 | S | DIC-001 |
| DIC-003 | 모두의 말뭉치 데이터셋 수집 및 라이센스 검토 | P0 | M | - |
| DIC-004 | AI Hub 말뭉치 활용 가능성 조사 | P1 | S | - |
| DIC-005 | 신조어 수집 파이프라인 구축 (나무위키, 위키피디아) | P1 | L | DIC-001 |
| DIC-006 | IT/기술 용어 도메인 사전 구축 | P1 | M | DIC-001 |
| DIC-007 | 외래어 표기 정규화 규칙 정의 | P1 | M | DIC-002 |
| DIC-008 | 연접 비용 행렬 재학습 (CRF 기반) | P0 | XL | DIC-003 |
| DIC-009 | 사전 검증 테스트셋 구축 | P1 | M | DIC-003 |
| DIC-010 | 바이너리 사전 포맷 v3.0 설계 | P0 | L | DIC-001 |

### Epic 2: Rust 코어 구현 (mecab-ko)

| ID | 이슈 | 우선순위 | 복잡도 | 의존성 |
|----|-----|---------|-------|--------|
| RST-001 | Lindera 코드베이스 분석 및 fork 전략 수립 | P0 | M | - |
| RST-002 | 프로젝트 구조 설계 (workspace, crate 분리) | P0 | S | RST-001 |
| RST-003 | 바이너리 사전 로더 구현 | P0 | L | DIC-010 |
| RST-004 | Double-Array Trie 구현 (character dict) | P0 | XL | RST-002 |
| RST-005 | Viterbi 알고리즘 구현 | P0 | XL | RST-003 |
| RST-006 | 연접 비용 행렬 로더 구현 | P0 | M | RST-003 |
| RST-007 | 미등록어 처리 모듈 구현 | P1 | L | RST-005 |
| RST-008 | 한글 자소 분리/결합 유틸리티 | P0 | M | - |
| RST-009 | 띄어쓰기 특화 비용 조정 (left-space-penalty) | P1 | M | RST-006 |
| RST-010 | N-best 결과 출력 기능 | P2 | M | RST-005 |
| RST-011 | 사용자 정의 사전 지원 | P1 | M | RST-003 |
| RST-012 | CLI 인터페이스 구현 (clap) | P1 | S | RST-005 |
| RST-013 | 단위 테스트 및 벤치마크 | P1 | M | RST-005 |
| RST-014 | 문서화 (rustdoc + mdbook) | P2 | M | RST-012 |

### Epic 3: 바인딩 및 통합 (Bindings & Integration)

| ID | 이슈 | 우선순위 | 복잡도 | 의존성 |
|----|-----|---------|-------|--------|
| BND-001 | Python 바인딩 설계 (konlpy 호환 API) | P1 | M | RST-005 |
| BND-002 | PyO3 wrapper 구현 | P1 | L | BND-001 |
| BND-003 | maturin 기반 PyPI 배포 설정 | P1 | M | BND-002 |
| BND-004 | WASM 바인딩 구현 | P2 | L | RST-005 |
| BND-005 | Nori 호환 레이어 설계 | P2 | L | RST-005 |
| BND-006 | Kiwi 품사 태그 매핑 테이블 | P2 | S | DIC-002 |
| BND-007 | Node.js 바인딩 (neon) | P3 | M | RST-005 |

### Epic 4: Elasticsearch/Lucene 플러그인

| ID | 이슈 | 우선순위 | 복잡도 | 의존성 |
|----|-----|---------|-------|--------|
| ELS-001 | Lucene Nori 모듈 분석 | P2 | M | - |
| ELS-002 | JNI 바인딩 설계 | P2 | L | RST-005 |
| ELS-003 | Elasticsearch analysis-nori-rs 플러그인 | P2 | XL | ELS-002 |
| ELS-004 | OpenSearch 호환성 테스트 | P3 | M | ELS-003 |

### Epic 5: 품질 및 배포 (Quality & Release)

| ID | 이슈 | 우선순위 | 복잡도 | 의존성 |
|----|-----|---------|-------|--------|
| QA-001 | 정확도 벤치마크 프레임워크 | P1 | M | RST-013 |
| QA-002 | 성능 벤치마크 (Kiwi, mecab-ko 대비) | P1 | M | RST-013 |
| QA-003 | 메모리 사용량 프로파일링 | P2 | S | RST-005 |
| QA-004 | 멀티플랫폼 CI/CD 파이프라인 | P1 | M | RST-012 |
| QA-005 | crates.io 배포 자동화 | P1 | S | QA-004 |
| QA-006 | 문서 웹사이트 (docs.rs + mdbook) | P2 | M | RST-014 |

---

## 🗓️ 24주 스프린트 계획

### Phase 1: 기반 구축 (Sprint 1-4)

#### Sprint 1 (Week 1-2): 프로젝트 셋업 및 분석

| 작업 | 담당 | 산출물 |
|-----|-----|-------|
| DIC-001 mecab-ko-dic 구조 분석 | Core | 분석 문서 |
| RST-001 Lindera 코드 분석 | Core | 아키텍처 문서 |
| RST-002 프로젝트 구조 설계 | Core | Cargo workspace 초기화 |
| DIC-002 품사 태그 체계 정리 | NLP | 태그 매핑 테이블 |

**마일스톤**: 기술 분석 완료, 프로젝트 저장소 초기화

#### Sprint 2 (Week 3-4): 사전 파이프라인 기초

| 작업 | 담당 | 산출물 |
|-----|-----|-------|
| DIC-003 모두의 말뭉치 수집 | Data | 원시 데이터셋 |
| DIC-010 바이너리 포맷 v3.0 설계 | Core | 포맷 명세서 |
| RST-008 한글 유틸리티 구현 | Core | `hangul-utils` crate |
| RST-003 사전 로더 스켈레톤 | Core | 기본 구조 구현 |

**마일스톤**: 한글 처리 기초 모듈 완성

#### Sprint 3 (Week 5-6): 코어 데이터 구조

| 작업 | 담당 | 산출물 |
|-----|-----|-------|
| RST-004 Double-Array Trie 구현 | Core | `da-trie` crate |
| RST-006 연접 비용 행렬 로더 | Core | `cost-matrix` 모듈 |
| DIC-005 신조어 수집 시작 | Data | 수집 스크립트 |
| RST-013 테스트 프레임워크 | Core | 테스트 인프라 |

**마일스톤**: 핵심 데이터 구조 구현 완료

#### Sprint 4 (Week 7-8): Viterbi 엔진

| 작업 | 담당 | 산출물 |
|-----|-----|-------|
| RST-005 Viterbi 알고리즘 구현 | Core | `viterbi` crate |
| RST-007 미등록어 처리 기초 | Core | `unknown-handler` 모듈 |
| DIC-007 외래어 규칙 정의 | NLP | 정규화 규칙셋 |

**마일스톤**: 기본 형태소 분석 가능

---

### Phase 2: 핵심 기능 구현 (Sprint 5-10)

#### Sprint 5-6 (Week 9-12): 사전 학습 파이프라인

| 작업 | 담당 | 산출물 |
|-----|-----|-------|
| DIC-008 CRF 기반 연접 비용 재학습 | ML | 학습 파이프라인 |
| DIC-006 IT 도메인 사전 1차 | Data | 기술 용어 10K+ |
| RST-009 띄어쓰기 비용 조정 | Core | 한국어 특화 기능 |

#### Sprint 7-8 (Week 13-16): 정확도 향상

| 작업 | 담당 | 산출물 |
|-----|-----|-------|
| DIC-009 검증 테스트셋 구축 | QA | 골든 데이터셋 |
| QA-001 정확도 벤치마크 | QA | 측정 결과 리포트 |
| RST-011 사용자 사전 지원 | Core | 커스텀 사전 기능 |

#### Sprint 9-10 (Week 17-20): CLI 및 기본 API

| 작업 | 담당 | 산출물 |
|-----|-----|-------|
| RST-012 CLI 구현 | Core | `mecab-ko` 바이너리 |
| RST-010 N-best 출력 | Core | 다중 결과 기능 |
| QA-002 성능 벤치마크 | QA | 성능 리포트 |

**마일스톤**: 독립 실행 가능한 CLI 도구

---

### Phase 3: 생태계 통합 (Sprint 11-18)

#### Sprint 11-12 (Week 21-24): Python 바인딩

| 작업 | 담당 | 산출물 |
|-----|-----|-------|
| BND-001 Python API 설계 | Binding | API 명세 |
| BND-002 PyO3 구현 | Binding | `mecab-ko-python` |
| BND-003 PyPI 배포 | Binding | pip install 가능 |

#### Sprint 13-14 (Week 25-28): WASM 및 웹

| 작업 | 담당 | 산출물 |
|-----|-----|-------|
| BND-004 WASM 바인딩 | Binding | npm 패키지 |
| RST-014 문서화 | Docs | rustdoc + mdbook |
| QA-006 문서 웹사이트 | Docs | docs.mecab-ko.dev |

#### Sprint 15-16 (Week 29-32): Nori 호환성

| 작업 | 담당 | 산출물 |
|-----|-----|-------|
| BND-005 Nori 호환 레이어 | Binding | 호환 API |
| ELS-001 Lucene 분석 | Integration | 분석 문서 |
| BND-006 Kiwi 품사 매핑 | NLP | 매핑 테이블 |

#### Sprint 17-18 (Week 33-36): Elasticsearch 플러그인

| 작업 | 담당 | 산출물 |
|-----|-----|-------|
| ELS-002 JNI 바인딩 | Integration | JNI 레이어 |
| ELS-003 ES 플러그인 | Integration | analysis-nori-rs |

**마일스톤**: Elasticsearch 통합 완료

---

### Phase 4: 안정화 및 배포 (Sprint 19-24)

#### Sprint 19-20 (Week 37-40): 최적화

| 작업 | 담당 | 산출물 |
|-----|-----|-------|
| QA-003 메모리 프로파일링 | QA | 최적화 리포트 |
| 성능 튜닝 | Core | 최적화된 코어 |
| ELS-004 OpenSearch 호환 | Integration | 테스트 결과 |

#### Sprint 21-22 (Week 41-44): 안정화

| 작업 | 담당 | 산출물 |
|-----|-----|-------|
| 버그 수정 | All | 안정화 릴리스 |
| 사전 v3.0 최종본 | Data | 완성된 사전 |
| BND-007 Node.js 바인딩 | Binding | npm 패키지 |

#### Sprint 23-24 (Week 45-48): 정식 릴리스

| 작업 | 담당 | 산출물 |
|-----|-----|-------|
| QA-004 CI/CD 완성 | DevOps | 자동화 파이프라인 |
| QA-005 crates.io 배포 | DevOps | v1.0.0 릴리스 |
| 커뮤니티 공개 | All | 공식 발표 |

**마일스톤**: **mecab-ko v1.0.0 정식 릴리스**

---

## 📈 성공 지표 (KPI)

### 정확도 지표

| 메트릭 | 현재 (mecab-ko) | 목표 (v1.0) | 측정 방법 |
|-------|----------------|------------|---------|
| 형태소 분석 정확도 | ~93% | **≥95%** | 세종 테스트셋 |
| 신조어 커버리지 | ~60% | **≥85%** | 2020-2024 신조어 리스트 |
| 미등록어 처리율 | 기본 | **≥80%** | 커스텀 테스트셋 |

### 성능 지표

| 메트릭 | 현재 (mecab-ko) | 목표 (v1.0) | 측정 방법 |
|-------|----------------|------------|---------|
| 처리 속도 | ~100K 어절/초 | **≥150K** | 벤치마크 |
| 메모리 사용량 | ~200MB | **≤150MB** | 프로파일링 |
| 사전 크기 | ~60MB | **≤50MB** | 압축 후 |
| 콜드 스타트 | ~500ms | **≤200ms** | 초기화 시간 |

### 생태계 지표

| 메트릭 | 목표 | 측정 방법 |
|-------|------|---------|
| PyPI 다운로드 | 10K/월 | PyPI 통계 |
| GitHub Stars | 500+ | GitHub |
| ES 플러그인 설치 | 1K+ | 다운로드 |

---

## 🔗 관련 리소스

### 참조 프로젝트

- **Lindera**: https://github.com/lindera/lindera (Rust, Apache 2.0)
- **Kiwi**: https://github.com/bab2min/Kiwi (C++, LGPL v3)
- **mecab-ko**: https://bitbucket.org/eunjeon/mecab-ko (C++, BSD/GPL/LGPL)
- **Nori**: Lucene 내장 (Java, Apache 2.0)

### 데이터 소스

- **세종 말뭉치**: 국립국어원 (연구 목적 사용)
- **모두의 말뭉치**: https://corpus.korean.go.kr/ (공개)
- **AI Hub**: https://aihub.or.kr/ (조건부 공개)

### 기술 문서

- mecab-ko-dic 태그 체계: [Google Spreadsheet](https://docs.google.com/spreadsheets/d/1-9blXKjtjeKZqsf4NzHeYJCrr49-nXeRF6D80udfcwY)
- MeCab 알고리즘: https://taku910.github.io/mecab/

---

## 📝 변경 이력

| 버전 | 날짜 | 변경 내용 |
|-----|-----|---------|
| 0.1.0 | 2025-01-04 | 초기 계획 작성 |

---

**문서 작성**: Claude (Anthropic)  
**프로젝트 오너**: Mario  
**라이센스**: Apache 2.0 / MIT (선택 가능)
