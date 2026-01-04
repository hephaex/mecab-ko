# MeCab-Ko Multi-Agent Development System

> **Project**: MeCab-Ko - Korean Morphological Analyzer in Rust  
> **Author**: hephaex (hephaex@gmail.com)  
> **Repository**: https://github.com/hephaex/mecab-ko

---

## 개요

MeCab-Ko 프로젝트는 효율적인 개발을 위해 **역할 기반 멀티 에이전트 시스템**을 사용합니다. 각 에이전트는 특정 책임 영역을 담당하며, 작업 난이도에 따라 적절한 리소스(모델 티어)가 할당됩니다.

---

## 에이전트 역할 정의

### 1. 🏗️ Architect Agent (설계 에이전트)

**책임 영역:**
- 시스템 아키텍처 설계 및 의사결정
- 기술 스택 선정 및 평가
- 모듈 간 인터페이스 정의
- 성능 목표 설정 및 트레이드오프 분석
- 장기 로드맵 관리

**담당 이슈 유형:**
- `RST-001`: Lindera 분석 및 포크 결정
- `RST-002`: Cargo 워크스페이스 구조 설계
- `DIC-010`: 바이너리 사전 포맷 v3.0 설계
- 모든 아키텍처 결정 기록 (ADR)

**산출물:**
- Architecture Decision Records (ADR)
- 시스템 다이어그램
- API 스펙 문서
- 기술 평가 보고서

---

### 2. 🔍 Analyst Agent (분석 에이전트)

**책임 영역:**
- 기존 시스템 분석 및 역공학
- 데이터 포맷 분석 및 문서화
- 벤치마크 및 성능 분석
- 경쟁 제품/기술 조사
- 요구사항 정제

**담당 이슈 유형:**
- `DIC-001`: mecab-ko-dic 포맷 분석
- `DIC-002`: Sejong 품사 태그 체계 검토
- `ELS-001`: Lucene Nori 모듈 분석
- 모든 리서치 및 분석 태스크

**산출물:**
- 기술 분석 보고서
- 데이터 포맷 명세서
- 비교 분석 문서
- 역공학 문서

---

### 3. 💻 Developer Agent (개발 에이전트)

**책임 영역:**
- 핵심 알고리즘 구현
- 모듈 및 라이브러리 개발
- 단위 테스트 작성
- 버그 수정 및 리팩토링
- 성능 최적화

**담당 이슈 유형:**
- `RST-004`: Double-Array Trie 구현
- `RST-005`: Viterbi 알고리즘 구현
- `RST-008`: 한글 자모 유틸리티
- `BND-002`: PyO3 바인딩 구현
- 모든 구현 태스크

**산출물:**
- 소스 코드
- 단위 테스트
- 인라인 문서 (rustdoc)
- 구현 노트

---

### 4. 👁️ Reviewer Agent (리뷰 에이전트)

**책임 영역:**
- 코드 품질 검토
- 보안 취약점 분석
- 성능 병목 탐지
- 코딩 컨벤션 준수 확인
- 개선 제안

**검토 기준:**
- Rust 안전성 (unsafe 코드 최소화)
- 에러 처리 패턴
- API 일관성
- 테스트 커버리지
- 문서화 수준

**산출물:**
- 코드 리뷰 코멘트
- 승인/변경요청 결정
- 보안 검토 보고서
- 리팩토링 제안

---

### 5. 🧪 QA Agent (품질보증 에이전트)

**책임 영역:**
- 통합 테스트 설계 및 실행
- 성능 벤치마킹
- 정확도 검증
- 회귀 테스트 관리
- 릴리스 검증

**담당 이슈 유형:**
- `QA-001`: 정확도 벤치마킹 프레임워크
- `QA-002`: 성능 벤치마킹
- `QA-003`: 메모리 프로파일링
- `DIC-009`: 검증 테스트셋 구축

**산출물:**
- 테스트 스위트
- 벤치마크 결과
- 품질 리포트
- 릴리스 체크리스트

---

### 6. 📝 Documentation Agent (문서화 에이전트)

