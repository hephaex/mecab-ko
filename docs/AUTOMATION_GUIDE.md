# Automation & Tooling Guide

> **Project**: MeCab-Ko - Korean Morphological Analyzer in Rust  
> **Author**: hephaex (hephaex@gmail.com)  
> **Repository**: https://github.com/hephaex/mecab-ko

---

## 개요

MeCab-Ko 프로젝트는 개발 효율성을 위해 다양한 자동화 도구를 활용합니다. 이 문서는 작업 복잡도에 따른 적절한 도구 선택 가이드를 제공합니다.

---

## 작업 복잡도 분류

### 복잡도 레벨 정의

| 레벨 | 코드명 | 특성 | 예상 시간 | 처리 방식 |
|------|--------|------|-----------|-----------|
| **S** | Small | 단순, 명확, 반복적 | < 2시간 | 자동화/스크립트 |
| **M** | Medium | 단일 모듈, 명확한 범위 | 2-8시간 | 표준 도구 |
| **L** | Large | 다중 모듈, 설계 필요 | 1-3일 | 심층 분석 도구 |
| **XL** | Extra Large | 핵심 알고리즘, 복잡한 로직 | 1-2주 | 전문가 검토 + 고급 도구 |

---

## 복잡도별 도구 선택

### Tier 1: 경량 자동화 (S 복잡도)

단순하고 반복적인 작업에 적합합니다.

```yaml
적용 작업:
  - 의존성 버전 업데이트
  - 간단한 버그 수정
  - 문서 오타 수정
  - 코드 포맷팅
  - 린트 수정

도구:
  - Dependabot / Renovate
  - cargo fmt / cargo clippy --fix
  - 정규식 기반 검색/치환
  - GitHub Actions 자동화

예시 자동화:
  # 자동 의존성 업데이트
  .github/dependabot.yml:
    - package-ecosystem: cargo
      schedule:
        interval: weekly
```

### Tier 2: 표준 개발 도구 (M/L 복잡도)

대부분의 개발 작업에 사용됩니다.

```yaml
적용 작업:
  - 새로운 기능 구현
  - API 설계 및 구현
  - 테스트 작성
  - 문서 작성
  - 리팩토링

도구:
  - IDE 코드 어시스턴트
  - 코드 생성 도구
  - 테스트 프레임워크
  - 벤치마킹 도구
  - 문서 생성기

워크플로우:
  1. 이슈 분석 및 계획
  2. 브랜치 생성
  3. 구현 및 테스트
  4. 코드 리뷰
  5. 병합
```

### Tier 3: 고급 분석 도구 (XL 복잡도)

핵심 알고리즘과 아키텍처 결정에 사용됩니다.

```yaml
적용 작업:
  - 핵심 알고리즘 구현 (Viterbi, Trie)
  - 아키텍처 설계 결정
  - 성능 최적화
  - 보안 감사
  - 복잡한 버그 분석

도구:
  - 성능 프로파일러 (perf, flamegraph)
  - 메모리 분석기 (Valgrind, heaptrack)
  - 정적 분석 도구
  - 퍼징 테스터 (cargo-fuzz)
  - 아키텍처 다이어그램 도구

워크플로우:
  1. 심층 분석 및 설계
  2. 프로토타입 구현
  3. 성능 벤치마킹
  4. 반복 개선
  5. 전문가 리뷰
```

---

## 이슈별 도구 매핑

### Epic 1: Dictionary (DIC)

```
┌────────────────────────────────────────────────────────────┐
│ Issue ID │ Title                    │ Complexity │ Tier   │
├──────────┼──────────────────────────┼────────────┼────────┤
│ DIC-001  │ mecab-ko-dic 포맷 분석    │     M      │ Tier 2 │
│ DIC-002  │ Sejong 품사 태그 검토     │     S      │ Tier 1 │
│ DIC-003  │ Modu 코퍼스 수집          │     M      │ Tier 2 │
│ DIC-008  │ CRF 연결 비용 재학습      │    XL      │ Tier 3 │
│ DIC-010  │ 바이너리 사전 포맷 설계   │     L      │ Tier 3 │
└────────────────────────────────────────────────────────────┘
```

### Epic 2: Rust Core (RST)

```
┌────────────────────────────────────────────────────────────┐
│ Issue ID │ Title                    │ Complexity │ Tier   │
├──────────┼──────────────────────────┼────────────┼────────┤
│ RST-001  │ Lindera 분석/포크 결정    │     M      │ Tier 3 │
│ RST-003  │ Cargo.toml 초기 설정     │     S      │ Tier 1 │
│ RST-004  │ Double-Array Trie 구현   │    XL      │ Tier 3 │
│ RST-005  │ Viterbi 알고리즘 구현    │    XL      │ Tier 3 │
│ RST-008  │ 한글 자모 유틸리티       │     M      │ Tier 2 │
│ RST-012  │ CLI 인터페이스           │     M      │ Tier 2 │
└────────────────────────────────────────────────────────────┘
```

### Epic 3: Bindings (BND)

```
┌────────────────────────────────────────────────────────────┐
│ Issue ID │ Title                    │ Complexity │ Tier   │
├──────────┼──────────────────────────┼────────────┼────────┤
│ BND-001  │ konlpy 호환 API 설계     │     M      │ Tier 2 │
│ BND-002  │ PyO3 바인딩 구현         │     L      │ Tier 2 │
│ BND-004  │ WASM 바인딩              │     L      │ Tier 2 │
│ BND-005  │ Nori 호환 레이어         │     L      │ Tier 2 │
└────────────────────────────────────────────────────────────┘
```

### Epic 4: Quality (QA)

```
┌────────────────────────────────────────────────────────────┐
│ Issue ID │ Title                    │ Complexity │ Tier   │
├──────────┼──────────────────────────┼────────────┼────────┤
│ QA-001   │ 정확도 벤치마킹 프레임워크│     M      │ Tier 2 │
│ QA-003   │ 메모리 프로파일링        │     M      │ Tier 3 │
│ QA-004   │ CI/CD 파이프라인         │     M      │ Tier 1 │
│ QA-005   │ crates.io 배포 자동화    │     S      │ Tier 1 │
└────────────────────────────────────────────────────────────┘
```

---

## 자동화 스크립트

### 이슈 생성 자동화

```bash
#!/bin/bash
# scripts/create-issue.sh

EPIC=$1
TITLE=$2
COMPLEXITY=$3

gh issue create \
  --title "[$EPIC] $TITLE" \
  --label "$EPIC,complexity-$COMPLEXITY" \
  --body-file .github/ISSUE_TEMPLATE/task.md
```

### 복잡도 기반 라벨링

```yaml
# .github/labeler.yml
complexity-S:
  - changed-files:
    - any-glob-to-any-file: ['**/*.md', '**/Cargo.toml']

complexity-M:
  - changed-files:
    - any-glob-to-any-file: ['**/src/**/*.rs']
    - all-globs-to-any-file:
      - '!**/lib.rs'
```

### 티어별 리뷰 자동 할당

```yaml
# .github/CODEOWNERS
# Tier 3 (XL 이슈) - 메인테이너 직접 리뷰
crates/mecab-ko-core/src/viterbi.rs @hephaex
crates/mecab-ko-dict/src/trie.rs @hephaex

# Tier 2 - 기여자 리뷰 가능
crates/mecab-ko-hangul/ @hephaex
crates/mecab-ko-cli/ @hephaex

# Tier 1 - 자동 병합 가능 (CI 통과 시)
*.md
```

---

## 도구 통합 설정

### Cargo.toml 품질 설정

```toml
[workspace.lints.clippy]
# Tier 3 요구사항
unwrap_used = "deny"
expect_used = "deny"
panic = "deny"

# Tier 2 요구사항  
pedantic = { level = "warn", priority = -1 }

# 프로젝트별
missing_docs = "warn"
```

### VS Code 설정

```json
// .vscode/settings.json
{
  "rust-analyzer.checkOnSave.command": "clippy",
  "rust-analyzer.checkOnSave.extraArgs": [
    "--all-features",
    "--",
    "-D", "warnings"
  ],
  "editor.formatOnSave": true,
  "[rust]": {
    "editor.defaultFormatter": "rust-lang.rust-analyzer"
  }
}
```

---

## 작업 흐름 최적화

### 복잡도별 예상 시간

```
S (Small):
├── 분석: 30분
├── 구현: 1시간
├── 테스트: 30분
├── 리뷰: 15분
└── 총계: ~2시간

M (Medium):
├── 분석: 2시간
├── 구현: 4시간
├── 테스트: 2시간
├── 리뷰: 1시간
└── 총계: ~1일

L (Large):
├── 분석: 4시간
├── 구현: 12시간
├── 테스트: 4시간
├── 리뷰: 2시간
└── 총계: ~3일

XL (Extra Large):
├── 분석: 8시간
├── 설계: 8시간
├── 구현: 40시간
├── 테스트: 16시간
├── 리뷰: 4시간
└── 총계: ~2주
```

### 병렬 처리 전략

```
Sprint N 병렬 작업 구성:

Stream A (Tier 3):
└── XL 이슈 1개 (단독 진행)

Stream B (Tier 2):
├── M 이슈 2-3개
└── L 이슈 1개

Stream C (Tier 1):
├── S 이슈 5-10개
└── 자동화 태스크
```

---

## 도구 목록

### 개발 도구

| 도구 | 용도 | Tier |
|------|------|------|
| cargo | 빌드, 테스트, 배포 | All |
| rustfmt | 코드 포맷팅 | Tier 1 |
| clippy | 린트 | Tier 1-2 |
| rust-analyzer | IDE 지원 | Tier 2 |
| cargo-watch | 자동 재빌드 | Tier 2 |
| cargo-edit | 의존성 관리 | Tier 1 |

### 품질 도구

| 도구 | 용도 | Tier |
|------|------|------|
| cargo-llvm-cov | 커버리지 | Tier 2 |
| cargo-audit | 보안 감사 | Tier 1 |
| cargo-deny | 의존성 검사 | Tier 1 |
| cargo-fuzz | 퍼징 테스트 | Tier 3 |
| cargo-flamegraph | 성능 프로파일링 | Tier 3 |

### CI/CD 도구

| 도구 | 용도 | Tier |
|------|------|------|
| GitHub Actions | 자동화 파이프라인 | All |
| Dependabot | 의존성 업데이트 | Tier 1 |
| Codecov | 커버리지 리포트 | Tier 2 |
| GitHub Releases | 릴리스 관리 | Tier 1 |

---

## 참고 자료

- [AGENTS.md](./AGENTS.md) - 에이전트 역할 정의
- [DEVELOPMENT_WORKFLOW.md](./DEVELOPMENT_WORKFLOW.md) - 개발 프로세스
- [CODE_QUALITY.md](./CODE_QUALITY.md) - 코드 품질 기준

---

*Last Updated: 2026-01-04*  
*Maintainer: hephaex (hephaex@gmail.com)*