**책임 영역:**
- API 문서 작성
- 사용자 가이드 작성
- 아키텍처 문서 정리
- CHANGELOG 관리
- 예제 코드 작성

**담당 이슈 유형:**
- `RST-014`: rustdoc + mdbook 문서화
- `QA-006`: 문서 웹사이트 구축
- README, CONTRIBUTING 등

**산출물:**
- API 레퍼런스
- 튜토리얼
- 마이그레이션 가이드
- 릴리스 노트

---

## 난이도별 리소스 할당

작업 복잡도에 따라 적절한 컴퓨팅 리소스(모델 티어)를 할당합니다.

### 복잡도 레벨 정의

| 레벨 | 코드명 | 설명 | 예상 시간 | 모델 티어 |
|------|--------|------|-----------|-----------|
| **S** | Small | 단순 수정, 문서 업데이트 | < 2시간 | Tier 1 (Fast) |
| **M** | Medium | 단일 모듈, 명확한 범위 | 2-8시간 | Tier 2 (Balanced) |
| **L** | Large | 다중 모듈, 설계 필요 | 1-3일 | Tier 2 (Balanced) |
| **XL** | Extra Large | 핵심 알고리즘, 복잡한 로직 | 1-2주 | Tier 3 (Advanced) |

### 모델 티어 특성

#### Tier 1 (Fast) - 빠른 처리
```yaml
적용 대상:
  - 단순 버그 수정
  - 문서 오타 수정
  - 의존성 버전 업데이트
  - 간단한 리팩토링
  - 테스트 케이스 추가

특성:
  - 빠른 응답 속도
  - 낮은 리소스 소비
  - 반복 작업에 적합
```

#### Tier 2 (Balanced) - 균형 잡힌 처리
```yaml
적용 대상:
  - 새로운 기능 구현
  - 중간 규모 리팩토링
  - API 설계
  - 테스트 스위트 작성
  - 문서 작성

특성:
  - 적절한 품질과 속도 균형
  - 표준 개발 작업에 최적화
  - 대부분의 작업에 적합
```

#### Tier 3 (Advanced) - 심층 분석
```yaml
적용 대상:
  - 핵심 알고리즘 구현 (Viterbi, Double-Array Trie)
  - 아키텍처 설계 결정
  - CRF 학습 파이프라인
  - 복잡한 성능 최적화
  - 보안 감사

특성:
  - 깊은 분석 능력
  - 복잡한 추론
  - 높은 정확도
  - 창의적 문제 해결
```

### 이슈별 티어 매핑

```
┌─────────────────────────────────────────────────────────────┐
│                     TIER 3 (Advanced)                       │
├─────────────────────────────────────────────────────────────┤
│ RST-004  Double-Array Trie 구현              [XL]           │
│ RST-005  Viterbi 알고리즘 구현               [XL]           │
│ DIC-008  CRF 기반 연결 비용 재학습           [XL]           │
│ RST-001  Lindera 분석 및 포크 결정           [M] (전략적)    │
│ DIC-010  바이너리 사전 포맷 v3.0 설계        [L] (전략적)    │
└─────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│                    TIER 2 (Balanced)                        │
├─────────────────────────────────────────────────────────────┤
│ DIC-001  mecab-ko-dic 포맷 분석              [M]            │
│ DIC-003  Modu 코퍼스 수집                    [M]            │
│ RST-006  연결 비용 행렬 로더                 [M]            │
│ RST-011  사용자 사전 지원                    [M]            │
│ RST-012  CLI 인터페이스                      [M]            │
│ BND-001  konlpy 호환 API 설계                [M]            │
│ BND-002  PyO3 바인딩 구현                    [L]            │
│ BND-004  WASM 바인딩                         [L]            │
│ QA-001   정확도 벤치마킹 프레임워크          [M]            │
└─────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│                      TIER 1 (Fast)                          │
├─────────────────────────────────────────────────────────────┤
│ RST-003  Cargo.toml 초기 설정                [S]            │
│ RST-013  로깅 시스템                         [S]            │
│ DIC-002  Sejong 품사 태그 체계 검토          [S]            │
│ QA-004   CI/CD 파이프라인                    [M]            │
│ RST-014  문서화                              [M]            │
│ QA-005   crates.io 배포 자동화               [S]            │
└─────────────────────────────────────────────────────────────┘
```

---

## 에이전트 협업 워크플로우

### 이슈 처리 흐름

```
┌──────────────────────────────────────────────────────────────────────────┐
│                           ISSUE LIFECYCLE                                │
└──────────────────────────────────────────────────────────────────────────┘

     ┌─────────┐
     │  Issue  │
     │ Created │
     └────┬────┘
          │
          ▼
    ┌───────────┐      ┌────────────────────────────────────┐
    │  ANALYST  │──────│ 1. 요구사항 분석                    │
    │   Agent   │      │ 2. 기술 조사                        │
    └─────┬─────┘      │ 3. 접근 방식 제안                   │
          │            └────────────────────────────────────┘
          ▼
    ┌───────────┐      ┌────────────────────────────────────┐
    │ ARCHITECT │──────│ 1. 설계 검토                        │
    │   Agent   │      │ 2. 아키텍처 결정                    │
    └─────┬─────┘      │ 3. 작업 분해                        │
          │            └────────────────────────────────────┘
          ▼
    ┌───────────┐      ┌────────────────────────────────────┐
    │ DEVELOPER │──────│ 1. 구현                             │
    │   Agent   │      │ 2. 단위 테스트                      │
    └─────┬─────┘      │ 3. PR 생성                          │
          │            └────────────────────────────────────┘
          ▼
    ┌───────────┐      ┌────────────────────────────────────┐
    │ REVIEWER  │──────│ 1. 코드 품질 검토                   │
    │   Agent   │      │ 2. 보안 검토                        │
    └─────┬─────┘      │ 3. 승인/변경요청                    │
          │            └────────────────────────────────────┘
          ▼
    ┌───────────┐      ┌────────────────────────────────────┐
    │    QA     │──────│ 1. 통합 테스트                      │
    │   Agent   │      │ 2. 성능 검증                        │
    └─────┬─────┘      │ 3. 회귀 테스트                      │
          │            └────────────────────────────────────┘
          ▼
    ┌───────────┐      ┌────────────────────────────────────┐
    │   DOCS    │──────│ 1. API 문서 업데이트                │
    │   Agent   │      │ 2. CHANGELOG 갱신                   │
    └─────┬─────┘      │ 3. 예제 코드                        │
          │            └────────────────────────────────────┘
          ▼
     ┌─────────┐
     │   PR    │
     │ Merged  │
     └─────────┘
```

### 에이전트 간 핸드오프 규칙

```yaml
Analyst → Architect:
  trigger: 분석 완료
  deliverables:
    - 기술 분석 문서
    - 접근 방식 옵션
  handoff_comment: "[Analysis Complete] @architect 설계 검토 요청"

Architect → Developer:
  trigger: 설계 승인
  deliverables:
    - 설계 문서/ADR
    - 작업 분해 목록
    - 인터페이스 정의
  handoff_comment: "[Design Approved] @developer 구현 시작"

Developer → Reviewer:
  trigger: PR 생성
  deliverables:
    - 구현 코드
    - 단위 테스트
    - PR 설명
  handoff_comment: "[Ready for Review] @reviewer 코드 리뷰 요청"

Reviewer → QA:
  trigger: 코드 리뷰 승인
  deliverables:
    - 리뷰 승인
    - 개선 제안 (있는 경우)
  handoff_comment: "[Review Approved] @qa 검증 요청"

QA → Documentation:
  trigger: 모든 테스트 통과
  deliverables:
    - 테스트 결과
    - 성능 벤치마크
  handoff_comment: "[Tests Passed] @docs 문서화 요청"

Documentation → Merge:
  trigger: 문서 완료
  deliverables:
    - 업데이트된 문서
    - CHANGELOG 항목
  handoff_comment: "[Docs Complete] PR 병합 준비 완료"
```

---

## 동시 작업 관리

### 병렬 처리 가능 작업

여러 에이전트가 동시에 작업할 수 있는 경우:

```
Sprint 1-2 병렬 작업:
├── Analyst Agent
│   ├── DIC-001: mecab-ko-dic 포맷 분석
│   └── DIC-002: Sejong 품사 태그 검토
│
├── Architect Agent
│   ├── RST-001: Lindera 분석
│   └── RST-002: 워크스페이스 설계
│
└── Developer Agent (RST-008 완료 후)
    └── RST-003: Cargo.toml 초기 설정
```

### 의존성 있는 순차 작업

반드시 순서대로 진행해야 하는 작업:

```
Critical Path:
DIC-001 ──▶ DIC-008 ──▶ DIC-009
(분석)      (학습)      (검증)

RST-001 ──▶ RST-004 ──▶ RST-005 ──▶ RST-010
(분석)      (Trie)      (Viterbi)   (N-best)
```

---

## 에이전트 설정 템플릿

각 에이전트의 동작을 정의하는 설정 파일:

```yaml
# .agents/analyst.yaml
name: Analyst Agent
role: analysis
tier: 2  # 기본 티어

responsibilities:
  - 기술 조사
  - 데이터 분석
  - 요구사항 정제

triggers:
  - label: "needs-analysis"
  - comment: "@analyst"

outputs:
  - type: markdown
    template: templates/analysis-report.md
  - type: comment
    format: structured

context:
  - docs/architecture/
  - docs/research/
  - ISSUE_BACKLOG.md
```

```yaml
# .agents/developer.yaml
name: Developer Agent
role: implementation
tier: 2  # 기본, 이슈 복잡도에 따라 동적 조정

responsibilities:
  - 코드 구현
  - 단위 테스트
  - PR 생성

tier_override:
  complexity_XL: 3  # XL 이슈는 Tier 3 사용
  complexity_S: 1   # S 이슈는 Tier 1 사용

coding_standards:
  - rust_edition: "2021"
  - unsafe_allowed: false
  - test_coverage_min: 80%

outputs:
  - type: code
    language: rust
  - type: tests
    framework: cargo-test
```

```yaml
# .agents/reviewer.yaml
name: Reviewer Agent
role: review
tier: 2

responsibilities:
  - 코드 품질 검토
  - 보안 분석
  - 성능 검토

review_checklist:
  - [ ] Rust safety (no unnecessary unsafe)
  - [ ] Error handling
  - [ ] Test coverage >= 80%
  - [ ] Documentation
  - [ ] Performance implications
  - [ ] Security considerations

outputs:
  - type: review
    format: github_review
  - type: suggestions
    inline: true
```

---

## 메트릭 및 모니터링

### 에이전트 성과 지표

```yaml
Analyst Agent:
  - 분석 정확도
  - 조사 완료 시간
  - 이슈 명확화율

Developer Agent:
  - 코드 품질 점수 (clippy)
  - 테스트 커버리지
  - PR 리뷰 통과율

Reviewer Agent:
  - 리뷰 처리 시간
  - 탐지된 이슈 수
  - False positive율

QA Agent:
  - 테스트 통과율
  - 회귀 탐지율
  - 벤치마크 정확도

Documentation Agent:
  - 문서 완성도
  - 예제 품질
  - 업데이트 적시성
```

### 자동화 대시보드 항목

```
Sprint Progress:
├── Issues: 12/30 completed (40%)
├── Story Points: 45/120 (37.5%)
├── Code Coverage: 72%
└── Documentation: 65%

Agent Activity (Last 7 Days):
├── Analyst: 8 analyses completed
├── Developer: 15 PRs created
├── Reviewer: 12 reviews completed
├── QA: 45 test runs
└── Docs: 3 documents updated
```

---

## 참고 자료

- [DEVELOPMENT_WORKFLOW.md](./DEVELOPMENT_WORKFLOW.md) - 상세 개발 프로세스
- [CONTRIBUTING.md](./CONTRIBUTING.md) - 기여 가이드라인
- [PROJECT_PLAN.md](./PROJECT_PLAN.md) - 전체 프로젝트 계획
- [ISSUE_BACKLOG.md](./ISSUE_BACKLOG.md) - 이슈 백로그

---

*Last Updated: 2026-01-04*  
*Maintainer: hephaex (hephaex@gmail.com)*
